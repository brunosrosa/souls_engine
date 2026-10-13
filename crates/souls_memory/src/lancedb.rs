//! LanceDB Vector Store Engine (`lancedb.rs`) for Souls Engine (v7).
//!
//! Manages dense vector embedding persistence and cosine similarity search
//! over observation nodes (384 dimensions, fast local embedding) in Dev Drive ReFS.
//!
//! Bare-Metal Principles:
//! - Operates via direct `mmap` I/O at `Z:\souls_engine\.souls_data\souls_vectors.lance`.
//! - Zero continuous VRAM consumption: 100% CPU/RAM Arrow memory model (ADR-027).
//! - Cleanly integrates with RRF (Reciprocal Rank Fusion) hybrid recall.
//!
//! Conforms to CRATES_SPECIFICATION.md Section 2.4.

use crate::error::MemoryError;
use crate::rrf::VectorCandidate;
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use futures_util::StreamExt;
use lancedb::connect;
use lancedb::connection::Connection;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::table::Table;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default embedding vector dimension for fast local embeddings (bge-small-en-v1.5 / MiniLM).
pub const VECTOR_DIMENSION: usize = 384;

/// Canonical table name in the LanceDB database.
pub const TABLE_NAME: &str = "observations_vector";

/// Returns the Apache Arrow schema for `observations_vector`.
#[must_use]
pub fn get_vector_table_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("observation_id", DataType::Utf8, false),
        Field::new("entity_name", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("temporal_stability", DataType::Utf8, false),
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

/// Dedicated LanceDB vector store operating with 0 MB of VRAM.
#[derive(Debug, Clone)]
pub struct LanceDbVectorStore {
    db_path: PathBuf,
}

impl LanceDbVectorStore {
    /// Creates a vector store instance anchored to the specified filesystem path.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
        }
    }

    /// Returns the canonical Dev Drive ReFS store path.
    pub fn default_store() -> Self {
        Self::new(PathBuf::from(r"Z:\souls_engine\.souls_data\souls_vectors.lance"))
    }

    /// Path to the LanceDB directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Connects to LanceDB in embedded serverless mode.
    pub async fn connect_db(&self) -> Result<Connection, MemoryError> {
        let path_str = self.db_path.to_string_lossy().to_string();
        connect(&path_str)
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to connect to LanceDB at '{path_str}': {e}")))
    }

    /// Opens or initializes the canonical table in LanceDB.
    pub async fn get_or_create_table(&self, conn: &Connection) -> Result<Table, MemoryError> {
        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to list LanceDB tables: {e}")))?;

        if table_names.contains(&TABLE_NAME.to_string()) {
            conn.open_table(TABLE_NAME)
                .execute()
                .await
                .map_err(|e| MemoryError::LanceDb(format!("Failed to open table '{TABLE_NAME}': {e}")))
        } else {
            let schema = get_vector_table_schema();
            conn.create_empty_table(TABLE_NAME, schema)
                .execute()
                .await
                .map_err(|e| MemoryError::LanceDb(format!("Failed to create empty table '{TABLE_NAME}': {e}")))
        }
    }

    /// Inserts a memory observation record with its 384-dimensional vector embedding.
    pub async fn insert_vector(
        &self,
        observation_id: &str,
        entity_name: &str,
        content: &str,
        stability: &str,
        embedding: Vec<f32>,
    ) -> Result<(), MemoryError> {
        if embedding.len() != VECTOR_DIMENSION {
            return Err(MemoryError::VectorDimensionMismatch {
                expected: VECTOR_DIMENSION,
                received: embedding.len(),
            });
        }

        let schema = get_vector_table_schema();

        // Construct Apache Arrow RecordBatch columns
        let id_array = StringArray::from(vec![observation_id]);
        let entity_array = StringArray::from(vec![entity_name]);
        let content_array = StringArray::from(vec![content]);
        let stability_array = StringArray::from(vec![stability]);

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
                Arc::new(id_array),
                Arc::new(entity_array),
                Arc::new(content_array),
                Arc::new(stability_array),
                Arc::new(vector_array),
            ],
        )
        .map_err(|e| MemoryError::LanceDb(format!("Failed to build Arrow RecordBatch: {e}")))?;

        let conn = self.connect_db().await?;
        let table = self.get_or_create_table(&conn).await?;

        table
            .add(vec![batch])
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to append batch to LanceDB: {e}")))?;

        tracing::debug!(
            target: "souls_memory::lancedb",
            "Appended vector for observation '{}' ({})",
            observation_id,
            entity_name
        );

        Ok(())
    }

    /// Executes approximate vector nearest neighbor search using cosine similarity.
    /// Returns standardized `VectorCandidate` structs ready for RRF fusion.
    pub async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filter_stability: Option<String>,
    ) -> Result<Vec<VectorCandidate>, MemoryError> {
        if query_vector.len() != VECTOR_DIMENSION {
            return Err(MemoryError::VectorDimensionMismatch {
                expected: VECTOR_DIMENSION,
                received: query_vector.len(),
            });
        }

        let conn = self.connect_db().await?;
        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to query tables: {e}")))?;

        if !table_names.contains(&TABLE_NAME.to_string()) {
            return Ok(Vec::new());
        }

        let table = conn
            .open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("Failed to open table: {e}")))?;

        let mut query = table
            .vector_search(query_vector)
            .map_err(|e| MemoryError::LanceDb(e.to_string()))?;

        query = query.limit(limit);

        if let Some(stability) = filter_stability {
            let filter_expr = format!("temporal_stability = '{}'", stability.replace('\'', "''"));
            query = query.only_if(filter_expr);
        }

        let mut stream = query
            .execute()
            .await
            .map_err(|e| MemoryError::LanceDb(format!("LanceDB query execution failed: {e}")))?;

        let mut candidates = Vec::new();

        while let Some(batch_result) = stream.next().await {
            let batch = batch_result
                .map_err(|e| MemoryError::LanceDb(format!("Stream error reading batch: {e}")))?;
            let num_rows = batch.num_rows();

            let id_col = batch
                .column_by_name("observation_id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| MemoryError::LanceDb("Column 'observation_id' missing".to_string()))?;

            let entity_col = batch
                .column_by_name("entity_name")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| MemoryError::LanceDb("Column 'entity_name' missing".to_string()))?;

            let content_col = batch
                .column_by_name("content")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| MemoryError::LanceDb("Column 'content' missing".to_string()))?;

            let distance_col = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

            for i in 0..num_rows {
                let distance = distance_col.map(|d| d.value(i) as f64).unwrap_or(0.0);
                // Convert distance to cosine similarity score in [0.0..1.0]
                let similarity = 1.0 / (1.0 + distance.max(0.0));

                candidates.push(VectorCandidate {
                    id: id_col.value(i).to_string(),
                    content: content_col.value(i).to_string(),
                    category: entity_col.value(i).to_string(),
                    similarity,
                });
            }
        }

        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_lancedb_vector_store_crud() {
        let dir = tempdir().expect("Failed to create tempdir");
        let db_path = dir.path().join("test_vectors.lance");
        let store = LanceDbVectorStore::new(&db_path);

        let obs_id = "obs-001-guid";
        let entity = "ARCH_DECISION";
        let content = "ADR-027: Zero VRAM LanceDB vector store operating on host CPU/RAM.";
        let stability = "STABLE";

        // Deterministic synthetic 384-dimensional unit vector
        let mut synthetic_vec = vec![0.0f32; VECTOR_DIMENSION];
        synthetic_vec[0] = 1.0;

        // Insert
        store
            .insert_vector(obs_id, entity, content, stability, synthetic_vec.clone())
            .await
            .expect("Failed to insert vector into LanceDB");

        // Search
        let results = store
            .search_vectors(synthetic_vec, 5, None)
            .await
            .expect("Failed to search vectors in LanceDB");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, obs_id);
        assert_eq!(results[0].category, entity);
        assert_eq!(results[0].content, content);
        assert!(results[0].similarity > 0.5);
    }

    #[tokio::test]
    async fn test_lancedb_vector_dimension_mismatch() {
        let dir = tempdir().expect("Failed to create tempdir");
        let db_path = dir.path().join("test_dim_mismatch.lance");
        let store = LanceDbVectorStore::new(&db_path);

        let invalid_vec = vec![0.5f32; 128]; // Wrong dimension
        let err = store
            .insert_vector("id", "entity", "content", "STABLE", invalid_vec)
            .await;

        assert!(err.is_err());
        match err.unwrap_err() {
            MemoryError::VectorDimensionMismatch { expected, received } => {
                assert_eq!(expected, 384);
                assert_eq!(received, 128);
            }
            other => panic!("Expected VectorDimensionMismatch, got: {:?}", other),
        }
    }
}
