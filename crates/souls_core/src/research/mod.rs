//! # Tier 0 Deterministic Zero-LLM Research Engine (`souls_core::research`)
//!
//! High-throughput deterministic pipeline designed for Deep Research subagents (Scouts & Verifiers):
//! 1. In-memory zero-allocation HTML to Markdown extraction (`clean_html_to_markdown`).
//! 2. Canonical URL normalization and SipHash deduplication (`ResearchDedupRegistry`).
//! 3. Sliding window structured chunking (~500-600 words) (`chunk_document`).
//! 4. In-memory Okapi BM25 lexical passage ranking (`rank_chunks`).

pub mod bm25;
pub mod chunker;
pub mod dedup;
pub mod html_cleaner;

pub use bm25::{rank_chunks, tokenize_lexical, ScoredChunk, DEFAULT_BM25_B, DEFAULT_BM25_K1};
pub use chunker::{chunk_document, compute_text_hash, ChunkerOptions, TextChunk};
pub use dedup::{
    compute_content_fingerprint, hash_url, normalize_url, ResearchDedupRegistry,
};
pub use html_cleaner::{clean_html_to_markdown, HtmlCleanOptions};

/// Comprehensive summary result from the deterministic research filter pipeline.
#[derive(Debug, Clone)]
pub struct FilteredPassageResult {
    pub canonical_url: String,
    pub is_duplicate: bool,
    pub original_bytes: usize,
    pub clean_markdown_bytes: usize,
    pub total_chunks: usize,
    pub top_passages: Vec<FilteredPassage>,
    pub compression_ratio: f64,
}

/// A selected high-relevance passage ready for Scout consumption.
#[derive(Debug, Clone)]
pub struct FilteredPassage {
    pub chunk_index: usize,
    pub text: String,
    pub word_count: usize,
    pub bm25_score: f64,
}

/// End-to-end deterministic processing pipeline: HTML -> Clean MD -> Dedup Check -> Chunking -> BM25 Ranking.
#[must_use]
pub fn process_raw_html_passage(
    raw_html: &str,
    raw_url: &str,
    query: &str,
    registry: &mut ResearchDedupRegistry,
    top_k: usize,
) -> FilteredPassageResult {
    let canonical_url = normalize_url(raw_url);
    let original_bytes = raw_html.len();

    // 1. Deduplication check
    let is_dup = !registry.register_url(raw_url);
    if is_dup {
        return FilteredPassageResult {
            canonical_url,
            is_duplicate: true,
            original_bytes,
            clean_markdown_bytes: 0,
            total_chunks: 0,
            top_passages: Vec::new(),
            compression_ratio: 0.0,
        };
    }

    // 2. HTML to Clean Markdown
    let clean_md = clean_html_to_markdown(raw_html, &HtmlCleanOptions::default());
    let clean_markdown_bytes = clean_md.len();

    // 3. Content fingerprint deduplication check
    let is_content_dup = !registry.register_content(&clean_md);
    if is_content_dup {
        return FilteredPassageResult {
            canonical_url,
            is_duplicate: true,
            original_bytes,
            clean_markdown_bytes,
            total_chunks: 0,
            top_passages: Vec::new(),
            compression_ratio: 0.0,
        };
    }

    // 4. Sliding Window Chunking
    let chunks = chunk_document(&clean_md, &ChunkerOptions::default());
    let total_chunks = chunks.len();

    // 5. Okapi BM25 Lexical Ranking against Query
    let scored = rank_chunks(&chunks, query, top_k, DEFAULT_BM25_K1, DEFAULT_BM25_B);

    let top_passages: Vec<FilteredPassage> = scored
        .into_iter()
        .map(|s| FilteredPassage {
            chunk_index: s.chunk.index,
            text: s.chunk.text.clone(),
            word_count: s.chunk.word_count,
            bm25_score: s.score,
        })
        .collect();

    let compression_ratio = if original_bytes > 0 {
        1.0 - (clean_markdown_bytes as f64 / original_bytes as f64)
    } else {
        0.0
    };

    FilteredPassageResult {
        canonical_url,
        is_duplicate: false,
        original_bytes,
        clean_markdown_bytes,
        total_chunks,
        top_passages,
        compression_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_deterministic_filter_pipeline() {
        let mut reg = ResearchDedupRegistry::new();
        let raw_html = r##"
            <html>
                <head><script>alert("ignore");</script></head>
                <body>
                    <nav><a href="#">Menu</a></nav>
                    <h1>Benchmark ReFS vs NTFS no Windows 11</h1>
                    <p>O ReFS com Block Cloning demonstrou 45.000 IOPS com latência inferior a 1.2ms.</p>
                    <p>Em contrapartida, sistemas de arquivos legados apresentam degradação em operações intensivas.</p>
                    <footer>Rodapé com termos de uso</footer>
                </body>
            </html>
        "##;

        let query = "ReFS Block Cloning IOPS";
        let res = process_raw_html_passage(
            raw_html,
            "https://tech.example.com/refs-benchmarks?utm_source=rss",
            query,
            &mut reg,
            1,
        );

        assert!(!res.is_duplicate);
        assert_eq!(res.canonical_url, "tech.example.com/refs-benchmarks");
        assert!(res.clean_markdown_bytes < res.original_bytes);
        assert!(res.compression_ratio > 0.3);
        assert!(!res.top_passages.is_empty());
        assert!(res.top_passages[0].text.contains("Block Cloning"));

        // Second ingestion with same URL should report duplicate
        let res_dup = process_raw_html_passage(
            raw_html,
            "http://www.tech.example.com/refs-benchmarks#section",
            query,
            &mut reg,
            1,
        );
        assert!(res_dup.is_duplicate);
    }
}
