//! Comprehensive Integration Tests for Triadic Epistemic Passage Cache (`souls_memory`).
//!
//! Validates:
//! - Exact-match URL resolution and SHA-256 query lookup.
//! - Non-blocking TTL-based expiration and bulk pruning.
//! - LanceDB zero-VRAM columnar vector indexing and cosine retrieval.
//! - LadybugDB ontological causal graph entity linking.

use std::sync::Arc;
use souls_memory::{
    create_in_memory_sqlite_pool, EpistemicPassageCache, LadybugOntologyGraph, LanceDbVectorStore,
    VECTOR_DIMENSION,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_triadic_cache_full_pipeline() {
    let pool = Arc::new(create_in_memory_sqlite_pool().await.unwrap());
    let dir = tempdir().unwrap();
    let lance_path = dir.path().join("test_triadic.lance");
    let lance_store = LanceDbVectorStore::new(lance_path);

    let cache = EpistemicPassageCache::new(pool.clone(), lance_store, Some(3600));
    let ladybug = LadybugOntologyGraph::new();

    let now = 1_700_000_000;
    let query = "Rust 2024 edition async traits";
    let url = "https://rust-lang.org/edition-2024.pdf";
    let title = "Rust Edition 2024 Official Spec";
    let passage = "Async fn in traits and return position impl trait in traits are stabilized natively.";

    // 1. Initial lookups return None
    assert!(cache.lookup_by_url(url, now).await.unwrap().is_none());
    assert!(cache.lookup_by_query(query, now).await.unwrap().is_empty());

    // 2. Cache with 384-dimensional synthetic vector
    let mut vec_384 = vec![0.0f32; VECTOR_DIMENSION];
    vec_384[0] = 0.95;
    vec_384[1] = 0.31;

    let cached = cache
        .cache_passage_with_vector(
            query,
            url,
            title,
            passage,
            15.42,
            vec_384.clone(),
            now,
            Some(7200), // 2 hours
        )
        .await
        .unwrap();

    assert_eq!(cached.raw_url, url);
    assert_eq!(cached.expires_at, now + 7200);

    // 3. Fast resolution: URL lookup
    let hit_url = cache.lookup_by_url(url, now + 500).await.unwrap().unwrap();
    assert_eq!(hit_url.passage_hash, cached.passage_hash);
    assert_eq!(hit_url.title, title);
    assert_eq!(hit_url.passage_text, passage);

    // 4. Fast resolution: Query lookup
    let query_hits = cache.lookup_by_query(query, now + 1000).await.unwrap();
    assert_eq!(query_hits.len(), 1);
    assert_eq!(query_hits[0].passage_hash, cached.passage_hash);

    // 5. LanceDB Semantic Search
    let matches = cache.semantic_search(vec_384, 5).await.unwrap();
    assert!(!matches.is_empty());
    assert_eq!(matches[0].passage_hash, cached.passage_hash);
    assert!(matches[0].score > 0.95);

    // 6. LadybugDB Ontological Entity Link
    cache
        .link_to_ontology(&ladybug, &cached, "AsyncTraits", now)
        .await
        .unwrap();

    // Verify entity in Ladybug
    let neighbors = ladybug
        .get_neighbors(&format!("passage_{}", cached.passage_hash))
        .await;
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].target_id, "entity_AsyncTraits");
    assert_eq!(neighbors[0].relationship, "relates_to");

    // 7. Expiration boundary test
    assert!(cache.lookup_by_url(url, now + 7201).await.unwrap().is_none());

    // 8. Bulk Purge
    let purged = cache.purge_expired(now + 8000).await.unwrap();
    assert_eq!(purged, 1);
}

