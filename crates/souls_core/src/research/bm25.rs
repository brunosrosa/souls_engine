//! # Bare-Metal Lexical BM25 Ranker (`souls_core::research::bm25`)
//!
//! High-efficiency Okapi BM25 implementation running in-memory on host CPU with AVX2 friendliness.
//! Ranks passages and text chunks against research questions and subtopics without neural embeddings.

use std::collections::HashMap;
use super::chunker::TextChunk;

/// Standard Okapi BM25 parameters.
pub const DEFAULT_BM25_K1: f64 = 1.2;
pub const DEFAULT_BM25_B: f64 = 0.75;

/// Common multilingual stop words ignored during BM25 lexical tokenization.
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "of", "to", "in", "is", "for", "on", "with", "as", "by",
    "at", "from", "it", "this", "that", "which", "are", "was", "were", "be", "been",
    // Portuguese common functional stop words
    "o", "os", "um", "uma", "uns", "umas", "de", "do", "da", "dos", "das", "em", "no", "na",
    "nos", "nas", "para", "por", "com", "que", "se", "como", "mas", "ao", "aos", "e",
];

/// A text chunk paired with its computed BM25 lexical relevance score.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredChunk<'a> {
    pub chunk: &'a TextChunk,
    pub score: f64,
}

/// Tokenizes text into normalized lowercase alphanumeric terms, omitting short tokens and stop words.
#[must_use]
pub fn tokenize_lexical(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let lower = text.to_lowercase();

    for raw in lower.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
        let t = raw.trim();
        if t.len() > 1 && !STOP_WORDS.contains(&t) {
            tokens.push(t.to_string());
        }
    }
    tokens
}

/// Ranks a collection of `TextChunk`s against a query using Okapi BM25.
///
/// Returns top-k matching chunks ordered by descending relevance score.
#[must_use]
pub fn rank_chunks<'a>(
    chunks: &'a [TextChunk],
    query: &str,
    top_k: usize,
    k1: f64,
    b: f64,
) -> Vec<ScoredChunk<'a>> {
    if chunks.is_empty() || query.trim().is_empty() {
        return Vec::new();
    }

    let query_terms = tokenize_lexical(query);
    if query_terms.is_empty() {
        return Vec::new();
    }

    let n_docs = chunks.len() as f64;
    let mut doc_tokens = Vec::with_capacity(chunks.len());
    let mut total_doc_len = 0;

    // 1. Tokenize all documents and build term frequencies
    for chunk in chunks {
        let tokens = tokenize_lexical(&chunk.text);
        total_doc_len += tokens.len();
        doc_tokens.push(tokens);
    }

    let avg_doc_len = if n_docs > 0.0 {
        (total_doc_len as f64) / n_docs
    } else {
        1.0
    };

    // 2. Compute Document Frequency (DF) for each query term
    let mut df: HashMap<String, usize> = HashMap::new();
    for term in &query_terms {
        let mut count = 0;
        for doc in &doc_tokens {
            if doc.iter().any(|t| t == term) {
                count += 1;
            }
        }
        df.insert(term.clone(), count);
    }

    // 3. Score each document against query terms
    let mut scored = Vec::with_capacity(chunks.len());

    for (i, doc) in doc_tokens.iter().enumerate() {
        let doc_len = doc.len() as f64;
        let mut tf_map: HashMap<&str, usize> = HashMap::new();
        for t in doc {
            *tf_map.entry(t.as_str()).or_insert(0) += 1;
        }

        let mut doc_score = 0.0;

        for term in &query_terms {
            let doc_freq = *df.get(term).unwrap_or(&0) as f64;
            if doc_freq == 0.0 {
                continue;
            }

            // Robertson-Spärck Jones IDF with smoothing
            let idf = ((n_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();

            let tf = *tf_map.get(term.as_str()).unwrap_or(&0) as f64;
            if tf > 0.0 {
                let numerator = tf * (k1 + 1.0);
                let denominator = tf + k1 * (1.0 - b + b * (doc_len / avg_doc_len));
                doc_score += idf * (numerator / denominator);
            }
        }

        if doc_score > 0.0 {
            scored.push(ScoredChunk {
                chunk: &chunks[i],
                score: doc_score,
            });
        }
    }

    // 4. Sort descending by score
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    if top_k > 0 && scored.len() > top_k {
        scored.truncate(top_k);
    }

    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::chunker::compute_text_hash;

    #[test]
    fn test_bm25_lexical_ranking_prefers_exact_matches() {
        let chunk1 = TextChunk {
            index: 0,
            text: "Receitas de bolo de cenoura com cobertura de chocolate e glacê cremoso.".to_string(),
            word_count: 12,
            start_char: 0,
            end_char: 70,
            content_hash: compute_text_hash("chunk1"),
        };

        let chunk2 = TextChunk {
            index: 1,
            text: "O Tokio runtime em Rust suporta IOCP nativo no Windows 11 com threads worker dedicadas.".to_string(),
            word_count: 15,
            start_char: 71,
            end_char: 160,
            content_hash: compute_text_hash("chunk2"),
        };

        let chunk3 = TextChunk {
            index: 2,
            text: "Linux epoll e kqueue no macOS oferecem mecanismos assíncronos alternativos ao IOCP.".to_string(),
            word_count: 13,
            start_char: 161,
            end_char: 245,
            content_hash: compute_text_hash("chunk3"),
        };

        let chunks = vec![chunk1, chunk2, chunk3];
        let query = "Tokio IOCP Windows";

        let results = rank_chunks(&chunks, query, 2, DEFAULT_BM25_K1, DEFAULT_BM25_B);
        assert!(!results.is_empty());
        assert_eq!(results[0].chunk.index, 1); // Chunk 2 must rank highest!
        assert!(results[0].score > 0.0);
    }
}
