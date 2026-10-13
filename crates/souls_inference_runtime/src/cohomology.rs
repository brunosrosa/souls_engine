//! Socratic Sheaf Cohomology Engine (`cohomology.rs`) for Souls Engine (v7).
//!
//! Evaluates logical consistency and sheaf cohomology over premise graphs
//! using Galois Field arithmetic GF(2) (AND/XOR bitwise ops) packed into `[u64; 4]`.
//!
//! Mathematical Foundations:
//! - Strictly models H¹ via the difference of ranks: `H¹ = rank([A|b]) - rank(A)` (Rouché-Capelli over GF(2)).
//! - Harmonious cycles (b = 0) yield H¹ = 0 (consistent system, no hallucination).
//! - Conflicting premises (b ≠ 0 outside im(A)) yield H¹ > 0 (provable contradiction).
//! - 32× stack footprint reduction (~8.2 KB in stack vs 256 KB in f32) with zero floating-point drift.
//!
//! Conforms to CRATES_SPECIFICATION.md Section 2.5 and ADR-047.

/// Physical ceiling for knowledge facts/vertices in the graph.
/// 256 bits = 4 u64 words (`WORDS_PER_ROW`).
pub const MAX_FACTS: usize = 256;

/// Physical ceiling for relations/constraints.
pub const MAX_RELATIONS: usize = 256;

/// Number of 64-bit words per row (256 / 64 = 4).
pub const WORDS_PER_ROW: usize = MAX_FACTS / 64;

/// Canonical relation types encoding linear constraints over GF(2):
/// - `DependsOn`: x_u = x_v (x_u + x_v = 0; b=0)
/// - `Implies`: x_u + x_v = 0; b=0
/// - `ConflictsWith`: x_u ⊕ x_v = 1 (x_u + x_v = 1; b=1)
/// - `EquivalentTo`: x_u = x_v (x_u + x_v = 0; b=0)
/// - `Negates`: x_u = ¬x_v (x_u + x_v = 1; b=1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    DependsOn,
    Implies,
    ConflictsWith,
    EquivalentTo,
    Negates,
}

impl RelationKind {
    /// Parses textual representation into relation kind with conservative fallback.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "depends_on" | "depends" => Self::DependsOn,
            "implies" | "implica" => Self::Implies,
            "conflicts_with" | "conflicts" => Self::ConflictsWith,
            "equivalent_to" | "equivalent" | "equals" => Self::EquivalentTo,
            "negates" | "contradicts" | "not" => Self::Negates,
            _ => Self::DependsOn,
        }
    }
}

/// Edge in the knowledge graph: index pair (from, to) with linear constraint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactEdge {
    pub from_idx: u16,
    pub to_idx: u16,
    pub kind: RelationKind,
}

/// Socratic knowledge graph representation in memory.
#[derive(Debug, Clone, Default)]
pub struct SocraticFactGraph {
    /// Names of vertices (facts / propositions). Up to `MAX_FACTS`.
    pub vertex_names: Vec<String>,
    /// Constraints / relations. Up to `MAX_RELATIONS`.
    pub edges: Vec<FactEdge>,
}

impl SocraticFactGraph {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertex_names: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Constructs graph from named propositions and indexed relations.
    #[must_use]
    pub fn from_parts(vertices: Vec<String>, edges: Vec<(usize, usize, RelationKind)>) -> Self {
        let edges = edges
            .into_iter()
            .filter(|(u, v, _)| u != v && *u < MAX_FACTS && *v < MAX_FACTS)
            .map(|(u, v, kind)| FactEdge {
                from_idx: u as u16,
                to_idx: v as u16,
                kind,
            })
            .collect();
        Self {
            vertex_names: vertices,
            edges,
        }
    }

    /// Adds a proposition vertex, returning its index.
    pub fn add_vertex(&mut self, name: impl Into<String>) -> Option<usize> {
        if self.vertex_names.len() >= MAX_FACTS {
            return None;
        }
        let idx = self.vertex_names.len();
        self.vertex_names.push(name.into());
        Some(idx)
    }

    /// Adds a constraint edge between two vertices.
    pub fn add_edge(&mut self, u: usize, v: usize, kind: RelationKind) -> bool {
        if u >= self.vertex_names.len() || v >= self.vertex_names.len() || u == v {
            return false;
        }
        if self.edges.len() >= MAX_RELATIONS {
            return false;
        }
        self.edges.push(FactEdge {
            from_idx: u as u16,
            to_idx: v as u16,
            kind,
        });
        true
    }

    #[must_use]
    pub fn n_vertices(&self) -> usize {
        self.vertex_names.len()
    }

    #[must_use]
    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }
}

/// Augmented constraint matrix [A | b] over GF(2) packed in stack memory.
///
/// Memory Footprint:
/// - coefs: [[u64; 4]; 256] = 8,192 bytes (8 KB)
/// - b_data: [u64; 4] = 32 bytes
/// - Total: ~8.2 KB in stack, zero dynamic heap allocations during Gaussian elimination.
pub struct RestrictionMatrix {
    coefs: [[u64; WORDS_PER_ROW]; MAX_RELATIONS],
    b_data: [u64; WORDS_PER_ROW],
    rows: usize,
    cols: usize,
}

impl RestrictionMatrix {
    /// Creates a zeroed matrix for up to `n_vars` variables and `n_constraints` rows.
    #[must_use]
    pub fn new(n_vars: usize, n_constraints: usize) -> Self {
        assert!(n_vars <= MAX_FACTS);
        assert!(n_constraints <= MAX_RELATIONS);
        Self {
            coefs: [[0u64; WORDS_PER_ROW]; MAX_RELATIONS],
            b_data: [0u64; WORDS_PER_ROW],
            rows: 0,
            cols: n_vars,
        }
    }

    /// Appends a linear constraint row: a_1·x_1 ⊕ ... ⊕ a_n·x_n = b.
    pub fn push_row(&mut self, coeffs: &[u8], b: u8) {
        assert!(self.rows < MAX_RELATIONS, "Restriction matrix is full");
        assert!(coeffs.len() <= self.cols);

        for (j, &c) in coeffs.iter().enumerate() {
            if c != 0 {
                self.set_bit(self.rows, j, true);
            }
        }
        if b != 0 {
            self.set_b_bit(self.rows, true);
        }
        self.rows += 1;
    }

    #[inline]
    fn set_bit(&mut self, row: usize, col: usize, val: bool) {
        let (word, bit) = (col / 64, col % 64);
        if val {
            self.coefs[row][word] |= 1u64 << bit;
        } else {
            self.coefs[row][word] &= !(1u64 << bit);
        }
    }

    #[inline]
    fn set_b_bit(&mut self, row: usize, val: bool) {
        let (word, bit) = (row / 64, row % 64);
        if val {
            self.b_data[word] |= 1u64 << bit;
        } else {
            self.b_data[word] &= !(1u64 << bit);
        }
    }

    #[inline]
    #[must_use]
    pub fn get_bit(&self, row: usize, col: usize) -> bool {
        let (word, bit) = (col / 64, col % 64);
        (self.coefs[row][word] >> bit) & 1 == 1
    }

    #[inline]
    #[must_use]
    pub fn get_b_bit(&self, row: usize) -> bool {
        let (word, bit) = (row / 64, row % 64);
        (self.b_data[word] >> bit) & 1 == 1
    }

    #[must_use]
    pub fn n_rows(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub fn n_vars(&self) -> usize {
        self.cols
    }

    #[inline]
    fn swap_rows(&mut self, a: usize, b: usize) {
        if a != b {
            self.coefs.swap(a, b);
            let abit = self.get_b_bit(a);
            let bbit = self.get_b_bit(b);
            self.set_b_bit(a, bbit);
            self.set_b_bit(b, abit);
        }
    }

    #[inline]
    fn xor_rows_from(&mut self, target: usize, source: usize, start_col: usize) {
        let start_word = start_col / 64;
        for w in start_word..WORDS_PER_ROW {
            self.coefs[target][w] ^= self.coefs[source][w];
        }
        let tbit = self.get_b_bit(target);
        let sbit = self.get_b_bit(source);
        self.set_b_bit(target, tbit ^ sbit);
    }

    #[inline]
    fn find_pivot(&self, col: usize, from_row: usize) -> Option<usize> {
        let (word, bit) = (col / 64, col % 64);
        let mask = 1u64 << bit;
        (from_row..self.rows).find(|&r| (self.coefs[r][word] & mask) != 0)
    }
}

/// Output structure of Socratic sheaf cohomology analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CohomologyResult {
    /// Dimension of the first cohomology group H¹.
    /// H¹ = 0 indicates absolute consistency (even with loops).
    /// H¹ > 0 indicates provable logical contradiction.
    pub h1_dimension: usize,
    /// Boolean flag: true if H¹ > 0.
    pub has_contradiction: bool,
    /// Homogeneous rank of the constraint matrix over GF(2).
    pub rank: usize,
}

/// In-place Gaussian elimination over GF(2) computing rank of homogeneous matrix A.
#[must_use]
pub fn rank_homogeneous(mat: &mut RestrictionMatrix) -> usize {
    let mut rank = 0;
    let mut row_idx = 0;
    let n = mat.n_vars();

    for col in 0..n {
        let Some(pivot_r) = mat.find_pivot(col, row_idx) else {
            continue;
        };
        if pivot_r != row_idx {
            mat.swap_rows(pivot_r, row_idx);
        }
        for r in (row_idx + 1)..mat.n_rows() {
            if mat.get_bit(r, col) {
                mat.xor_rows_from(r, row_idx, col);
            }
        }
        row_idx += 1;
        rank += 1;
    }
    rank
}

/// In-place Gaussian elimination over GF(2) computing rank of augmented matrix [A | b].
#[must_use]
pub fn rank_augmented(mat: &mut RestrictionMatrix) -> usize {
    let mut rank = 0;
    let mut row_idx = 0;
    let n = mat.n_vars();
    let total_cols = n + 1;

    for col in 0..total_cols {
        if col == n {
            // Pivoting on augmented column b
            let mut pivot_row: Option<usize> = None;
            for r in row_idx..mat.n_rows() {
                if mat.get_b_bit(r) {
                    pivot_row = Some(r);
                    break;
                }
            }
            let Some(pivot_r) = pivot_row else { continue };
            if pivot_r != row_idx {
                mat.swap_rows(pivot_r, row_idx);
            }
            for r in (row_idx + 1)..mat.n_rows() {
                if mat.get_b_bit(r) {
                    mat.xor_rows_from(r, row_idx, 0);
                }
            }
        } else {
            let Some(pivot_r) = mat.find_pivot(col, row_idx) else {
                continue;
            };
            if pivot_r != row_idx {
                mat.swap_rows(pivot_r, row_idx);
            }
            for r in (row_idx + 1)..mat.n_rows() {
                if mat.get_bit(r, col) {
                    mat.xor_rows_from(r, row_idx, col);
                }
            }
        }
        row_idx += 1;
        rank += 1;
    }
    rank
}

/// Computes sheaf cohomology over the given proposition graph.
///
/// H¹ = `rank([A|b]) - rank(A)` (Rouché-Capelli over GF(2)).
#[must_use]
pub fn compute_cohomology(graph: &SocraticFactGraph) -> CohomologyResult {
    let n = graph.n_vertices();
    let m = graph.n_edges();

    if n == 0 {
        return CohomologyResult {
            h1_dimension: 0,
            has_contradiction: false,
            rank: 0,
        };
    }

    let mut mat_h = RestrictionMatrix::new(n, m);
    let mut mat_aug = RestrictionMatrix::new(n, m);

    for edge in &graph.edges {
        let u = edge.from_idx as usize;
        let v = edge.to_idx as usize;
        if u >= n || v >= n {
            continue;
        }

        let mut coeffs = vec![0u8; n];
        let b: u8 = match edge.kind {
            RelationKind::DependsOn | RelationKind::EquivalentTo | RelationKind::Implies => {
                coeffs[u] = 1;
                coeffs[v] = 1;
                0
            }
            RelationKind::ConflictsWith | RelationKind::Negates => {
                coeffs[u] = 1;
                coeffs[v] = 1;
                1
            }
        };

        mat_h.push_row(&coeffs, 0);
        mat_aug.push_row(&coeffs, b);
    }

    let rank_h = rank_homogeneous(&mut mat_h);
    let rank_aug = rank_augmented(&mut mat_aug);
    let h1 = rank_aug.saturating_sub(rank_h);

    CohomologyResult {
        h1_dimension: h1,
        has_contradiction: h1 > 0,
        rank: rank_h,
    }
}

/// Deterministic memory conflict probability booster when H¹ > 0.
#[must_use]
pub fn boost_memory_conflict(base: f32, h1: &CohomologyResult) -> f32 {
    if h1.has_contradiction {
        base.max(0.86)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harmonious_cycle_has_zero_cohomology() {
        // A -> B -> C -> A (all DependsOn / consistent equivalence cycle)
        let mut graph = SocraticFactGraph::new();
        let a = graph.add_vertex("A").unwrap();
        let b = graph.add_vertex("B").unwrap();
        let c = graph.add_vertex("C").unwrap();

        graph.add_edge(a, b, RelationKind::DependsOn);
        graph.add_edge(b, c, RelationKind::DependsOn);
        graph.add_edge(c, a, RelationKind::DependsOn);

        let res = compute_cohomology(&graph);
        assert_eq!(res.h1_dimension, 0, "Harmonious cycle must have H1 = 0");
        assert!(!res.has_contradiction);
    }

    #[test]
    fn test_contradictory_triangle_detected_via_rouche_capelli() {
        // A == B, B == C, but A ConflictsWith C (x_A ⊕ x_C = 1)
        let mut graph = SocraticFactGraph::new();
        let a = graph.add_vertex("A").unwrap();
        let b = graph.add_vertex("B").unwrap();
        let c = graph.add_vertex("C").unwrap();

        graph.add_edge(a, b, RelationKind::EquivalentTo); // x_a + x_b = 0
        graph.add_edge(b, c, RelationKind::EquivalentTo); // x_b + x_c = 0
        graph.add_edge(a, c, RelationKind::ConflictsWith); // x_a + x_c = 1

        let res = compute_cohomology(&graph);
        assert!(res.h1_dimension > 0, "Contradiction must produce H1 > 0");
        assert!(res.has_contradiction);
        assert_eq!(res.h1_dimension, 1);
    }

    #[test]
    fn test_boost_memory_conflict_threshold() {
        let ok_result = CohomologyResult {
            h1_dimension: 0,
            has_contradiction: false,
            rank: 2,
        };
        assert_eq!(boost_memory_conflict(0.3, &ok_result), 0.3);

        let err_result = CohomologyResult {
            h1_dimension: 1,
            has_contradiction: true,
            rank: 2,
        };
        assert_eq!(boost_memory_conflict(0.3, &err_result), 0.86);
        assert_eq!(boost_memory_conflict(0.95, &err_result), 0.95);
    }
}
