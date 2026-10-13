//! In-Memory Deterministic Call Graph and Symbol Navigation Cache (`call_graph.rs`).
//!
//! Provides lock-free O(1) semantic navigation over repository symbols, callers, and callees.
//! Operates 100% in Host RAM via DashMap shards with zero SQLite contention.
//!
//! Conforms to CRATES_SPECIFICATION.md Section 2.3.

use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Syntactic symbol classification emitted during static AST parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Fn,
    Struct,
    Enum,
    Trait,
    Const,
    Static,
}

impl SymbolKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fn => "fn",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Const => "const",
            Self::Static => "static",
        }
    }
}

/// Physical file and line coordinates of a parsed symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    pub qualified_name: String,
    pub kind: SymbolKind,
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
}

/// Node within the directed CALL_GRAPH maintaining incoming and outgoing edges.
///
/// Directionality Invariant:
/// For every edge `a -> b`:
/// - `a.callees` contains `b` (outgoing)
/// - `b.callers` contains `a` (incoming)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallGraphNode {
    pub symbol: String,
    pub callers: HashSet<String>,
    pub callees: HashSet<String>,
    pub last_updated: i64,
}

/// Global shared singleton stores for symbol index and call graph.
static SYMBOL_INDEX: OnceLock<DashMap<String, SymbolEntry>> = OnceLock::new();
static CALL_GRAPH: OnceLock<DashMap<String, CallGraphNode>> = OnceLock::new();

#[must_use]
pub fn global_symbol_index() -> &'static DashMap<String, SymbolEntry> {
    SYMBOL_INDEX.get_or_init(DashMap::new)
}

#[must_use]
pub fn global_call_graph() -> &'static DashMap<String, CallGraphNode> {
    CALL_GRAPH.get_or_init(DashMap::new)
}

/// Manages symbol registration, file unindexing, and directed edge traversal.
#[derive(Debug, Clone)]
pub struct CallGraphStore {
    symbols: &'static DashMap<String, SymbolEntry>,
    graph: &'static DashMap<String, CallGraphNode>,
}

impl Default for CallGraphStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CallGraphStore {
    /// Obtains an instance backed by the global lock-free DashMaps.
    #[must_use]
    pub fn new() -> Self {
        Self {
            symbols: global_symbol_index(),
            graph: global_call_graph(),
        }
    }

    /// Registers or updates a symbol in the index.
    pub fn insert_symbol(&self, entry: SymbolEntry) {
        self.symbols.insert(entry.qualified_name.clone(), entry);
    }

    /// Looks up a symbol by its fully qualified name.
    #[must_use]
    pub fn lookup_symbol(&self, name: &str) -> Option<SymbolEntry> {
        self.symbols.get(name).map(|r| r.value().clone())
    }

    /// Removes all indexed symbols associated with a specific file path.
    pub fn remove_symbols_for_file(&self, path: &Path) -> usize {
        let to_remove: Vec<String> = self
            .symbols
            .iter()
            .filter(|kv| kv.value().file_path == path)
            .map(|kv| kv.key().clone())
            .collect();

        let count = to_remove.len();
        for k in to_remove {
            self.symbols.remove(&k);
        }
        count
    }

    /// Adds a directed invocation edge `caller -> callee`.
    pub fn insert_edge(&self, caller: &str, callee: &str, now_epoch: i64) {
        if caller == callee {
            return; // ignore self-recursion
        }

        // Caller -> outgoing edge to callee
        self.graph
            .entry(caller.to_string())
            .and_modify(|n| {
                n.callees.insert(callee.to_string());
                n.last_updated = now_epoch;
            })
            .or_insert_with(|| CallGraphNode {
                symbol: caller.to_string(),
                callers: HashSet::new(),
                callees: HashSet::from([callee.to_string()]),
                last_updated: now_epoch,
            });

        // Callee -> incoming edge from caller
        self.graph
            .entry(callee.to_string())
            .and_modify(|n| {
                n.callers.insert(caller.to_string());
                n.last_updated = now_epoch;
            })
            .or_insert_with(|| CallGraphNode {
                symbol: callee.to_string(),
                callers: HashSet::from([caller.to_string()]),
                callees: HashSet::new(),
                last_updated: now_epoch,
            });
    }

    /// Retrieves all callers (functions that invoke `symbol`).
    #[must_use]
    pub fn get_callers(&self, symbol: &str) -> Vec<String> {
        self.graph
            .get(symbol)
            .map(|n| n.callers.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Retrieves all callees (functions invoked by `symbol`).
    #[must_use]
    pub fn get_callees(&self, symbol: &str) -> Vec<String> {
        self.graph
            .get(symbol)
            .map(|n| n.callees.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Total number of indexed symbols.
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Total number of distinct nodes in the call graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.graph.len()
    }

    /// Total directed edge count across the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let mut total: usize = 0;
        for kv in self.graph.iter() {
            total = total.saturating_add(kv.value().callers.len());
        }
        total
    }

    /// Clears all stored data (primarily for tests).
    pub fn clear(&self) {
        self.symbols.clear();
        self.graph.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_indexing_and_lookup() {
        let store = CallGraphStore::new();
        store.clear();

        let entry = SymbolEntry {
            qualified_name: "crate::inference::prober::probe".to_string(),
            kind: SymbolKind::Fn,
            file_path: PathBuf::from("crates/souls_inference_runtime/src/lib.rs"),
            line: 42,
            column: 4,
        };

        store.insert_symbol(entry.clone());
        let found = store.lookup_symbol("crate::inference::prober::probe");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), entry);
        assert_eq!(store.symbol_count(), 1);

        // Remove by file
        let removed = store.remove_symbols_for_file(&PathBuf::from("crates/souls_inference_runtime/src/lib.rs"));
        assert_eq!(removed, 1);
        assert_eq!(store.symbol_count(), 0);
    }

    #[test]
    fn test_call_graph_directed_edges() {
        let store = CallGraphStore::new();
        store.clear();

        // A calls B and C
        store.insert_edge("pkg::fn_a", "pkg::fn_b", 100);
        store.insert_edge("pkg::fn_a", "pkg::fn_c", 100);

        // Callees of A
        let callees_a = store.get_callees("pkg::fn_a");
        assert_eq!(callees_a.len(), 2);
        assert!(callees_a.contains(&"pkg::fn_b".to_string()));
        assert!(callees_a.contains(&"pkg::fn_c".to_string()));

        // Callers of B
        let callers_b = store.get_callers("pkg::fn_b");
        assert_eq!(callers_b.len(), 1);
        assert!(callers_b.contains(&"pkg::fn_a".to_string()));

        // Self recursion is ignored
        store.insert_edge("pkg::fn_a", "pkg::fn_a", 100);
        assert_eq!(store.get_callers("pkg::fn_a").len(), 0);
    }
}
