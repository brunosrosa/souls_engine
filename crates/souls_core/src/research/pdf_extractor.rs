//! # Tier 0 Deterministic PDF Extractor (`souls_core::research::pdf_extractor`)
//!
//! Native zero-C-bindings PDF text and tabular extraction:
//! 1. Streaming page-by-page extraction (`lopdf`).
//! 2. Semantic page demarcation (`<!-- Page X of Y -->`).
//! 3. Heuristic tabular layout detection converted directly into Markdown tables (`| col1 | col2 |`).
//! 4. Integration into the STORM Research pipeline (`process_raw_pdf_passage`).

use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use crate::research::chunker::chunk_document;
use crate::research::dedup::{normalize_url, ResearchDedupRegistry};
use crate::research::bm25::rank_chunks;
use crate::research::{FilteredPassage, FilteredPassageResult};

/// Errors encountered during deterministic PDF processing.
#[derive(Debug, Error, Clone)]
pub enum PdfExtractError {
    #[error("PDF document is empty (0 bytes)")]
    EmptyDocument,

    #[error("Failed to parse PDF document structure: {0}")]
    ParseFailure(String),

    #[error("Document is encrypted with unsupported password protection")]
    EncryptedDocument,

    #[error("Failed to extract text from page {page}: {error}")]
    PageExtractionError { page: u32, error: String },

    #[error("I/O error reading PDF file: {0}")]
    IoError(String),
}

/// Options configuring deterministic PDF extraction.
#[derive(Debug, Clone)]
pub struct PdfExtractOptions {
    /// Whether to insert semantic page break comments `<!-- Page X of Y -->`. Default: true.
    pub demarcate_pages: bool,
    /// Whether to detect columnar/tabular lines and convert them to Markdown tables. Default: true.
    pub detect_tables: bool,
    /// Minimum whitespace gap between tokens to treat as separate table columns. Default: 2.
    pub min_col_gap_spaces: usize,
    /// Minimum consecutive rows required to form a table. Default: 2.
    pub min_table_rows: usize,
    /// Maximum number of pages to process (None = entire document). Default: None.
    pub max_pages: Option<usize>,
}

impl Default for PdfExtractOptions {
    fn default() -> Self {
        Self {
            demarcate_pages: true,
            detect_tables: true,
            min_col_gap_spaces: 2,
            min_table_rows: 2,
            max_pages: None,
        }
    }
}

/// Metadata extracted from the PDF trailer and info dictionary.
#[derive(Debug, Clone, Default)]
pub struct PdfDocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub creator: Option<String>,
    pub creation_date: Option<String>,
    pub producer: Option<String>,
}

/// Extracted content of an individual PDF page.
#[derive(Debug, Clone)]
pub struct PdfPageExtract {
    pub page_number: u32,
    pub raw_text: String,
    pub markdown_text: String,
    pub table_count: usize,
    pub word_count: usize,
}

/// Consolidated extraction result for an entire PDF document.
#[derive(Debug, Clone)]
pub struct PdfExtractResult {
    pub metadata: PdfDocumentMetadata,
    pub total_pages: usize,
    pub processed_pages: usize,
    pub pages: Vec<PdfPageExtract>,
    pub full_markdown: String,
    pub total_words: usize,
    pub total_tables: usize,
    pub original_bytes: usize,
}

/// Extracts structured Markdown and tabular content from raw PDF bytes.
pub fn extract_pdf_bytes(
    pdf_bytes: &[u8],
    options: &PdfExtractOptions,
) -> Result<PdfExtractResult, PdfExtractError> {
    if pdf_bytes.is_empty() {
        return Err(PdfExtractError::EmptyDocument);
    }

    let original_bytes = pdf_bytes.len();
    let doc = lopdf::Document::load_mem(pdf_bytes)
        .map_err(|e| PdfExtractError::ParseFailure(e.to_string()))?;

    // Extract metadata from info dictionary if present
    let mut metadata = PdfDocumentMetadata::default();
    if let Ok(info_id) = doc.trailer.get(b"Info").and_then(|obj| obj.as_reference()) {
        if let Ok(info_dict) = doc.get_object(info_id).and_then(|obj| obj.as_dict()) {
            metadata.title = info_dict.get(b"Title").ok().and_then(clean_pdf_string);
            metadata.author = info_dict.get(b"Author").ok().and_then(clean_pdf_string);
            metadata.creator = info_dict.get(b"Creator").ok().and_then(clean_pdf_string);
            metadata.producer = info_dict.get(b"Producer").ok().and_then(clean_pdf_string);
            metadata.creation_date = info_dict.get(b"CreationDate").ok().and_then(clean_pdf_string);
        }
    }

    let pages_map: BTreeMap<u32, lopdf::ObjectId> = doc.get_pages();
    let total_pages = pages_map.len();

    let mut pages = Vec::with_capacity(total_pages);
    let mut full_markdown = String::new();
    let mut total_words = 0usize;
    let mut total_tables = 0usize;

    let page_numbers: Vec<u32> = pages_map.keys().copied().collect();
    let pages_to_process = match options.max_pages {
        Some(max) => page_numbers.into_iter().take(max).collect(),
        None => page_numbers,
    };

    for page_num in &pages_to_process {
        let raw_text = match doc.extract_text(&[*page_num]) {
            Ok(txt) => txt,
            Err(e) => {
                // If single page fails to extract, record error string and continue resiliently
                format!("[Page {} Extraction Notice: {}]", page_num, e)
            }
        };

        let (page_md, table_count) = process_page_text_to_markdown(&raw_text, *page_num, total_pages, options);
        let word_count = page_md.split_whitespace().count();

        total_words += word_count;
        total_tables += table_count;

        if !full_markdown.is_empty() {
            full_markdown.push_str("\n\n");
        }
        full_markdown.push_str(&page_md);

        pages.push(PdfPageExtract {
            page_number: *page_num,
            raw_text,
            markdown_text: page_md,
            table_count,
            word_count,
        });
    }

    Ok(PdfExtractResult {
        metadata,
        total_pages,
        processed_pages: pages.len(),
        pages,
        full_markdown,
        total_words,
        total_tables,
        original_bytes,
    })
}

/// Extracts PDF content directly from host storage.
pub fn extract_pdf_file<P: AsRef<Path>>(
    path: P,
    options: &PdfExtractOptions,
) -> Result<PdfExtractResult, PdfExtractError> {
    let p = path.as_ref();
    let bytes = std::fs::read(p)
        .map_err(|e| PdfExtractError::IoError(format!("Path {:?}: {}", p, e)))?;
    extract_pdf_bytes(&bytes, options)
}

/// Helper converting PDF raw text to structured Markdown with demarcations and detected tables.
fn process_page_text_to_markdown(
    raw_text: &str,
    page_num: u32,
    total_pages: usize,
    options: &PdfExtractOptions,
) -> (String, usize) {
    let mut md = String::new();

    if options.demarcate_pages {
        md.push_str(&format!("<!-- Page {} of {} -->\n\n", page_num, total_pages));
    }

    let lines: Vec<&str> = raw_text.lines().map(str::trim_end).collect();
    let mut table_count = 0usize;
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx];

        // Check if current line starts a tabular block
        if options.detect_tables && is_potential_table_row(line, options.min_col_gap_spaces) {
            let mut table_block = Vec::new();
            table_block.push(line);
            let mut next_idx = idx + 1;

            while next_idx < lines.len() && is_potential_table_row(lines[next_idx], options.min_col_gap_spaces) {
                table_block.push(lines[next_idx]);
                next_idx += 1;
            }

            if table_block.len() >= options.min_table_rows {
                let (md_table, cols_detected) = format_markdown_table(&table_block, options.min_col_gap_spaces);
                if cols_detected >= 2 {
                    if !md.is_empty() && !md.ends_with("\n\n") {
                        md.push_str("\n\n");
                    }
                    md.push_str(&md_table);
                    table_count += 1;
                    idx = next_idx;
                    continue;
                }
            }
        }

        // Regular textual paragraph / heading handling
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if !md.is_empty() && !md.ends_with('\n') {
                md.push('\n');
            }
            md.push_str(trimmed);
        } else if !md.ends_with("\n\n") && !md.is_empty() {
            md.push('\n');
        }

        idx += 1;
    }

    (md.trim().to_string(), table_count)
}

/// Checks if a string contains 2+ multi-space separated tokens representing columns.
fn is_potential_table_row(line: &str, min_gap: usize) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() < 4 {
        return false;
    }

    // Must contain at least one gap of `min_gap` spaces or tab
    if line.contains('\t') {
        return line.split('\t').filter(|c| !c.trim().is_empty()).count() >= 2;
    }

    let gap_pattern = " ".repeat(min_gap);
    if line.contains(&gap_pattern) {
        let cols: Vec<&str> = line
            .split(&gap_pattern)
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        return cols.len() >= 2;
    }

    false
}

/// Formats a block of aligned tabular rows into a GitHub-flavored Markdown table.
fn format_markdown_table(rows: &[&str], min_gap: usize) -> (String, usize) {
    let gap_pattern = " ".repeat(min_gap);
    let mut parsed_rows: Vec<Vec<String>> = Vec::with_capacity(rows.len());
    let mut max_cols = 0usize;

    for r in rows {
        let cols: Vec<String> = if r.contains('\t') {
            r.split('\t')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect()
        } else {
            r.split(&gap_pattern)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect()
        };

        if cols.len() > max_cols {
            max_cols = cols.len();
        }
        parsed_rows.push(cols);
    }

    if max_cols < 2 {
        return (rows.join("\n"), max_cols);
    }

    let mut out = String::new();
    // Header
    if let Some(header) = parsed_rows.first() {
        out.push_str("| ");
        for i in 0..max_cols {
            let col = header.get(i).map(String::as_str).unwrap_or("Col");
            out.push_str(col);
            out.push_str(" | ");
        }
        out.push('\n');

        // Separator
        out.push_str("| ");
        for _ in 0..max_cols {
            out.push_str("--- | ");
        }
        out.push('\n');
    }

    // Rows
    for row in parsed_rows.iter().skip(1) {
        out.push_str("| ");
        for i in 0..max_cols {
            let col = row.get(i).map(String::as_str).unwrap_or("");
            out.push_str(col);
            out.push_str(" | ");
        }
        out.push('\n');
    }

    (out.trim_end().to_string(), max_cols)
}

/// Helper to sanitize PDF string bytes (handling UTF-16BE BOM and ASCII).
fn clean_pdf_string(obj: &lopdf::Object) -> Option<String> {
    match obj {
        lopdf::Object::String(bytes, _) => {
            if bytes.starts_with(&[0xFE, 0xFF]) {
                // UTF-16BE
                let u16_chars: Vec<u16> = bytes[2..]
                    .chunks_exact(2)
                    .map(|pair| ((pair[0] as u16) << 8) | (pair[1] as u16))
                    .collect();
                String::from_utf16(&u16_chars).ok()
            } else {
                String::from_utf8(bytes.clone())
                    .or_else(|_| Ok::<String, ()>(String::from_utf8_lossy(bytes).to_string()))
                    .ok()
            }
        }
        lopdf::Object::Name(name) => String::from_utf8(name.clone()).ok(),
        _ => None,
    }
}

/// End-to-end deterministic processing pipeline for PDF documents in STORM Deep Research.
///
/// 1. Extracts streaming text and tables into clean Markdown.
/// 2. Performs SipHash deduplication against the research registry.
/// 3. Chunks the document into sliding window passages (~500 words).
/// 4. Ranks the passages using Okapi BM25 against the Scout's query.
#[must_use]
pub fn process_raw_pdf_passage(
    pdf_bytes: &[u8],
    source_id: &str,
    query: &str,
    registry: &mut ResearchDedupRegistry,
    top_k: usize,
) -> Result<FilteredPassageResult, PdfExtractError> {
    let canonical_url = normalize_url(source_id);
    let original_bytes = pdf_bytes.len();

    // 1. Deduplication check
    let is_dup = !registry.register_url(source_id);
    if is_dup {
        return Ok(FilteredPassageResult {
            canonical_url,
            is_duplicate: true,
            original_bytes,
            clean_markdown_bytes: 0,
            total_chunks: 0,
            top_passages: Vec::new(),
            compression_ratio: 0.0,
        });
    }

    // 2. Deterministic PDF Extraction
    let options = PdfExtractOptions::default();
    let extract_res = extract_pdf_bytes(pdf_bytes, &options)?;

    let clean_md = extract_res.full_markdown;
    let clean_markdown_bytes = clean_md.len();

    // 3. Content fingerprint deduplication check
    let is_content_dup = !registry.register_content(&clean_md);
    if is_content_dup {
        return Ok(FilteredPassageResult {
            canonical_url,
            is_duplicate: true,
            original_bytes,
            clean_markdown_bytes,
            total_chunks: 0,
            top_passages: Vec::new(),
            compression_ratio: 0.0,
        });
    }

    // 4. Sliding window chunking
    let chunks = chunk_document(&clean_md, &Default::default());
    let total_chunks = chunks.len();

    if chunks.is_empty() {
        return Ok(FilteredPassageResult {
            canonical_url,
            is_duplicate: false,
            original_bytes,
            clean_markdown_bytes,
            total_chunks: 0,
            top_passages: Vec::new(),
            compression_ratio: if original_bytes > 0 {
                clean_markdown_bytes as f64 / original_bytes as f64
            } else {
                1.0
            },
        });
    }

    // 5. Okapi BM25 Lexical Ranking against Query
    use crate::research::bm25::{DEFAULT_BM25_B, DEFAULT_BM25_K1};
    let ranked = rank_chunks(&chunks, query, top_k, DEFAULT_BM25_K1, DEFAULT_BM25_B);
    let top_passages = ranked
        .into_iter()
        .map(|r| FilteredPassage {
            chunk_index: r.chunk.index,
            text: r.chunk.text.clone(),
            word_count: r.chunk.word_count,
            bm25_score: r.score,
        })
        .collect();

    let compression_ratio = if original_bytes > 0 {
        clean_markdown_bytes as f64 / original_bytes as f64
    } else {
        1.0
    };

    Ok(FilteredPassageResult {
        canonical_url,
        is_duplicate: false,
        original_bytes,
        clean_markdown_bytes,
        total_chunks,
        top_passages,
        compression_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_potential_table_row() {
        assert!(is_potential_table_row("Item    Price    Quantity", 2));
        assert!(is_potential_table_row("ColA\tColB\tColC", 2));
        assert!(!is_potential_table_row("This is just regular text with single spaces.", 2));
        assert!(!is_potential_table_row("", 2));
    }

    #[test]
    fn test_format_markdown_table() {
        let rows = vec![
            "Model           TPS    Latency",
            "Arch-Function   47.1   229ms",
            "Phi-4-Mini      38.4   290ms",
        ];
        let (md, cols) = format_markdown_table(&rows, 2);
        assert_eq!(cols, 3);
        assert!(md.contains("| Model | TPS | Latency |"));
        assert!(md.contains("| --- | --- | --- |"));
        assert!(md.contains("| Arch-Function | 47.1 | 229ms |"));
    }

    #[test]
    fn test_empty_pdf_returns_error() {
        let err = extract_pdf_bytes(&[], &Default::default()).unwrap_err();
        match err {
            PdfExtractError::EmptyDocument => {}
            _ => panic!("Expected EmptyDocument error"),
        }
    }

    #[test]
    fn test_synthetic_pdf_in_memory_extraction() {
        use lopdf::{dictionary, Document, Object, Stream};
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        });
        let content = b"BT /F1 12 Tf 100 700 Td (Architecture Benchmark Report) Tj ET\nBT 100 650 Td (Model    TPS    Latency) Tj ET\nBT 100 630 Td (Qwen3.5  46.6   241ms) Tj ET";
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.to_vec()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);

        let mut buffer = Vec::new();
        doc.save_to(&mut buffer).unwrap();

        // 1. Extract bytes
        let opts = PdfExtractOptions::default();
        let res = extract_pdf_bytes(&buffer, &opts).unwrap();
        assert_eq!(res.total_pages, 1);
        assert!(res.full_markdown.contains("<!-- Page 1 of 1 -->"));
        assert!(res.full_markdown.contains("Architecture Benchmark Report"));

        // 2. Test STORM pipeline integration
        let mut registry = ResearchDedupRegistry::new();
        let storm_res = process_raw_pdf_passage(
            &buffer,
            "file:///reports/bench.pdf",
            "Qwen3.5 TPS",
            &mut registry,
            3,
        )
        .unwrap();

        assert!(!storm_res.is_duplicate);
        assert!(storm_res.total_chunks >= 1);
        assert!(!storm_res.top_passages.is_empty());
        assert!(storm_res.top_passages[0].text.contains("Architecture Benchmark Report"));
    }
}

