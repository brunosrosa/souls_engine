//! LEAN context compression and deduplication data types for Souls Engine (v7).
//!
//! Provides memory-efficient structures for syntactic dehydration,
//! lossless deduplication caches, and context headroom metrics.
//!
//! Pure types, zero I/O side effects, zero unsafe code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of syntactic compression applied to source code or documents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeanCompressedContent {
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub compression_ratio: f32,
    pub content: String,
}

impl LeanCompressedContent {
    #[must_use]
    pub fn new(original_bytes: usize, content: String) -> Self {
        let compressed_bytes = content.len();
        let compression_ratio = if original_bytes > 0 {
            compressed_bytes as f32 / original_bytes as f32
        } else {
            1.0
        };

        Self {
            original_bytes,
            compressed_bytes,
            compression_ratio,
            content,
        }
    }
}

/// A deduplicated chunk identified across multiple files or context windows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanDedupChunk {
    pub chunk_hash: String,
    pub line_count: usize,
    pub occurrences: usize,
    pub token_savings: usize,
}

/// Map of placeholder keys to original dehydrated content for lossless rehydration.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LeanFillMap {
    pub placeholders: HashMap<String, String>,
}

impl LeanFillMap {
    #[must_use]
    pub fn new() -> Self {
        Self {
            placeholders: HashMap::new(),
        }
    }

    pub fn insert(&mut self, placeholder: impl Into<String>, content: impl Into<String>) {
        self.placeholders.insert(placeholder.into(), content.into());
    }

    #[must_use]
    pub fn get(&self, placeholder: &str) -> Option<&str> {
        self.placeholders.get(placeholder).map(|s| s.as_str())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.placeholders.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.placeholders.is_empty()
    }
}

/// Context headroom metrics tracking prompt consumption against model window limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeanHeadroomMetrics {
    pub current_tokens: usize,
    pub max_context_window: usize,
    pub remaining_headroom: usize,
    pub headroom_percentage: f32,
}

impl LeanHeadroomMetrics {
    #[must_use]
    pub fn new(current_tokens: usize, max_context_window: usize) -> Self {
        let remaining_headroom = max_context_window.saturating_sub(current_tokens);
        let headroom_percentage = if max_context_window > 0 {
            (remaining_headroom as f32 / max_context_window as f32) * 100.0
        } else {
            0.0
        };

        Self {
            current_tokens,
            max_context_window,
            remaining_headroom,
            headroom_percentage,
        }
    }
}

/// Context summary emitted during LEAN prompt compaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeanContextSummary {
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub tokens_saved: usize,
    pub active_preset: String,
    pub headroom: LeanHeadroomMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lean_compressed_content_ratio() {
        let content = "fn main() {}".to_string();
        let compressed = LeanCompressedContent::new(100, content.clone());
        assert_eq!(compressed.original_bytes, 100);
        assert_eq!(compressed.compressed_bytes, content.len());
        assert!((compressed.compression_ratio - (content.len() as f32 / 100.0)).abs() < 1e-6);
    }

    #[test]
    fn test_lean_fill_map_operations() {
        let mut map = LeanFillMap::new();
        assert!(map.is_empty());
        map.insert("__SOULS_CHUNK_1__", "large block of code");
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("__SOULS_CHUNK_1__"), Some("large block of code"));
        assert_eq!(map.get("__NON_EXISTENT__"), None);
    }

    #[test]
    fn test_lean_headroom_metrics_calculation() {
        let metrics = LeanHeadroomMetrics::new(2000, 8000);
        assert_eq!(metrics.remaining_headroom, 6000);
        assert!((metrics.headroom_percentage - 75.0).abs() < 1e-4);

        let saturated = LeanHeadroomMetrics::new(9000, 8000);
        assert_eq!(saturated.remaining_headroom, 0);
        assert_eq!(saturated.headroom_percentage, 0.0);
    }
}
