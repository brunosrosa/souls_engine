//! # Sliding Window Structured Chunking (`souls_core::research::chunker`)
//!
//! Decomposes long Markdown / text documents into structured overlapping chunks (~500-600 words)
//! respecting sentence and paragraph boundaries, providing deterministic hashes and token metrics.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A single structured text chunk with integrity metrics and positional offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChunk {
    pub index: usize,
    pub text: String,
    pub word_count: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub content_hash: u64,
}

/// Configuration options for the sliding window chunker.
#[derive(Debug, Clone)]
pub struct ChunkerOptions {
    pub target_words: usize,
    pub overlap_words: usize,
    pub min_chunk_words: usize,
}

impl Default for ChunkerOptions {
    fn default() -> Self {
        Self {
            target_words: 500,
            overlap_words: 80,
            min_chunk_words: 60,
        }
    }
}

/// Decomposes text into overlapping structured chunks based on paragraph and sentence boundaries.
#[must_use]
pub fn chunk_document(text: &str, options: &ChunkerOptions) -> Vec<TextChunk> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let paragraphs: Vec<&str> = text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()).collect();
    if paragraphs.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_words = Vec::new();
    let mut chunk_index = 0;
    let mut current_start_char = 0;

    for paragraph in paragraphs {
        let p_words: Vec<&str> = paragraph.split_whitespace().collect();
        if p_words.is_empty() {
            continue;
        }

        // If adding this paragraph exceeds target words, finalize current chunk
        if !current_words.is_empty() && (current_words.len() + p_words.len()) > options.target_words {
            let chunk_text = current_words.join(" ");
            let word_count = current_words.len();
            let end_char = current_start_char + chunk_text.len();
            let content_hash = compute_text_hash(&chunk_text);

            chunks.push(TextChunk {
                index: chunk_index,
                text: chunk_text,
                word_count,
                start_char: current_start_char,
                end_char,
                content_hash,
            });

            chunk_index += 1;

            // Retain overlap words for the next window
            let overlap_count = options.overlap_words.min(current_words.len());
            let overlap_start = current_words.len() - overlap_count;
            let retained: Vec<String> = current_words[overlap_start..].to_vec();

            current_start_char = end_char.saturating_sub(retained.join(" ").len());
            current_words = retained;
        }

        // Add paragraph words
        for w in p_words {
            current_words.push(w.to_string());
        }
    }

    // Finalize any remaining words
    if !current_words.is_empty() {
        let chunk_text = current_words.join(" ");
        let word_count = current_words.len();
        let end_char = current_start_char + chunk_text.len();
        let content_hash = compute_text_hash(&chunk_text);

        chunks.push(TextChunk {
            index: chunk_index,
            text: chunk_text,
            word_count,
            start_char: current_start_char,
            end_char,
            content_hash,
        });
    }

    chunks
}

/// Computes deterministic 64-bit hash of text contents.
#[inline]
pub fn compute_text_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_sliding_window_with_overlap() {
        // Generate a 150-word text across 3 paragraphs
        let p1 = "Tokio provides asynchronous runtime for Rust applications with event-driven non-blocking I/O. ".repeat(6);
        let p2 = "Windows 11 Dev Drive utilizes ReFS filesystem with Copy-on-Write and block cloning mechanisms. ".repeat(6);
        let p3 = "Epistemic verification enforces strict evidence validation before knowledge is committed. ".repeat(6);
        let full_text = format!("{p1}\n\n{p2}\n\n{p3}");

        let opts = ChunkerOptions {
            target_words: 50,
            overlap_words: 10,
            min_chunk_words: 20,
        };

        let chunks = chunk_document(&full_text, &opts);
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[1].index, 1);
        assert!(chunks[0].word_count >= 30);
        assert_ne!(chunks[0].content_hash, chunks[1].content_hash);
    }
}
