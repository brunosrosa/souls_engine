//! Tier 0 MinionS corpus preparation & SHA-256 disk caching for Google documents,
//! PDFs, Gmail threads, and Forms summaries.
//!
//! Reuses `souls_core::research::{pdf_extractor, html_cleaner, chunker, bm25}` to shrink
//! large corpora into compact BM25-ranked passages before optional local SLM synthesis.

use super::security::wrap_untrusted_content;
use super::session::write_json_atomic_lf;
use crate::error::CoreError;
use crate::research::bm25::{rank_chunks, DEFAULT_BM25_B, DEFAULT_BM25_K1};
use crate::research::chunker::{chunk_document, compute_text_hash, ChunkerOptions, TextChunk};
use crate::research::html_cleaner::{clean_html_to_markdown, HtmlCleanOptions};
use crate::research::pdf_extractor::{extract_pdf_bytes, PdfExtractOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use souls_protocol::google::{MinionGoogleSourceKind, MinionPassageHitDto};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAX_CACHE_FILES: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableChunk {
    index: usize,
    text: String,
    word_count: usize,
    start_char: usize,
    end_char: usize,
    content_hash: u64,
}

impl From<&TextChunk> for SerializableChunk {
    fn from(c: &TextChunk) -> Self {
        Self {
            index: c.index,
            text: c.text.clone(),
            word_count: c.word_count,
            start_char: c.start_char,
            end_char: c.end_char,
            content_hash: c.content_hash,
        }
    }
}

impl From<SerializableChunk> for TextChunk {
    fn from(c: SerializableChunk) -> Self {
        Self {
            index: c.index,
            text: c.text,
            word_count: c.word_count,
            start_char: c.start_char,
            end_char: c.end_char,
            content_hash: c.content_hash,
        }
    }
}

/// Persisted Tier 0 chunk cache entry (`<data_dir>/minions_cache/<safe_id>_<short_sha>.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCorpusChunks {
    pub source_kind: MinionGoogleSourceKind,
    pub source_ref: String,
    pub content_sha256: String,
    pub normalized_text: String,
    pub raw_chars: usize,
    chunks: Vec<SerializableChunk>,
}

/// Output of Tier 0 deterministic corpus preparation and BM25 passage selection.
#[derive(Debug, Clone)]
pub struct Tier0MinionPrep {
    pub source_kind: MinionGoogleSourceKind,
    pub source_ref: String,
    pub query: String,
    pub cache_hit: bool,
    pub content_sha256: String,
    pub raw_chars: usize,
    pub retained_chars: usize,
    pub estimated_token_savings_pct: f32,
    pub chunks_total: usize,
    pub chunks_selected: usize,
    pub top_passages: Vec<MinionPassageHitDto>,
    pub framed_context_for_slm: String,
    pub factual_claims: Vec<Value>,
    pub statistical_data: Vec<Value>,
    pub direct_quotes: Vec<String>,
    pub executive_summary: String,
    pub markdown_export: String,
}

/// Extracts, normalizes, chunks, caches, and BM25-ranks a Google corpus offline at Tier 0.
pub fn prepare_google_corpus_tier0(
    data_dir: &Path,
    source_kind: MinionGoogleSourceKind,
    source_ref: &str,
    raw_bytes: &[u8],
    query: &str,
    top_k: usize,
) -> Result<Tier0MinionPrep, CoreError> {
    if raw_bytes.is_empty() {
        return Err(CoreError::WebExtractionError(
            "Cannot prepare empty Google corpus".to_string(),
        ));
    }

    let content_sha256 = hash_corpus_bytes(source_kind, raw_bytes);
    let short_sha = &content_sha256[..16];
    let safe_id = sanitize_cache_id(source_ref);

    let cache_dir = data_dir.join("minions_cache");
    let cache_path = cache_dir.join(format!("{safe_id}_{short_sha}.json"));

    let (normalized_text, raw_chars, chunks, cache_hit) = if cache_path.exists() {
        if let Ok(raw_cache) = fs::read_to_string(&cache_path) {
            if let Ok(cached) = serde_json::from_str::<CachedCorpusChunks>(&raw_cache) {
                let loaded_chunks: Vec<TextChunk> =
                    cached.chunks.into_iter().map(TextChunk::from).collect();
                (
                    cached.normalized_text,
                    cached.raw_chars,
                    loaded_chunks,
                    true,
                )
            } else {
                extract_and_cache(
                    &cache_dir,
                    &cache_path,
                    source_kind,
                    source_ref,
                    &content_sha256,
                    raw_bytes,
                )?
            }
        } else {
            extract_and_cache(
                &cache_dir,
                &cache_path,
                source_kind,
                source_ref,
                &content_sha256,
                raw_bytes,
            )?
        }
    } else {
        extract_and_cache(
            &cache_dir,
            &cache_path,
            source_kind,
            source_ref,
            &content_sha256,
            raw_bytes,
        )?
    };

    let k = top_k.max(1);
    let scored = rank_chunks(&chunks, query, k, DEFAULT_BM25_K1, DEFAULT_BM25_B);

    let mut top_passages: Vec<MinionPassageHitDto> = if !scored.is_empty() {
        scored
            .into_iter()
            .map(|s| MinionPassageHitDto {
                chunk_index: s.chunk.index,
                score: ((s.score * 1000.0).round() / 1000.0) as f32,
                citation: format!("chunk#{}", s.chunk.index),
                excerpt: s.chunk.text.clone(),
            })
            .collect()
    } else {
        chunks
            .iter()
            .take(k)
            .map(|c| MinionPassageHitDto {
                chunk_index: c.index,
                score: 0.0,
                citation: format!("chunk#{}", c.index),
                excerpt: c.text.clone(),
            })
            .collect()
    };

    if top_passages.is_empty() && !normalized_text.trim().is_empty() {
        top_passages.push(MinionPassageHitDto {
            chunk_index: 0,
            score: 0.0,
            citation: "chunk#0".to_string(),
            excerpt: normalized_text.trim().to_string(),
        });
    }

    let joined_passages: String = top_passages
        .iter()
        .map(|p| format!("[{}] {}", p.citation, p.excerpt))
        .collect::<Vec<_>>()
        .join("\n\n");

    let retained_chars: usize = top_passages.iter().map(|p| p.excerpt.chars().count()).sum();
    let estimated_token_savings_pct = if raw_chars > 0 && raw_chars > retained_chars {
        let ratio = ((raw_chars - retained_chars) as f32 / raw_chars as f32) * 100.0;
        (ratio * 10.0).round() / 10.0
    } else {
        0.0
    };

    let framed_context_for_slm =
        wrap_untrusted_content(&format!("minion:{source_ref}"), &joined_passages);

    let (factual_claims, statistical_data, direct_quotes) = extract_structured_signals(&top_passages);
    let executive_summary = build_executive_summary(source_ref, query, &top_passages, estimated_token_savings_pct);
    let markdown_export = build_markdown_export(
        source_kind,
        source_ref,
        query,
        cache_hit,
        raw_chars,
        retained_chars,
        estimated_token_savings_pct,
        &executive_summary,
        &top_passages,
    );

    Ok(Tier0MinionPrep {
        source_kind,
        source_ref: source_ref.to_string(),
        query: query.to_string(),
        cache_hit,
        content_sha256,
        raw_chars,
        retained_chars,
        estimated_token_savings_pct,
        chunks_total: chunks.len(),
        chunks_selected: top_passages.len(),
        top_passages,
        framed_context_for_slm,
        factual_claims,
        statistical_data,
        direct_quotes,
        executive_summary,
        markdown_export,
    })
}

fn extract_and_cache(
    cache_dir: &Path,
    cache_path: &Path,
    source_kind: MinionGoogleSourceKind,
    source_ref: &str,
    content_sha256: &str,
    raw_bytes: &[u8],
) -> Result<(String, usize, Vec<TextChunk>, bool), CoreError> {
    let normalized_text = match source_kind {
        MinionGoogleSourceKind::DrivePdf => {
            let pdf = extract_pdf_bytes(raw_bytes, &PdfExtractOptions::default()).map_err(|e| {
                CoreError::WebExtractionError(format!("PDF extraction failed for '{source_ref}': {e}"))
            })?;
            pdf.full_markdown.replace("\r\n", "\n")
        }
        _ => {
            let raw_str = String::from_utf8_lossy(raw_bytes).into_owned();
            if looks_like_html(&raw_str) {
                clean_html_to_markdown(&raw_str, &HtmlCleanOptions::default()).replace("\r\n", "\n")
            } else {
                raw_str.replace("\r\n", "\n")
            }
        }
    };

    let raw_chars = raw_bytes.len().max(normalized_text.chars().count());
    let chunks = chunk_google_corpus(&normalized_text);

    fs::create_dir_all(cache_dir).map_err(|e| {
        CoreError::SubprocessError(format!(
            "Failed creating minions_cache dir '{}': {e}",
            cache_dir.display()
        ))
    })?;

    let cached_payload = CachedCorpusChunks {
        source_kind,
        source_ref: source_ref.to_string(),
        content_sha256: content_sha256.to_string(),
        normalized_text: normalized_text.clone(),
        raw_chars,
        chunks: chunks.iter().map(SerializableChunk::from).collect(),
    };
    write_json_atomic_lf(cache_path, &cached_payload)?;
    prune_cache_dir(cache_dir, MAX_CACHE_FILES);

    Ok((normalized_text, raw_chars, chunks, false))
}

fn chunk_google_corpus(text: &str) -> Vec<TextChunk> {
    const MAX_PARAGRAPH_CHUNK_WORDS: usize = 350;
    const SUB_WINDOW_WORDS: usize = 300;
    const SUB_OVERLAP_WORDS: usize = 50;

    let base_chunks = chunk_document(text, &ChunkerOptions::default());
    let mut expanded = Vec::with_capacity(base_chunks.len());

    for chunk in base_chunks {
        if chunk.word_count <= MAX_PARAGRAPH_CHUNK_WORDS {
            expanded.push(chunk);
            continue;
        }

        let words: Vec<&str> = chunk.text.split_whitespace().collect();
        let step = SUB_WINDOW_WORDS.saturating_sub(SUB_OVERLAP_WORDS).max(1);
        let mut cursor_words = 0usize;
        let mut char_offset = chunk.start_char;

        while cursor_words < words.len() {
            let end_words = (cursor_words + SUB_WINDOW_WORDS).min(words.len());
            let sub_text = words[cursor_words..end_words].join(" ");
            let sub_word_count = end_words - cursor_words;
            let sub_len = sub_text.len();

            expanded.push(TextChunk {
                index: expanded.len(),
                word_count: sub_word_count,
                start_char: char_offset,
                end_char: char_offset + sub_len,
                content_hash: compute_text_hash(&sub_text),
                text: sub_text,
            });

            if end_words >= words.len() {
                break;
            }
            let consumed = words[cursor_words..cursor_words + step].join(" ");
            char_offset = char_offset.saturating_add(consumed.len() + 1);
            cursor_words += step;
        }
    }

    for (idx, chunk) in expanded.iter_mut().enumerate() {
        chunk.index = idx;
    }

    expanded
}

fn hash_corpus_bytes(kind: MinionGoogleSourceKind, raw_bytes: &[u8]) -> String {
    let tag = match kind {
        MinionGoogleSourceKind::DriveDoc => "drive_doc",
        MinionGoogleSourceKind::DrivePdf => "drive_pdf",
        MinionGoogleSourceKind::GmailThread => "gmail_thread",
        MinionGoogleSourceKind::GoogleForm => "google_form",
        MinionGoogleSourceKind::RawText => "raw_text",
    };
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    hasher.update(b":");
    hasher.update(raw_bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

fn sanitize_cache_id(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
        if out.len() >= 32 {
            break;
        }
    }
    let clean = out.trim_matches('_').to_string();
    if clean.is_empty() {
        "corpus".to_string()
    } else {
        clean
    }
}

fn looks_like_html(text: &str) -> bool {
    let sample: String = text.chars().take(1024).collect();
    let lower = sample.to_ascii_lowercase();
    lower.contains("<html")
        || lower.contains("<body")
        || lower.contains("<div")
        || lower.contains("<p>")
        || lower.contains("<table")
}

fn prune_cache_dir(cache_dir: &Path, max_files: usize) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    let mut files: Vec<(SystemTime, PathBuf)> = entries
        .filter_map(|res| res.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                return None;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            Some((modified, path))
        })
        .collect();

    if files.len() <= max_files {
        return;
    }

    files.sort_by_key(|(ts, _)| *ts);
    let remove_count = files.len() - max_files;
    for (_, path) in files.into_iter().take(remove_count) {
        let _ = fs::remove_file(path);
    }
}

fn extract_structured_signals(
    passages: &[MinionPassageHitDto],
) -> (Vec<Value>, Vec<Value>, Vec<String>) {
    let mut factual_claims = Vec::new();
    let mut statistical_data = Vec::new();
    let mut direct_quotes = Vec::new();

    for passage in passages {
        for raw_sentence in passage.excerpt.split(['.', '\n']) {
            let sentence = raw_sentence.trim();
            if sentence.len() < 16 {
                continue;
            }
            if direct_quotes.len() < 5 {
                direct_quotes.push(format!("[{}] {}", passage.citation, sentence));
            }
            let has_digit = sentence.bytes().any(|b| b.is_ascii_digit());
            if has_digit && statistical_data.len() < 8 {
                statistical_data.push(json!({
                    "citation": passage.citation,
                    "statement": sentence,
                }));
            } else if factual_claims.len() < 8 {
                factual_claims.push(json!({
                    "citation": passage.citation,
                    "claim": sentence,
                }));
            }
        }
    }

    (factual_claims, statistical_data, direct_quotes)
}

fn build_executive_summary(
    source_ref: &str,
    query: &str,
    passages: &[MinionPassageHitDto],
    savings_pct: f32,
) -> String {
    let Some(top) = passages.first() else {
        return format!("Nenhum trecho relevante encontrado em '{source_ref}' para '{query}'.");
    };
    let preview: String = top.excerpt.chars().take(240).collect();
    format!(
        "Análise econômica Tier 0 de '{source_ref}' para \"{query}\" selecionou {} trecho(s) relevante(s) (economia estimada de {:.1}% de tokens). Destaque [{}]: {}",
        passages.len(),
        savings_pct,
        top.citation,
        preview.replace('\n', " ")
    )
}

#[allow(clippy::too_many_arguments)]
fn build_markdown_export(
    source_kind: MinionGoogleSourceKind,
    source_ref: &str,
    query: &str,
    cache_hit: bool,
    raw_chars: usize,
    retained_chars: usize,
    savings_pct: f32,
    summary: &str,
    passages: &[MinionPassageHitDto],
) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Análise Econômica — `{source_ref}`\n\n"));
    md.push_str(&format!("- **Origem:** {:?}\n", source_kind));
    md.push_str(&format!("- **Consulta:** {query}\n"));
    md.push_str(&format!("- **Cache local:** {}\n", if cache_hit { "hit" } else { "miss" }));
    md.push_str(&format!(
        "- **Compressão:** {raw_chars} -> {retained_chars} caracteres ({savings_pct:.1}% de economia)\n\n"
    ));
    md.push_str("## Resumo Executivo\n\n");
    md.push_str(summary);
    md.push_str("\n\n## Trechos Selecionados\n\n");

    for p in passages {
        md.push_str(&format!("### {} (score {:.3})\n\n{}\n\n", p.citation, p.score, p.excerpt));
    }

    md.replace("\r\n", "\n")
}
