//! Canonical Reciprocal Rank Fusion (RRF) Engine.
//!
//! Blends lexical BM25 (SQLite FTS5) and vectorial cosine similarity (LanceDB)
//! using smoothing parameter k = 60.0 and exact match additive bonuses in <5ms CPU latency.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Canonical smoothing constant k for Reciprocal Rank Fusion.
pub const DEFAULT_RRF_K: f64 = 60.0;

/// Additive score bonus for exact rigid term matches (ADRs, paths, system constants).
pub const EXACT_MATCH_BONUS: f64 = 10.0;

/// Rigid system keywords that automatically trigger exact match bonus if present.
pub const EXACT_RIGID_KEYWORDS: &[&str] = &[
    "GGML_TYPE_TQ1",
    "ADR-001",
    "ADR-003",
    "ADR-005",
    "ADR-025",
    "ADR-027",
    "ADR-030",
    "ADR-040",
    "ADR-047",
    "windows-sys",
    "winapi",
    "core_affinity",
    "LadybugDB",
    "LanceDB",
    "FrankenSQLite",
    "ChyrosDaemon",
];

/// Candidate item produced by lexical search (SQLite FTS5 BM25).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LexicalCandidate {
    pub id: String,
    pub content: String,
    pub category: String,
    pub score: f64,
}

/// Candidate item produced by dense vector search (LanceDB cosine similarity).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorCandidate {
    pub id: String,
    pub content: String,
    pub category: String,
    pub similarity: f64,
}

/// Unified search match synthesized by the RRF engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedMatch {
    pub id: String,
    pub content: String,
    pub category: String,
    pub rrf_score: f64,
    pub lexical_rank: Option<usize>,
    pub vector_rank: Option<usize>,
    pub is_exact_match: bool,
}

/// Determines whether a query and candidate content contain rigid exact matches.
pub fn is_exact_term_match(query: &str, content: &str, id: &str) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return false;
    }

    // Direct case-insensitive substring match of full query
    let q_lower = q.to_lowercase();
    let c_lower = content.to_lowercase();
    let id_lower = id.to_lowercase();

    if c_lower.contains(&q_lower) || id_lower.contains(&q_lower) {
        return true;
    }

    // Check rigid keyword triggers
    for &kw in EXACT_RIGID_KEYWORDS {
        if q.contains(kw) && (content.contains(kw) || id.contains(kw)) {
            return true;
        }
    }

    false
}

/// Engine executing Reciprocal Rank Fusion on CPU host with strict latency guarantees.
#[derive(Debug, Clone)]
pub struct RrfFusionEngine {
    pub k: f64,
}

impl Default for RrfFusionEngine {
    fn default() -> Self {
        Self { k: DEFAULT_RRF_K }
    }
}

type RrfAccumulator = (f64, Option<usize>, Option<usize>, String, String, bool);

impl RrfFusionEngine {
    /// Instantiates engine with custom smoothing parameter `k`.
    pub fn new(k: f64) -> Self {
        Self { k }
    }

    /// Fuses lexical and vector candidate lists with tombstone filtering and exact match bonuses.
    pub fn fuse_with_query(
        &self,
        query: &str,
        lexical: &[LexicalCandidate],
        vectorial: &[VectorCandidate],
        tombstones: &HashSet<String>,
    ) -> (Vec<UnifiedMatch>, Duration) {
        let start = Instant::now();

        // Map: id -> (rrf_score, lexical_rank, vector_rank, content, category, is_exact)
        let mut map: HashMap<String, RrfAccumulator> =
            HashMap::with_capacity(lexical.len() + vectorial.len());

        // Process lexical ranks (1-based)
        for (idx, item) in lexical.iter().enumerate() {
            if tombstones.contains(&item.id) {
                continue;
            }

            let rank = idx + 1;
            let base_score = 1.0 / (self.k + rank as f64);
            let exact = is_exact_term_match(query, &item.content, &item.id);
            let score = if exact {
                base_score + EXACT_MATCH_BONUS
            } else {
                base_score
            };

            map.entry(item.id.clone())
                .and_modify(|(s, r_lex, _, _, _, is_ex)| {
                    *s += base_score;
                    *r_lex = Some(rank);
                    if exact {
                        *s += EXACT_MATCH_BONUS;
                        *is_ex = true;
                    }
                })
                .or_insert((
                    score,
                    Some(rank),
                    None,
                    item.content.clone(),
                    item.category.clone(),
                    exact,
                ));
        }

        // Process vectorial ranks (1-based)
        for (idx, item) in vectorial.iter().enumerate() {
            if tombstones.contains(&item.id) {
                continue;
            }

            let rank = idx + 1;
            let base_score = 1.0 / (self.k + rank as f64);
            let exact = is_exact_term_match(query, &item.content, &item.id);
            let score = if exact {
                base_score + EXACT_MATCH_BONUS
            } else {
                base_score
            };

            map.entry(item.id.clone())
                .and_modify(|(s, _, r_vec, _, _, is_ex)| {
                    *s += base_score;
                    *r_vec = Some(rank);
                    if exact {
                        *s += EXACT_MATCH_BONUS;
                        *is_ex = true;
                    }
                })
                .or_insert((
                    score,
                    None,
                    Some(rank),
                    item.content.clone(),
                    item.category.clone(),
                    exact,
                ));
        }

        // Convert accumulator into unified result vector
        let mut results: Vec<UnifiedMatch> = map
            .into_iter()
            .map(|(id, (score, r_lex, r_vec, content, category, is_exact))| UnifiedMatch {
                id,
                content,
                category,
                rrf_score: score,
                lexical_rank: r_lex,
                vector_rank: r_vec,
                is_exact_match: is_exact,
            })
            .collect();

        // Sort descending by RRF score (highest priority first)
        results.sort_by(|a, b| {
            b.rrf_score
                .partial_cmp(&a.rrf_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let elapsed = start.elapsed();
        (results, elapsed)
    }

    /// Compatibility helper without explicit query bonus.
    pub fn fuse(
        &self,
        lexical: &[LexicalCandidate],
        vectorial: &[VectorCandidate],
        tombstones: &HashSet<String>,
    ) -> Vec<UnifiedMatch> {
        let (results, _) = self.fuse_with_query("", lexical, vectorial, tombstones);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_and_exact_match_bonus() {
        let engine = RrfFusionEngine::default();
        let tombstones = HashSet::new();

        let lexical = vec![
            LexicalCandidate {
                id: "mem_1".to_string(),
                content: "General architecture rules".to_string(),
                category: "arch".to_string(),
                score: 10.5,
            },
            LexicalCandidate {
                id: "mem_2".to_string(),
                content: "FrankenSQLite WAL configuration details".to_string(),
                category: "db".to_string(),
                score: 8.2,
            },
        ];

        let vectorial = vec![
            VectorCandidate {
                id: "mem_3".to_string(),
                content: "Unrelated topic".to_string(),
                category: "misc".to_string(),
                similarity: 0.95,
            },
            VectorCandidate {
                id: "mem_2".to_string(),
                content: "FrankenSQLite WAL configuration details".to_string(),
                category: "db".to_string(),
                similarity: 0.88,
            },
        ];

        // Query mentioning exact rigid keyword 'FrankenSQLite'
        let (results, duration) =
            engine.fuse_with_query("Configure FrankenSQLite", &lexical, &vectorial, &tombstones);

        // Verification: Latency must be strictly sub-5ms (typically < 0.2ms)
        assert!(
            duration < Duration::from_millis(5),
            "RRF execution took too long: {:?}",
            duration
        );

        assert_eq!(results.len(), 3);

        // mem_2 has exact match bonus (+10) + lexical rank 2 + vector rank 2
        // It must be ranked #1
        assert_eq!(results[0].id, "mem_2");
        assert!(results[0].is_exact_match);
        assert!(results[0].rrf_score > 10.0);
        assert_eq!(results[0].lexical_rank, Some(2));
        assert_eq!(results[0].vector_rank, Some(2));
    }

    #[test]
    fn test_rrf_tombstones_filtering() {
        let engine = RrfFusionEngine::default();
        let mut tombstones = HashSet::new();
        tombstones.insert("mem_dead".to_string());

        let lexical = vec![LexicalCandidate {
            id: "mem_dead".to_string(),
            content: "Superseded obsolete content".to_string(),
            category: "deprecated".to_string(),
            score: 20.0,
        }];

        let vectorial = vec![VectorCandidate {
            id: "mem_alive".to_string(),
            content: "Active living memory".to_string(),
            category: "active".to_string(),
            similarity: 0.7,
        }];

        let (results, _) = engine.fuse_with_query("query", &lexical, &vectorial, &tombstones);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "mem_alive");
    }
}
