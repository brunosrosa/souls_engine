//! Epistemic Research Passage Cache (Triadic L3 Memory) for Souls Engine (v7/v0.8.0).
//!
//! Provides deterministic caching across SQLite STRICT (`epistemic_passages`),
//! LanceDB columnar dense vector search (`research_passages_vec`), and optional
//! LadybugDB ontological entity linking.
//!
//! FinOps & Resilience:
//! - Guarantees zero unnecessary web re-scraping and re-OCR of PDF documents.
//! - Stores bm25 scores, content hashes, and configurable TTL expirations.
//! - Operates with zero continuous VRAM (100% CPU/RAM Arrow memory model).

use std::sync::Arc;
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use futures_util::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Row, Sqlite};
use tracing::{debug, info};

use crate::error::MemoryError;
use crate::ladybug::{LadybugOntologyGraph, OntologicalEdge, OntologicalNode};
use crate::lancedb::{LanceDbVectorStore, VECTOR_DIMENSION};

/// Canonical LanceDB table name for research passages.
pub const RESEARCH_VECTORS_TABLE: &str = "research_passages_vec";

/// Default TTL for cached research passages: 7 days in seconds.
pub const DEFAULT_RESEARCH_CACHE_TTL_SECS: i64 = 7 * 86400;

/// Relational record representation stored in SQLite STRICT table `epistemic_passages`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpistemicPassageRecord {
    pub passage_hash: String,
    pub query_hash: String,
    pub raw_url: String,
    pub title: String,
    pub passage_text: String,
    pub bm25_score: f64,
    pub created_at: i64,
    pub expires_at: i64,
}

/// Semantic match candidate returned from LanceDB research vector index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchPassageMatch {
    pub passage_hash: String,
    pub query_hash: String,
    pub raw_url: String,
    pub title: String,
    pub passage_text: String,
    pub score: f32,
}

/// Returns the Apache Arrow schema for `research_passages_vec`.
#[must_use]
pub fn get_research_vector_table_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("passage_hash", DataType::Utf8, false),
        Field::new("query_hash", DataType::Utf8, false),
        Field::new("raw_url", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("passage_text", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                VECTOR_DIMENSION as i32,
            ),
            false,
        ),
    ]))
}

/// Computes a deterministic SHA-256 hash for a search query.
#[must_use]
pub fn compute_query_hash(query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.trim().to_lowercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Computes a deterministic SHA-256 hash for a passage based on its source URL and content.
#[must_use]
pub fn compute_passage_hash(raw_url: &str, passage_text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_url.trim().as_bytes());
    hasher.update(b":::");
    hasher.update(passage_text.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Inserts or updates an epistemic passage in SQLite STRICT table `epistemic_passages`.
pub async fn upsert_epistemic_passage(
    pool: &Pool<Sqlite>,
    record: &EpistemicPassageRecord,
) -> Result<(), MemoryError> {
    sqlx::query(
        r#"
        INSERT INTO epistemic_passages (
            passage_hash, query_hash, raw_url, title, passage_text, bm25_score, created_at, expires_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(passage_hash) DO UPDATE SET
            query_hash = excluded.query_hash,
            title = excluded.title,
            passage_text = excluded.passage_text,
            bm25_score = excluded.bm25_score,
            expires_at = excluded.expires_at
        "#,
    )
    .bind(&record.passage_hash)
    .bind(&record.query_hash)
    .bind(&record.raw_url)
    .bind(&record.title)
    .bind(&record.passage_text)
    .bind(record.bm25_score)
    .bind(record.created_at)
    .bind(record.expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves an epistemic passage by its unique deterministic `passage_hash`.
pub async fn get_epistemic_passage(
    pool: &Pool<Sqlite>,
    passage_hash: &str,
) -> Result<Option<EpistemicPassageRecord>, MemoryError> {
    let row = sqlx::query(
        r#"
        SELECT passage_hash, query_hash, raw_url, title, passage_text, bm25_score, created_at, expires_at
        FROM epistemic_passages
        WHERE passage_hash = ?1
        "#,
    )
    .bind(passage_hash)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        Ok(Some(EpistemicPassageRecord {
            passage_hash: r.get(0),
            query_hash: r.get(1),
            raw_url: r.get(2),
            title: r.get(3),
            passage_text: r.get(4),
            bm25_score: r.get(5),
            created_at: r.get(6),
            expires_at: r.get(7),
        }))
    } else {
        Ok(None)
    }
}

/// Retrieves all valid (non-expired) passages for a given `query_hash`.
pub async fn find_valid_passages_for_query(
    pool: &Pool<Sqlite>,
    query_hash: &str,
    now: i64,
) -> Result<Vec<EpistemicPassageRecord>, MemoryError> {
    let rows = sqlx::query(
        r#"
        SELECT passage_hash, query_hash, raw_url, title, passage_text, bm25_score, created_at, expires_at
        FROM epistemic_passages
        WHERE query_hash = ?1 AND expires_at > ?2
        ORDER BY bm25_score DESC
        "#,
    )
    .bind(query_hash)
    .bind(now)
    .fetch_all(pool)
    .await?;

    let mut list = Vec::with_capacity(rows.len());
    for r in rows {
        list.push(EpistemicPassageRecord {
            passage_hash: r.get(0),
            query_hash: r.get(1),
            raw_url: r.get(2),
            title: r.get(3),
            passage_text: r.get(4),
            bm25_score: r.get(5),
            created_at: r.get(6),
            expires_at: r.get(7),
        });
    }
    Ok(list)
}

/// Retrieves all valid (non-expired) passages for a given `raw_url`.
pub async fn find_valid_passages_for_url(
    pool: &Pool<Sqlite>,
    raw_url: &str,
    now: i64,
) -> Result<Vec<EpistemicPassageRecord>, MemoryError> {
    let rows = sqlx::query(
        r#"
        SELECT passage_hash, query_hash, raw_url, title, passage_text, bm25_score, created_at, expires_at
        FROM epistemic_passages
        WHERE raw_url = ?1 AND expires_at > ?2
        ORDER BY created_at DESC
        "#,
    )
    .bind(raw_url)
    .bind(now)
    .fetch_all(pool)
    .await?;

    let mut list = Vec::with_capacity(rows.len());
    for r in rows {
        list.push(EpistemicPassageRecord {
            passage_hash: r.get(0),
            query_hash: r.get(1),
            raw_url: r.get(2),
            title: r.get(3),
            passage_text: r.get(4),
            bm25_score: r.get(5),
            created_at: r.get(6),
            expires_at: r.get(7),
        });
    }
    Ok(list)
}

/// Purges all expired epistemic passages from the relational store.
pub async fn purge_expired_epistemic_passages(
    pool: &Pool<Sqlite>,
    now: i64,
) -> Result<u64, MemoryError> {
    let result = sqlx::query("DELETE FROM epistemic_passages WHERE expires_at <= ?1")
        .bind(now)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Opens or creates the `research_passages_vec` table in LanceDB.
pub async fn get_or_create_research_table(
    conn: &lancedb::connection::Connection,
) -> Result<lancedb::table::Table, MemoryError> {
    let table_names = conn
        .table_names()
        .execute()
        .await
        .map_err(|e| MemoryError::LanceDb(format!("Failed to list LanceDB tables: {e}")))?;

    if table_names.contains(&RESEARCH_VECTORS_TABLE.to_string()) {
        conn.open_table(RESEARCH_VECTORS_TABLE)
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to open table '{RESEARCH_VECTORS_TABLE}': {e}")))
    } else {
        let schema = get_research_vector_table_schema();
        conn.create_empty_table(RESEARCH_VECTORS_TABLE, schema)
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to create empty table '{RESEARCH_VECTORS_TABLE}': {e}")))
    }
}

/// Appends a passage embedding to the LanceDB `research_passages_vec` table.
pub async fn insert_research_passage_vector(
    lance_store: &LanceDbVectorStore,
    record: &EpistemicPassageRecord,
    embedding: Vec<f32>,
) -> Result<(), MemoryError> {
    if embedding.len() != VECTOR_DIMENSION {
        return Err(MemoryError::VectorDimensionMismatch {
            expected: VECTOR_DIMENSION,
            received: embedding.len(),
        });
    }

    let schema = get_research_vector_table_schema();

    let passage_hash_array = StringArray::from(vec![record.passage_hash.as_str()]);
    let query_hash_array = StringArray::from(vec![record.query_hash.as_str()]);
    let raw_url_array = StringArray::from(vec![record.raw_url.as_str()]);
    let title_array = StringArray::from(vec![record.title.as_str()]);
    let passage_text_array = StringArray::from(vec![record.passage_text.as_str()]);

    let values_array = Float32Array::from(embedding);
    let vector_array = FixedSizeListArray::new(
        Arc::new(Field::new("item", DataType::Float32, true)),
        VECTOR_DIMENSION as i32,
        Arc::new(values_array),
        None,
    );

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(passage_hash_array),
            Arc::new(query_hash_array),
            Arc::new(raw_url_array),
            Arc::new(title_array),
            Arc::new(passage_text_array),
            Arc::new(vector_array),
        ],
    )
    .map_err(|e| MemoryError::LanceDb(format!("Failed to build Arrow RecordBatch for research vector: {e}")))?;

    let conn = lance_store.connect_db().await?;
    let table = get_or_create_research_table(&conn).await?;

    table
        .add(vec![batch])
        .execute()
        .await
        .map_err(|e| MemoryError::LanceDb(format!("Failed to append batch to '{RESEARCH_VECTORS_TABLE}': {e}")))?;

    debug!(
        target: "souls_memory::research_cache",
        "Appended vector for passage '{}' ({})",
        record.passage_hash,
        record.title
    );

    Ok(())
}

/// Searches the LanceDB `research_passages_vec` table using cosine similarity.
pub async fn search_research_passage_vectors(
    lance_store: &LanceDbVectorStore,
    query_vector: Vec<f32>,
    limit: usize,
) -> Result<Vec<ResearchPassageMatch>, MemoryError> {
    if query_vector.len() != VECTOR_DIMENSION {
        return Err(MemoryError::VectorDimensionMismatch {
            expected: VECTOR_DIMENSION,
            received: query_vector.len(),
        });
    }

    let conn = lance_store.connect_db().await?;
    let table_names = conn
        .table_names()
        .execute()
        .await
        .map_err(|e| MemoryError::LanceDb(format!("Failed to query tables: {e}")))?;

    if !table_names.contains(&RESEARCH_VECTORS_TABLE.to_string()) {
        return Ok(Vec::new());
    }

    let table = conn
        .open_table(RESEARCH_VECTORS_TABLE)
        .execute()
        .await
        .map_err(|e| MemoryError::LanceDb(format!("Failed to open table: {e}")))?;

    let mut query = table
        .vector_search(query_vector)
        .map_err(|e| MemoryError::LanceDb(e.to_string()))?;
    query = query.limit(limit);

    let mut stream = query
        .execute()
        .await
        .map_err(|e| MemoryError::LanceDb(format!("LanceDB query execution failed: {e}")))?;

    let mut matches = Vec::new();

    while let Some(batch_result) = stream.next().await {
        let batch = batch_result
            .map_err(|e| MemoryError::LanceDb(format!("Stream error reading batch: {e}")))?;
        let num_rows = batch.num_rows();

        let passage_hash_col = batch
            .column_by_name("passage_hash")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| MemoryError::LanceDb("Column 'passage_hash' missing".to_string()))?;

        let query_hash_col = batch
            .column_by_name("query_hash")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| MemoryError::LanceDb("Column 'query_hash' missing".to_string()))?;

        let raw_url_col = batch
            .column_by_name("raw_url")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| MemoryError::LanceDb("Column 'raw_url' missing".to_string()))?;

        let title_col = batch
            .column_by_name("title")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| MemoryError::LanceDb("Column 'title' missing".to_string()))?;

        let passage_text_col = batch
            .column_by_name("passage_text")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| MemoryError::LanceDb("Column 'passage_text' missing".to_string()))?;

        let distance_col = batch
            .column_by_name("_distance")
            .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

        for i in 0..num_rows {
            let distance = distance_col.map(|d| d.value(i)).unwrap_or(0.0);
            let score = 1.0 / (1.0 + distance.max(0.0));

            matches.push(ResearchPassageMatch {
                passage_hash: passage_hash_col.value(i).to_string(),
                query_hash: query_hash_col.value(i).to_string(),
                raw_url: raw_url_col.value(i).to_string(),
                title: title_col.value(i).to_string(),
                passage_text: passage_text_col.value(i).to_string(),
                score,
            });
        }
    }

    Ok(matches)
}

/// High-level coordinator for the Triadic Epistemic Research Cache.
#[derive(Debug, Clone)]
pub struct EpistemicPassageCache {
    pool: Arc<Pool<Sqlite>>,
    lance_store: LanceDbVectorStore,
    default_ttl_secs: i64,
}

impl EpistemicPassageCache {
    /// Creates a new epistemic cache instance.
    pub fn new(
        pool: Arc<Pool<Sqlite>>,
        lance_store: LanceDbVectorStore,
        default_ttl_secs: Option<i64>,
    ) -> Self {
        Self {
            pool,
            lance_store,
            default_ttl_secs: default_ttl_secs.unwrap_or(DEFAULT_RESEARCH_CACHE_TTL_SECS),
        }
    }

    /// Accesses the underlying SQLite pool.
    #[must_use]
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Accesses the underlying LanceDB vector store.
    #[must_use]
    pub fn lance_store(&self) -> &LanceDbVectorStore {
        &self.lance_store
    }

    /// Default TTL configured in seconds.
    #[must_use]
    pub fn default_ttl_secs(&self) -> i64 {
        self.default_ttl_secs
    }

    /// Checks if a non-expired passage exists for a given exact URL.
    pub async fn lookup_by_url(
        &self,
        raw_url: &str,
        now: i64,
    ) -> Result<Option<EpistemicPassageRecord>, MemoryError> {
        let passages = find_valid_passages_for_url(&self.pool, raw_url, now).await?;
        Ok(passages.into_iter().next())
    }

    /// Checks if a non-expired passage exists by its content hash.
    pub async fn lookup_by_hash(
        &self,
        passage_hash: &str,
        now: i64,
    ) -> Result<Option<EpistemicPassageRecord>, MemoryError> {
        if let Some(record) = get_epistemic_passage(&self.pool, passage_hash).await? {
            if record.expires_at > now {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    /// Checks if valid passages exist for a given natural language search query.
    pub async fn lookup_by_query(
        &self,
        query: &str,
        now: i64,
    ) -> Result<Vec<EpistemicPassageRecord>, MemoryError> {
        let query_hash = compute_query_hash(query);
        find_valid_passages_for_query(&self.pool, &query_hash, now).await
    }

    /// Stores a passage into the relational SQLite cache with automatic hash generation.
    pub async fn cache_passage(
        &self,
        query: &str,
        raw_url: &str,
        title: &str,
        passage_text: &str,
        bm25_score: f64,
        now: i64,
        custom_ttl_secs: Option<i64>,
    ) -> Result<EpistemicPassageRecord, MemoryError> {
        let query_hash = compute_query_hash(query);
        let passage_hash = compute_passage_hash(raw_url, passage_text);
        let ttl = custom_ttl_secs.unwrap_or(self.default_ttl_secs);
        let expires_at = now + ttl;

        let record = EpistemicPassageRecord {
            passage_hash,
            query_hash,
            raw_url: raw_url.to_string(),
            title: title.to_string(),
            passage_text: passage_text.to_string(),
            bm25_score,
            created_at: now,
            expires_at,
        };

        upsert_epistemic_passage(&self.pool, &record).await?;
        info!(
            target: "souls_memory::research_cache",
            "Cached epistemic passage {} ('{}') for query '{}'",
            record.passage_hash,
            record.title,
            query
        );

        Ok(record)
    }

    /// Stores a passage and indexes its dense embedding into LanceDB.
    pub async fn cache_passage_with_vector(
        &self,
        query: &str,
        raw_url: &str,
        title: &str,
        passage_text: &str,
        bm25_score: f64,
        embedding: Vec<f32>,
        now: i64,
        custom_ttl_secs: Option<i64>,
    ) -> Result<EpistemicPassageRecord, MemoryError> {
        let record = self
            .cache_passage(
                query,
                raw_url,
                title,
                passage_text,
                bm25_score,
                now,
                custom_ttl_secs,
            )
            .await?;

        insert_research_passage_vector(&self.lance_store, &record, embedding).await?;
        Ok(record)
    }

    /// Executes semantic similarity search against indexed research vectors.
    pub async fn semantic_search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<ResearchPassageMatch>, MemoryError> {
        search_research_passage_vectors(&self.lance_store, query_vector, limit).await
    }

    /// Links an epistemic passage to a conceptual entity in LadybugDB causal graph.
    pub async fn link_to_ontology(
        &self,
        ladybug: &LadybugOntologyGraph,
        record: &EpistemicPassageRecord,
        entity_name: &str,
        now: i64,
    ) -> Result<(), MemoryError> {
        // Ensure entity node exists in Ladybug
        let entity_node = OntologicalNode {
            id: format!("entity_{entity_name}"),
            node_type: "RESEARCH_CONCEPT".to_string(),
            canonical_name: entity_name.to_string(),
            metadata_json: serde_json::json!({
                "origin": "research_passage",
                "passage_hash": record.passage_hash
            })
            .to_string(),
            banned_patterns: Vec::new(),
        };
        ladybug.insert_node(&self.pool, entity_node, now).await?;

        // Ensure passage node exists in Ladybug
        let passage_node = OntologicalNode {
            id: format!("passage_{}", record.passage_hash),
            node_type: "RESEARCH_PASSAGE".to_string(),
            canonical_name: record.title.clone(),
            metadata_json: serde_json::json!({
                "raw_url": record.raw_url,
                "bm25_score": record.bm25_score
            })
            .to_string(),
            banned_patterns: Vec::new(),
        };
        ladybug.insert_node(&self.pool, passage_node, now).await?;

        // Link with 'relates_to' relationship
        let edge = OntologicalEdge {
            source_id: format!("passage_{}", record.passage_hash),
            target_id: format!("entity_{entity_name}"),
            relationship: "relates_to".to_string(),
            weight: 1.0,
        };
        ladybug.insert_edge(&self.pool, edge, now).await?;

        Ok(())
    }

    /// Purges all expired passages across the relational database.
    pub async fn purge_expired(&self, now: i64) -> Result<u64, MemoryError> {
        purge_expired_epistemic_passages(&self.pool, now).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::create_in_memory_sqlite_pool;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_epistemic_passage_cache_relational_lifecycle() {
        let pool = Arc::new(create_in_memory_sqlite_pool().await.unwrap());
        let dir = tempdir().unwrap();
        let lance_store = LanceDbVectorStore::new(dir.path().join("test_lance.lance"));
        let cache = EpistemicPassageCache::new(pool.clone(), lance_store, Some(3600));

        let now = 1_000_000;
        let query = "qwen 3.5 architecture and benchmarks";
        let url = "https://example.com/qwen-report.pdf";
        let title = "Qwen 3.5 Architecture Deep Dive";
        let text = "Qwen 3.5 demonstrates exceptional agentic reasoning and native code execution.";

        // 1. Initial lookup should be None
        let miss = cache.lookup_by_url(url, now).await.unwrap();
        assert!(miss.is_none());

        // 2. Cache the passage
        let cached = cache
            .cache_passage(query, url, title, text, 12.85, now, None)
            .await
            .unwrap();
        assert_eq!(cached.raw_url, url);
        assert_eq!(cached.expires_at, now + 3600);

        // 3. Lookup by URL succeeds
        let hit_url = cache.lookup_by_url(url, now + 100).await.unwrap();
        assert!(hit_url.is_some());
        let record = hit_url.unwrap();
        assert_eq!(record.title, title);
        assert_eq!(record.passage_text, text);

        // 4. Lookup by query hash succeeds
        let query_hits = cache.lookup_by_query(query, now + 100).await.unwrap();
        assert_eq!(query_hits.len(), 1);
        assert_eq!(query_hits[0].passage_hash, cached.passage_hash);

        // 5. Lookup after expiration fails
        let expired_hit = cache.lookup_by_url(url, now + 4000).await.unwrap();
        assert!(expired_hit.is_none());

        // 6. Purge expired cleans up the table
        let purged = cache.purge_expired(now + 4000).await.unwrap();
        assert_eq!(purged, 1);

        let final_check = get_epistemic_passage(&pool, &cached.passage_hash)
            .await
            .unwrap();
        assert!(final_check.is_none());
    }

    #[tokio::test]
    async fn test_epistemic_passage_lancedb_vector_integration() {
        let pool = Arc::new(create_in_memory_sqlite_pool().await.unwrap());
        let dir = tempdir().unwrap();
        let lance_store = LanceDbVectorStore::new(dir.path().join("test_lance_vec.lance"));
        let cache = EpistemicPassageCache::new(pool.clone(), lance_store, Some(3600));

        let now = 1_000_000;
        let query = "agentic coding cli benchmarks";
        let url = "https://example.com/agentic_cli.pdf";
        let title = "Agentic CLI Benchmarks 2026";
        let text = "Claude Code and Antigravity CLI comparison on Dev Drive ReFS.";

        let mut synthetic_vec = vec![0.0f32; VECTOR_DIMENSION];
        synthetic_vec[0] = 1.0;

        let record = cache
            .cache_passage_with_vector(query, url, title, text, 8.5, synthetic_vec.clone(), now, None)
            .await
            .unwrap();
        assert_eq!(record.title, title);

        // Query vector nearest neighbor
        let results = cache.semantic_search(synthetic_vec, 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].passage_hash, record.passage_hash);
        assert_eq!(results[0].title, title);
        assert!(results[0].score > 0.9);
    }
}

