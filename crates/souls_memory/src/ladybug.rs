//! LadybugDB: In-memory ontological causal graph (`petgraph`) synchronized
//! with SQLite STRICT WAL (`ladybug_nodes`, `ladybug_edges`) and ON DELETE CASCADE.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use dashmap::DashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tokio::sync::RwLock;
use tracing::info;

use crate::error::MemoryError;

/// Ontological node representation stored in RAM and mirrored to `ladybug_nodes`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OntologicalNode {
    pub id: String,
    pub node_type: String,
    pub canonical_name: String,
    pub metadata_json: String,
    pub banned_patterns: Vec<String>,
}

/// Directed edge representing dependency or constraint relationship.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OntologicalEdge {
    pub source_id: String,
    pub target_id: String,
    pub relationship: String,
    pub weight: f32,
}

/// In-memory ontological graph engine with transactional SQLite synchronization.
#[derive(Debug, Clone)]
pub struct LadybugOntologyGraph {
    graph: Arc<RwLock<DiGraph<OntologicalNode, OntologicalEdge>>>,
    index_map: Arc<DashMap<String, NodeIndex>>,
}

impl Default for LadybugOntologyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl LadybugOntologyGraph {
    /// Instantiates an empty in-memory ontology graph.
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(DiGraph::new())),
            index_map: Arc::new(DashMap::new()),
        }
    }

    /// Rehydrates the in-memory graph from FrankenSQLite in cold-boot (<20ms).
    /// Offloads JSON parsing and petgraph assembly to tokio::task::spawn_blocking to keep the reactor unblocked.
    pub async fn load_from_db(&self, pool: &Pool<Sqlite>) -> Result<usize, MemoryError> {
        // 1. Fetch raw node records asynchronously
        let node_rows = sqlx::query(
            "SELECT id, node_type, canonical_name, metadata_json FROM ladybug_nodes",
        )
        .fetch_all(pool)
        .await?;

        let raw_nodes: Vec<(String, String, String, String)> = node_rows
            .into_iter()
            .map(|row| (row.get(0), row.get(1), row.get(2), row.get(3)))
            .collect();

        // 2. Fetch raw edge records asynchronously
        let edge_rows = sqlx::query(
            "SELECT source_id, target_id, relationship, weight FROM ladybug_edges",
        )
        .fetch_all(pool)
        .await?;

        let raw_edges: Vec<(String, String, String, f64)> = edge_rows
            .into_iter()
            .map(|row| (row.get(0), row.get(1), row.get(2), row.get(3)))
            .collect();

        // 3. Offload compute-heavy deserialization and graph assembly to blocking threadpool
        let (new_graph, new_index_map, loaded_edges) = tokio::task::spawn_blocking(move || {
            let mut g: DiGraph<OntologicalNode, OntologicalEdge> = DiGraph::new();
            let mut index_map: std::collections::HashMap<String, NodeIndex> =
                std::collections::HashMap::new();

            for (id, node_type, canonical_name, metadata_json) in raw_nodes {
                let banned = serde_json::from_str::<serde_json::Value>(&metadata_json)
                    .ok()
                    .and_then(|v| v.get("banned_patterns").cloned())
                    .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
                    .unwrap_or_default();

                let node = OntologicalNode {
                    id: id.clone(),
                    node_type,
                    canonical_name,
                    metadata_json,
                    banned_patterns: banned,
                };

                let idx = g.add_node(node);
                index_map.insert(id, idx);
            }

            let mut loaded_edges = 0;
            for (source_id, target_id, relationship, weight) in raw_edges {
                if let (Some(&src_idx), Some(&dst_idx)) = (
                    index_map.get(&source_id),
                    index_map.get(&target_id),
                ) {
                    let edge = OntologicalEdge {
                        source_id,
                        target_id,
                        relationship,
                        weight: weight as f32,
                    };
                    g.add_edge(src_idx, dst_idx, edge);
                    loaded_edges += 1;
                }
            }

            (g, index_map, loaded_edges)
        })
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // 4. Atomically swap graph and index_map under write lock
        let mut g = self.graph.write().await;
        *g = new_graph;
        self.index_map.clear();
        for (id, idx) in new_index_map {
            self.index_map.insert(id, idx);
        }

        info!(
            "LadybugDB rehydrated: {} nodes, {} edges loaded into RAM.",
            self.index_map.len(),
            loaded_edges
        );

        Ok(self.index_map.len())
    }

    /// Inserts a node into both in-memory RAM and the SQLite STRICT table `ladybug_nodes`.
    pub async fn insert_node(
        &self,
        pool: &Pool<Sqlite>,
        node: OntologicalNode,
        now_epoch: i64,
    ) -> Result<(), MemoryError> {
        // Persist to SQLite STRICT
        sqlx::query(
            r#"
            INSERT INTO ladybug_nodes (id, node_type, canonical_name, metadata_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                node_type = excluded.node_type,
                canonical_name = excluded.canonical_name,
                metadata_json = excluded.metadata_json
            "#,
        )
        .bind(&node.id)
        .bind(&node.node_type)
        .bind(&node.canonical_name)
        .bind(&node.metadata_json)
        .bind(now_epoch)
        .execute(pool)
        .await?;

        // Update in-memory graph
        let mut g = self.graph.write().await;
        if let Some(existing_idx) = self.index_map.get(&node.id) {
            g[*existing_idx] = node;
        } else {
            let id = node.id.clone();
            let idx = g.add_node(node);
            self.index_map.insert(id, idx);
        }

        Ok(())
    }

    /// Inserts a directed edge into both in-memory RAM and SQLite `ladybug_edges`.
    pub async fn insert_edge(
        &self,
        pool: &Pool<Sqlite>,
        edge: OntologicalEdge,
        now_epoch: i64,
    ) -> Result<(), MemoryError> {
        let src_idx = *self
            .index_map
            .get(&edge.source_id)
            .ok_or_else(|| MemoryError::NodeNotFound(edge.source_id.clone()))?;

        let dst_idx = *self
            .index_map
            .get(&edge.target_id)
            .ok_or_else(|| MemoryError::NodeNotFound(edge.target_id.clone()))?;

        // Persist to SQLite STRICT with foreign keys enabled
        sqlx::query(
            r#"
            INSERT INTO ladybug_edges (source_id, target_id, relationship, weight, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(source_id, target_id, relationship) DO UPDATE SET
                weight = excluded.weight
            "#,
        )
        .bind(&edge.source_id)
        .bind(&edge.target_id)
        .bind(&edge.relationship)
        .bind(edge.weight as f64)
        .bind(now_epoch)
        .execute(pool)
        .await?;

        // Add to petgraph RAM
        let mut g = self.graph.write().await;
        g.add_edge(src_idx, dst_idx, edge);

        Ok(())
    }

    /// Deletes a node. Automatically triggers SQLite `ON DELETE CASCADE` for all orphan edges
    /// and synchronizes the in-memory RAM graph.
    pub async fn delete_node(&self, pool: &Pool<Sqlite>, id: &str) -> Result<(), MemoryError> {
        // 1. Delete from SQLite (Triggers ON DELETE CASCADE on ladybug_edges)
        sqlx::query("DELETE FROM ladybug_nodes WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?;

        // 2. Remove from petgraph and index map
        if let Some((_, node_idx)) = self.index_map.remove(id) {
            let mut g = self.graph.write().await;
            g.remove_node(node_idx);

            // Rebuild index map since petgraph shifts indices on node removal
            self.index_map.clear();
            for idx in g.node_indices() {
                self.index_map.insert(g[idx].id.clone(), idx);
            }
        }

        Ok(())
    }

    /// Computes the Blast Radius (all downstream affected nodes) starting from `start_node_id` via BFS.
    pub async fn calculate_blast_radius(
        &self,
        start_node_id: &str,
        max_depth: usize,
    ) -> Result<Vec<String>, MemoryError> {
        let start_idx = *self
            .index_map
            .get(start_node_id)
            .ok_or_else(|| MemoryError::NodeNotFound(start_node_id.to_string()))?;

        let g = self.graph.read().await;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut blast_radius = Vec::new();

        visited.insert(start_idx);
        queue.push_back((start_idx, 0));

        while let Some((curr_idx, depth)) = queue.pop_front() {
            if depth > 0 {
                blast_radius.push(g[curr_idx].id.clone());
            }

            if depth >= max_depth {
                continue;
            }

            // Traverse outgoing edges (Direction::Outgoing)
            for neighbor in g.neighbors_directed(curr_idx, Direction::Outgoing) {
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        Ok(blast_radius)
    }

    /// Ontological Firewall: BFS scan verifying whether candidate content violates
    /// banned patterns across reachable nodes.
    pub async fn check_ontology_compliance(
        &self,
        start_node_id: &str,
        candidate_content: &str,
        max_depth: usize,
    ) -> Result<(), MemoryError> {
        let start_idx = *self
            .index_map
            .get(start_node_id)
            .ok_or_else(|| MemoryError::NodeNotFound(start_node_id.to_string()))?;

        let g = self.graph.read().await;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(start_idx);
        queue.push_back((start_idx, 0));

        while let Some((curr_idx, depth)) = queue.pop_front() {
            let node = &g[curr_idx];
            for banned in &node.banned_patterns {
                if candidate_content.contains(banned) {
                    return Err(MemoryError::OntologyViolation {
                        reason: format!(
                            "Candidate content contains forbidden pattern '{}' from node '{}'",
                            banned, node.id
                        ),
                        violated_node: node.id.clone(),
                    });
                }
            }

            if depth >= max_depth {
                continue;
            }

            for neighbor in g.neighbors_directed(curr_idx, Direction::Outgoing) {
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::create_in_memory_sqlite_pool;

    #[tokio::test]
    async fn test_ladybug_blast_radius_and_cascade_delete() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();
        let graph = LadybugOntologyGraph::new();

        // 1. Populate dependency topology:
        // Root (A) -> Service (B) -> DB (C)
        // Root (A) -> Worker (D)
        let node_a = OntologicalNode {
            id: "node_a".to_string(),
            node_type: "Module".to_string(),
            canonical_name: "Root".to_string(),
            metadata_json: r#"{"banned_patterns": ["legacy_v1"]}"#.to_string(),
            banned_patterns: vec!["legacy_v1".to_string()],
        };
        let node_b = OntologicalNode {
            id: "node_b".to_string(),
            node_type: "Service".to_string(),
            canonical_name: "WorkerB".to_string(),
            metadata_json: "{}".to_string(),
            banned_patterns: vec![],
        };
        let node_c = OntologicalNode {
            id: "node_c".to_string(),
            node_type: "Storage".to_string(),
            canonical_name: "DatabaseC".to_string(),
            metadata_json: "{}".to_string(),
            banned_patterns: vec![],
        };
        let node_d = OntologicalNode {
            id: "node_d".to_string(),
            node_type: "Worker".to_string(),
            canonical_name: "AsyncWorkerD".to_string(),
            metadata_json: "{}".to_string(),
            banned_patterns: vec![],
        };

        graph.insert_node(&pool, node_a, 100).await.unwrap();
        graph.insert_node(&pool, node_b, 100).await.unwrap();
        graph.insert_node(&pool, node_c, 100).await.unwrap();
        graph.insert_node(&pool, node_d, 100).await.unwrap();

        // Edges
        graph
            .insert_edge(
                &pool,
                OntologicalEdge {
                    source_id: "node_a".to_string(),
                    target_id: "node_b".to_string(),
                    relationship: "depends_on".to_string(),
                    weight: 1.0,
                },
                100,
            )
            .await
            .unwrap();

        graph
            .insert_edge(
                &pool,
                OntologicalEdge {
                    source_id: "node_b".to_string(),
                    target_id: "node_c".to_string(),
                    relationship: "depends_on".to_string(),
                    weight: 1.0,
                },
                100,
            )
            .await
            .unwrap();

        graph
            .insert_edge(
                &pool,
                OntologicalEdge {
                    source_id: "node_a".to_string(),
                    target_id: "node_d".to_string(),
                    relationship: "depends_on".to_string(),
                    weight: 0.8,
                },
                100,
            )
            .await
            .unwrap();

        // 2. Blast Radius from Root (A) should include B, C, D
        let blast = graph.calculate_blast_radius("node_a", 3).await.unwrap();
        assert_eq!(blast.len(), 3);
        assert!(blast.contains(&"node_b".to_string()));
        assert!(blast.contains(&"node_c".to_string()));
        assert!(blast.contains(&"node_d".to_string()));

        // 3. Ontological Firewall check
        let compliant = graph
            .check_ontology_compliance("node_a", "valid modern code", 3)
            .await;
        assert!(compliant.is_ok());

        let poisoned = graph
            .check_ontology_compliance("node_a", "import legacy_v1", 3)
            .await;
        assert!(poisoned.is_err());

        // 4. Verify SQLite edge count before deletion (3 edges)
        let pre_edge_count: i64 = sqlx::query("SELECT COUNT(*) FROM ladybug_edges")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
        assert_eq!(pre_edge_count, 3);

        // 5. Delete Root Node A -> ON DELETE CASCADE must purge edges (A->B) and (A->D)
        graph.delete_node(&pool, "node_a").await.unwrap();

        // Edge count should now be 1 (only B->C remains)
        let post_edge_count: i64 = sqlx::query("SELECT COUNT(*) FROM ladybug_edges")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            post_edge_count, 1,
            "SQLite ON DELETE CASCADE must clean orphan edges automatically"
        );

        // Verify remaining edge is indeed B->C
        let remaining_edge_src: String =
            sqlx::query("SELECT source_id FROM ladybug_edges LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
        assert_eq!(remaining_edge_src, "node_b");
    }

    #[tokio::test]
    async fn test_ladybug_load_from_db_spawn_blocking() {
        let pool = create_in_memory_sqlite_pool().await.unwrap();
        let graph1 = LadybugOntologyGraph::new();

        let node1 = OntologicalNode {
            id: "node_x".to_string(),
            node_type: "Component".to_string(),
            canonical_name: "CompX".to_string(),
            metadata_json: r#"{"banned_patterns": ["bad_pattern_1", "bad_pattern_2"]}"#.to_string(),
            banned_patterns: vec!["bad_pattern_1".to_string(), "bad_pattern_2".to_string()],
        };
        let node2 = OntologicalNode {
            id: "node_y".to_string(),
            node_type: "Service".to_string(),
            canonical_name: "ServY".to_string(),
            metadata_json: "{}".to_string(),
            banned_patterns: vec![],
        };

        graph1.insert_node(&pool, node1, 100).await.unwrap();
        graph1.insert_node(&pool, node2, 100).await.unwrap();
        graph1
            .insert_edge(
                &pool,
                OntologicalEdge {
                    source_id: "node_x".to_string(),
                    target_id: "node_y".to_string(),
                    relationship: "depends_on".to_string(),
                    weight: 0.95,
                },
                100,
            )
            .await
            .unwrap();

        // Fresh graph instance rehydrating from SQLite
        let graph2 = LadybugOntologyGraph::new();
        let loaded_nodes = graph2.load_from_db(&pool).await.unwrap();
        assert_eq!(loaded_nodes, 2);

        // Verify loaded nodes and firewall banned patterns were restored
        let firewall_res = graph2
            .check_ontology_compliance("node_x", "testing bad_pattern_1 here", 2)
            .await;
        assert!(firewall_res.is_err());

        let blast = graph2.calculate_blast_radius("node_x", 1).await.unwrap();
        assert_eq!(blast, vec!["node_y".to_string()]);
    }
}
