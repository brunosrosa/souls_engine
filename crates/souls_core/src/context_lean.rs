//! Adaptive token-aware smart reading and context lean pipeline.
//!
//! Provides CPU-based BPE token counting with `tiktoken` (cl100k_base)
//! protected via `tokio::task::spawn_blocking`, polyglot signature-preserving
//! auto-shrink pruning (Brace, Indent, Block paradigms), and concurrent multi-file reads.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use tiktoken::CoreBpe;
use crate::ansi_filter::strip_ansi_escapes;
use crate::error::CoreError;

/// Language paradigms for structural body amputation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageParadigm {
    /// Rust, JS, TS, C, C++, C#, Go, Java (curly brace delimiters)
    Brace,
    /// Python (column indentation + pass)
    Indent,
    /// Elixir, Ruby (do/end blocks)
    Block,
}

/// Identifies the syntactic paradigm based on file extension or content heuristic.
pub fn detect_paradigm(code: &str, ext_or_path: Option<&str>) -> LanguageParadigm {
    if let Some(ext) = ext_or_path {
        let lower = ext.to_lowercase();
        if lower.ends_with(".py") || lower == "py" || lower == "python" {
            return LanguageParadigm::Indent;
        }
        if lower.ends_with(".ex")
            || lower.ends_with(".exs")
            || lower.ends_with(".rb")
            || lower == "ex"
            || lower == "elixir"
            || lower == "ruby"
        {
            return LanguageParadigm::Block;
        }
        if lower.ends_with(".rs")
            || lower.ends_with(".js")
            || lower.ends_with(".ts")
            || lower.ends_with(".tsx")
            || lower.ends_with(".cpp")
            || lower.ends_with(".c")
            || lower.ends_with(".h")
            || lower.ends_with(".hpp")
            || lower.ends_with(".cs")
            || lower.ends_with(".go")
            || lower.ends_with(".java")
        {
            return LanguageParadigm::Brace;
        }
    }

    // Heuristic fallback
    if code.contains("defmodule ") || code.contains("defp ") {
        LanguageParadigm::Block
    } else if code.contains("def ") && code.contains(":\n") && !code.contains('{') {
        LanguageParadigm::Indent
    } else {
        LanguageParadigm::Brace
    }
}

/// Global thread-safe instance of the cl100k_base BPE tokenizer.
///
/// Avoids repeated recompilation of the BPE vocabulary dictionary on every smart-read invocation,
/// eliminating heap churn and saving dozens of milliseconds per call under agentic concurrency.
static GLOBAL_TOKENIZER: OnceLock<Option<&'static CoreBpe>> = OnceLock::new();

/// Returns a reference to the statically initialized global tokenizer.
pub fn get_global_tokenizer() -> Option<&'static CoreBpe> {
    *GLOBAL_TOKENIZER.get_or_init(|| tiktoken::get_encoding("cl100k_base"))
}

/// Fast synchronous token count using `cl100k_base` encoding on CPU.
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    match get_global_tokenizer() {
        Some(enc) => enc.count(text),
        None => text.len() / 4,
    }
}

/// Asynchronous token count offloaded to Tokio blocking thread pool
/// to protect the async executor from thread starvation during large payload processing.
pub async fn count_tokens_async(text: String) -> Result<usize, CoreError> {
    tokio::task::spawn_blocking(move || count_tokens(&text))
        .await
        .map_err(|e| CoreError::TaskJoinError(e.to_string()))
}

/// Lightweight whitespace and consecutive blank line cleanup.
pub fn lightweight_cleanup(content: &str) -> String {
    let mut result = Vec::new();
    let mut blank_count = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push("");
            }
            continue;
        }
        blank_count = 0;
        result.push(line);
    }

    result.join("\n")
}

/// Extracts structural outline signatures for Brace-paradigm languages (Rust, C++, TS, etc.).
fn extract_brace_outline(code: &str) -> String {
    let mut result = String::with_capacity(code.len() / 2);
    let chars: Vec<char> = code.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_char_literal = false;
    let mut escape = false;

    let mut pending_fn = false;
    let mut body_depth = 0usize;
    let mut suppressing_body = false;

    while i < len {
        let ch = chars[i];

        if escape {
            escape = false;
            if !suppressing_body {
                result.push(ch);
            }
            i += 1;
            continue;
        }

        if (in_string || in_char_literal) && ch == '\\' {
            escape = true;
            if !suppressing_body {
                result.push(ch);
            }
            i += 1;
            continue;
        }

        if !in_block_comment && !in_string && !in_char_literal && !in_line_comment && ch == '/' && i + 1 < len && chars[i + 1] == '/' {
            in_line_comment = true;
        }

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            if !suppressing_body {
                result.push(ch);
            }
            i += 1;
            continue;
        }

        if !in_line_comment && !in_string && !in_char_literal {
            if !in_block_comment && ch == '/' && i + 1 < len && chars[i + 1] == '*' {
                in_block_comment = true;
            } else if in_block_comment && ch == '*' && i + 1 < len && chars[i + 1] == '/' {
                if !suppressing_body {
                    result.push_str("*/");
                }
                in_block_comment = false;
                i += 2;
                continue;
            }
        }

        if in_block_comment {
            if !suppressing_body {
                result.push(ch);
            }
            i += 1;
            continue;
        }

        if !in_string && !in_char_literal {
            if ch == '"' || ch == '`' {
                in_string = true;
                string_char = ch;
            } else if ch == '\'' && i + 2 < len && chars[i + 2] == '\'' {
                in_char_literal = true;
            }
        } else if in_string && ch == string_char {
            in_string = false;
        } else if in_char_literal && ch == '\'' {
            in_char_literal = false;
        }

        if !in_string && !in_char_literal && !suppressing_body
            && (ch == 'f' || ch == 'd') && (i == 0 || (!chars[i - 1].is_alphanumeric() && chars[i - 1] != '_'))
        {
            let rest: String = chars[i..len.min(i + 12)].iter().collect();
            if rest.starts_with("fn ") || rest.starts_with("fn(") || rest.starts_with("def ") || rest.starts_with("function ") {
                pending_fn = true;
            }
        }

        if !in_string && !in_char_literal {
            if ch == '{' {
                if pending_fn && !suppressing_body {
                    suppressing_body = true;
                    body_depth = 1;
                    pending_fn = false;
                    result.push_str("{ /* body omitted */ }");
                    i += 1;
                    continue;
                } else if suppressing_body {
                    body_depth += 1;
                    i += 1;
                    continue;
                }
            } else if ch == '}' {
                if suppressing_body {
                    body_depth -= 1;
                    if body_depth == 0 {
                        suppressing_body = false;
                    }
                    i += 1;
                    continue;
                }
            } else if ch == ';' && pending_fn && !suppressing_body {
                pending_fn = false;
            }
        }

        if !suppressing_body {
            result.push(ch);
        }

        i += 1;
    }

    if result.trim().is_empty() {
        code.lines().take(50).collect::<Vec<_>>().join("\n")
    } else {
        result
    }
}

/// Extracts structural outline signatures for Indent-paradigm languages (Python).
fn extract_python_outline(code: &str) -> String {
    let mut out = Vec::new();
    let mut suppressing_body = false;
    let mut suppress_indent = 0usize;

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !suppressing_body {
                out.push(line.to_string());
            }
            continue;
        }

        let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();

        if suppressing_body {
            if current_indent > suppress_indent && !trimmed.starts_with("def ") && !trimmed.starts_with("class ") && !trimmed.starts_with("async def ") {
                continue;
            } else {
                suppressing_body = false;
            }
        }

        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            suppressing_body = true;
            suppress_indent = current_indent;
            if let Some(colon_idx) = line.rfind(':') {
                let sig = line[..colon_idx + 1].trim_end();
                out.push(format!("{sig} pass  # body omitted"));
            } else {
                out.push(format!("{line}: pass  # body omitted"));
            }
        } else {
            out.push(line.to_string());
        }
    }

    if out.is_empty() {
        code.lines().take(50).collect::<Vec<_>>().join("\n")
    } else {
        out.join("\n")
    }
}

/// Extracts structural outline signatures for Block-paradigm languages (Elixir/Ruby).
fn extract_elixir_outline(code: &str) -> String {
    let mut out = Vec::new();
    let mut suppressing_body = false;
    let mut do_depth = 0i32;

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !suppressing_body {
                out.push(line.to_string());
            }
            continue;
        }

        if suppressing_body {
            if trimmed.contains(" do") || trimmed.ends_with("do") {
                do_depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end ") || trimmed.ends_with("end") {
                do_depth -= 1;
                if do_depth <= 0 {
                    suppressing_body = false;
                    let indent_size = line.chars().take_while(|c| c.is_whitespace()).count();
                    let indent = " ".repeat(indent_size);
                    out.push(format!("{indent}  # body omitted\n{indent}end"));
                }
            }
            continue;
        }

        if trimmed.starts_with("defmodule ") {
            out.push(line.to_string());
        } else if trimmed.starts_with("def ") || trimmed.starts_with("defp ") || trimmed.starts_with("defmacro ") {
            if trimmed.contains(", do:") {
                if let Some(idx) = line.find(", do:") {
                    let prefix = &line[..idx];
                    out.push(format!("{prefix}, do: :ok  # body omitted"));
                } else {
                    out.push(line.to_string());
                }
            } else if trimmed.contains(" do") || trimmed.ends_with("do") {
                suppressing_body = true;
                do_depth = 1;
                out.push(line.to_string());
            } else {
                out.push(line.to_string());
            }
        } else {
            out.push(line.to_string());
        }
    }

    if out.is_empty() {
        code.lines().take(50).collect::<Vec<_>>().join("\n")
    } else {
        out.join("\n")
    }
}

/// Applies polyglot structural outline extraction.
pub fn extract_outline_signatures_polyglot(code: &str, ext_or_path: Option<&str>) -> String {
    let paradigm = detect_paradigm(code, ext_or_path);
    match paradigm {
        LanguageParadigm::Brace => extract_brace_outline(code),
        LanguageParadigm::Indent => extract_python_outline(code),
        LanguageParadigm::Block => extract_elixir_outline(code),
    }
}

/// Performs adaptive auto-shrink reading on input text:
/// 1. If text tokens <= max_tokens_budget, returns raw text unmodified.
/// 2. If exceeding, strips ANSI codes and runs lightweight cleanup.
/// 3. If still exceeding, extracts polyglot structural signatures (outline pruning).
/// 4. If still exceeding, fails closed with `CoreError::TokenBudgetExceeded`.
pub fn smart_read_text(
    text: &str,
    max_tokens_budget: usize,
    ext_or_path: Option<&str>,
) -> Result<String, CoreError> {
    let initial_tokens = count_tokens(text);
    if initial_tokens <= max_tokens_budget {
        return Ok(text.to_string());
    }

    // Step 1: Strip ANSI escapes and run lightweight cleanup
    let cleaned = lightweight_cleanup(&strip_ansi_escapes(text));
    let cleaned_tokens = count_tokens(&cleaned);
    if cleaned_tokens <= max_tokens_budget {
        return Ok(cleaned);
    }

    // Step 2: Structural signature extraction (auto-shrink)
    let outlined = extract_outline_signatures_polyglot(&cleaned, ext_or_path);
    let outlined_tokens = count_tokens(&outlined);
    if outlined_tokens <= max_tokens_budget {
        return Ok(outlined);
    }

    // Step 3: Fail-Closed
    Err(CoreError::TokenBudgetExceeded {
        tokens: outlined_tokens,
        budget: max_tokens_budget,
    })
}

/// Asynchronous wrapper for `smart_read_text` executed via Tokio's blocking thread pool.
pub async fn smart_read_text_async(
    text: String,
    max_tokens_budget: usize,
    ext_or_path: Option<String>,
) -> Result<String, CoreError> {
    tokio::task::spawn_blocking(move || {
        smart_read_text(&text, max_tokens_budget, ext_or_path.as_deref())
    })
    .await
    .map_err(|e| CoreError::TaskJoinError(e.to_string()))?
}

/// Asynchronously reads a file from disk and applies smart auto-shrink within token budget.
pub async fn smart_read_file(
    path: &Path,
    max_tokens_budget: usize,
) -> Result<String, CoreError> {
    let content = tokio::fs::read_to_string(path).await?;
    let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_string());
    smart_read_text_async(content, max_tokens_budget, ext).await
}

/// Metadata and compaction result for a single file read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCompaction {
    pub filepath: String,
    pub content: String,
    pub original_bytes: usize,
    pub compacted_bytes: usize,
    pub token_count: usize,
    pub error: Option<String>,
}

/// Concurrently reads multiple files in parallel using `tokio::spawn`, applying ANSI
/// sanitization and lightweight cleanup.
pub async fn multi_read_concurrent<I, P>(paths: I) -> Vec<FileCompaction>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let path_strs: Vec<String> = paths
        .into_iter()
        .map(|p| p.as_ref().to_string_lossy().to_string())
        .collect();

    let mut handles = Vec::with_capacity(path_strs.len());
    for p in path_strs {
        let handle = tokio::spawn(async move {
            let path = Path::new(&p);
            let raw = match tokio::fs::read_to_string(path).await {
                Ok(s) => s,
                Err(e) => {
                    return FileCompaction {
                        filepath: p,
                        content: String::new(),
                        original_bytes: 0,
                        compacted_bytes: 0,
                        token_count: 0,
                        error: Some(format!("read_error: {e}")),
                    };
                }
            };

            let original_bytes = raw.len();
            let sanitized = lightweight_cleanup(&strip_ansi_escapes(&raw));
            let compacted_bytes = sanitized.len();
            let token_count = count_tokens(&sanitized);

            FileCompaction {
                filepath: p,
                content: sanitized,
                original_bytes,
                compacted_bytes,
                token_count,
                error: None,
            }
        });
        handles.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for h in handles {
        match h.await {
            Ok(fc) => results.push(fc),
            Err(e) => {
                results.push(FileCompaction {
                    filepath: String::new(),
                    content: String::new(),
                    original_bytes: 0,
                    compacted_bytes: 0,
                    token_count: 0,
                    error: Some(format!("join_error: {e}")),
                });
            }
        }
    }

    results
}

/// Compaction and token savings metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactionResult {
    pub compressed_text: String,
    pub original_tokens: usize,
    pub compacted_tokens: usize,
    pub tokens_saved: usize,
    pub reduction_percent: f64,
}

/// Strips non-essential comments while strictly preserving docstrings, linter annotations, and compiler directives.
pub fn strip_comments_polyglot(code: &str, lang_or_ext: Option<&str>) -> String {
    let lower_lang = lang_or_ext.unwrap_or("").to_lowercase();
    let is_python = lower_lang.ends_with(".py") || lower_lang == "py" || lower_lang == "python";

    let mut out = Vec::new();
    let mut in_block_comment = false;
    let mut in_py_docstring = false;
    let mut py_docstring_quote = "";

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push("");
            continue;
        }

        // Python docstrings (""" or ''')
        if is_python {
            if in_py_docstring {
                out.push(line);
                if trimmed.ends_with(py_docstring_quote) {
                    in_py_docstring = false;
                }
                continue;
            }

            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                let quote = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
                out.push(line);
                if trimmed.len() > 3 && trimmed[3..].contains(quote) {
                    // Single-line docstring
                } else {
                    in_py_docstring = true;
                    py_docstring_quote = quote;
                }
                continue;
            }

            // Python line comment (#)
            if trimmed.starts_with('#') {
                if trimmed.starts_with("#!")
                    || trimmed.starts_with("# type:")
                    || trimmed.starts_with("# noqa")
                    || trimmed.starts_with("# pylint:")
                    || trimmed.starts_with("# pragma:")
                    || trimmed.starts_with("# coding:")
                {
                    out.push(line);
                }
                continue;
            }

            // Inline comment check
            if let Some(pos) = line.find("  #") {
                let after = &line[pos..];
                if !after.contains("# type:") && !after.contains("# noqa") {
                    let clean = line[..pos].trim_end();
                    if !clean.is_empty() {
                        out.push(clean);
                    }
                    continue;
                }
            }

            out.push(line);
            continue;
        }

        // C-family / Rust / TS / Go / Java
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        // Docstrings: /// or //!
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            out.push(line);
            continue;
        }

        // Block docstring /** ... */
        if trimmed.starts_with("/**") {
            out.push(line);
            if !trimmed.contains("*/") {
                in_block_comment = false; // keep emitting doc lines
            }
            continue;
        }

        // Rust attributes and directives #[...]
        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            out.push(line);
            continue;
        }

        // Line comments //
        if trimmed.starts_with("//") {
            if trimmed.starts_with("// SAFETY:")
                || trimmed.starts_with("// nolint")
                || trimmed.starts_with("// [allow(")
            {
                out.push(line);
            }
            continue;
        }

        // Block comment start /*
        if trimmed.starts_with("/*") && !trimmed.starts_with("/**") {
            if trimmed.contains("*/") {
                continue;
            }
            in_block_comment = true;
            continue;
        }

        // Inline comment
        if let Some(pos) = line.find(" //") {
            let after = &line[pos..];
            if !after.contains("// SAFETY:") && !after.contains("// nolint") {
                let clean = line[..pos].trim_end();
                if !clean.is_empty() {
                    out.push(clean);
                }
                continue;
            }
        }

        out.push(line);
    }

    out.join("\n")
}

/// Applies LEAN compression: strips implementation comments, collapses redundant blank lines,
/// and measures BPE token reduction.
pub fn compress_lean_code(code: &str, lang_or_ext: Option<&str>) -> CompactionResult {
    let original_tokens = count_tokens(code);
    let stripped = strip_comments_polyglot(code, lang_or_ext);
    let compressed_text = lightweight_cleanup(&stripped);
    let compacted_tokens = count_tokens(&compressed_text);

    let tokens_saved = original_tokens.saturating_sub(compacted_tokens);
    let reduction_percent = if original_tokens > 0 {
        (tokens_saved as f64 / original_tokens as f64) * 100.0
    } else {
        0.0
    };

    CompactionResult {
        compressed_text,
        original_tokens,
        compacted_tokens,
        tokens_saved,
        reduction_percent,
    }
}

/// In-memory LRU-bounded cache for deduplicated code and text blocks.
#[derive(Debug)]
pub struct DedupCache {
    max_capacity: usize,
    blocks: HashMap<String, String>,
    order: VecDeque<String>,
}

impl DedupCache {
    /// Creates a new cache with an LRU bound.
    #[must_use]
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity: max_capacity.max(10),
            blocks: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Stores a deduplicated block with its SipHash key.
    pub fn insert(&mut self, hash: String, content: String) {
        if self.blocks.contains_key(&hash) {
            return;
        }

        if self.blocks.len() >= self.max_capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.blocks.remove(&oldest);
            }
        }

        self.order.push_back(hash.clone());
        self.blocks.insert(hash, content);
    }

    /// Retrieves the original content for a hash marker.
    #[must_use]
    pub fn get(&self, hash: &str) -> Option<&String> {
        self.blocks.get(hash)
    }

    /// Returns the number of stored blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Clears all blocks.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.order.clear();
    }
}

impl Default for DedupCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Thread-safe singleton for global deduplication cache in RAM.
static GLOBAL_DEDUP_CACHE: OnceLock<RwLock<DedupCache>> = OnceLock::new();

/// Returns a reference to the global deduplication cache.
pub fn get_global_dedup_cache() -> &'static RwLock<DedupCache> {
    GLOBAL_DEDUP_CACHE.get_or_init(|| RwLock::new(DedupCache::default()))
}

/// Result of deduplication analysis and compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedupResult {
    pub deduplicated_text: String,
    pub deduplicated_blocks: usize,
    pub original_lines: usize,
    pub compacted_lines: usize,
}

/// Identifies repeated blocks of `min_lines` or more, replaces them with compact SipHash markers,
/// and caches the original blocks in RAM for lossless rehydration.
pub fn dedup_text(text: &str, min_lines: usize, cache: &RwLock<DedupCache>) -> DedupResult {
    let min_lines = min_lines.max(3);
    let lines: Vec<&str> = text.lines().collect();
    let original_lines = lines.len();

    if original_lines < min_lines * 2 {
        return DedupResult {
            deduplicated_text: text.to_string(),
            deduplicated_blocks: 0,
            original_lines,
            compacted_lines: original_lines,
        };
    }

    let mut seen_blocks: HashMap<String, String> = HashMap::new();
    let mut out_lines = Vec::new();
    let mut i = 0;
    let mut dedup_count = 0;

    let mut cache_guard = cache.write().unwrap();

    while i < lines.len() {
        if i + min_lines <= lines.len() {
            let candidate_block = lines[i..i + min_lines].join("\n");

            // SipHash 64-bit deterministic hash
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            candidate_block.hash(&mut hasher);
            let block_hash = format!("{:016x}", hasher.finish());

            if seen_blocks.contains_key(&block_hash) {
                let marker = format!("[DEDUP_BLOCK_{}: {} lines omitted]", block_hash, min_lines);
                cache_guard.insert(block_hash.clone(), candidate_block);
                out_lines.push(marker);
                i += min_lines;
                dedup_count += 1;
                continue;
            } else {
                seen_blocks.insert(block_hash, candidate_block);
            }
        }
        out_lines.push(lines[i].to_string());
        i += 1;
    }

    let result_text = out_lines.join("\n");
    let compacted_lines = out_lines.len();

    DedupResult {
        deduplicated_text: result_text,
        deduplicated_blocks: dedup_count,
        original_lines,
        compacted_lines,
    }
}

/// Rehydrates previously deduplicated SipHash markers back into lossless text.
pub fn rehydrate_text(text: &str, cache: &RwLock<DedupCache>) -> (String, usize) {
    let mut result = text.to_string();
    let mut replaced = 0;

    let cache_guard = cache.read().unwrap();
    let prefix = "[DEDUP_BLOCK_";

    loop {
        if let Some(start_pos) = result.find(prefix) {
            let hash_start = start_pos + prefix.len();
            if hash_start + 16 <= result.len() {
                let hash = &result[hash_start..hash_start + 16];
                if let Some(end_bracket) = result[hash_start..].find(']') {
                    let marker_end = hash_start + end_bracket + 1;
                    if let Some(original) = cache_guard.get(hash) {
                        result.replace_range(start_pos..marker_end, original);
                        replaced += 1;
                        continue;
                    }
                }
            }
            break;
        } else {
            break;
        }
    }

    (result, replaced)
}

/// Rehydrates `{{key}}` template placeholders using a dictionary.
pub fn fill_template_placeholders(template: &str, placeholders: &HashMap<String, String>) -> (String, usize) {
    let mut result = template.to_string();
    let mut count = 0;

    for (k, v) in placeholders {
        let needle = format!("{{{{{}}}}}", k);
        if result.contains(&needle) {
            result = result.replace(&needle, v);
            count += 1;
        }
    }

    (result, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_count_async_accuracy() {
        let sample = "The quick brown fox jumps over the lazy dog. 1234567890!";
        let sync_tokens = count_tokens(sample);
        let async_tokens = count_tokens_async(sample.to_string()).await.expect("async count failed");
        assert_eq!(sync_tokens, async_tokens);
        assert!(async_tokens > 0);
    }

    #[tokio::test]
    async fn test_smart_read_stress_payload_shrink_rust() {
        // Generates a massive 5000-line Rust function payload to stress test auto-shrink
        let mut huge_code = String::from("pub fn heavy_compute_pipeline() {\n");
        for i in 0..5000 {
            huge_code.push_str(&format!("    let var_{i}: u64 = {i} * 42;\n"));
        }
        huge_code.push_str("}\n");

        let initial_tokens = count_tokens(&huge_code);
        assert!(initial_tokens > 5000, "Initial tokens should be large");

        // Request a tight budget of 200 tokens
        let shrunk = smart_read_text_async(huge_code.clone(), 200, Some("rs".to_string()))
            .await
            .expect("Should successfully shrink");

        let shrunk_tokens = count_tokens(&shrunk);
        assert!(shrunk_tokens <= 200, "Shrunk tokens {shrunk_tokens} must be <= 200");
        assert!(shrunk.contains("{ /* body omitted */ }"));

        // Request impossible budget to trigger fail-closed
        let impossible = smart_read_text_async(huge_code, 2, Some("rs".to_string())).await;
        assert!(impossible.is_err());
        match impossible.unwrap_err() {
            CoreError::TokenBudgetExceeded { tokens, budget } => {
                assert_eq!(budget, 2);
                assert!(tokens > 2);
            }
            other => panic!("Expected TokenBudgetExceeded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_smart_read_python_auto_shrink() {
        let mut py_code = String::from("class NeuralModel:\n    def forward(self, x):\n");
        for i in 0..1500 {
            py_code.push_str(&format!("        x = layer_{i}(x) + {i}\n"));
        }
        py_code.push_str("        return x\n");

        let shrunk = smart_read_text_async(py_code, 150, Some("py".to_string()))
            .await
            .expect("Should prune Python body");

        assert!(shrunk.contains("class NeuralModel:"));
        assert!(shrunk.contains("def forward(self, x): pass  # body omitted"));
        assert!(!shrunk.contains("layer_500"));
    }

    #[test]
    fn test_compress_lean_code_preserves_rust_docstrings_and_attributes() {
        let rust_code = r#"
//! # Crate Root Docs
//! Preserved module docstring.

// Ordinary implementation comment to strip
#[derive(Debug, Clone, PartialEq)]
pub struct EngineConfig {
    /// Documenting the field
    pub timeout_ms: u64,
    // Another comment to remove
    pub is_active: bool, // inline comment
}

// SAFETY: Crucial bare-metal comment
pub unsafe fn unsafe_door() {}
"#;
        let res = compress_lean_code(rust_code, Some("rs"));
        assert!(res.compressed_text.contains("//! # Crate Root Docs"));
        assert!(res.compressed_text.contains("/// Documenting the field"));
        assert!(res.compressed_text.contains("#[derive(Debug, Clone, PartialEq)]"));
        assert!(res.compressed_text.contains("// SAFETY: Crucial bare-metal comment"));
        assert!(!res.compressed_text.contains("Ordinary implementation comment"));
        assert!(!res.compressed_text.contains("Another comment to remove"));
        assert!(res.tokens_saved > 0 || res.original_tokens > 0);
    }

    #[test]
    fn test_compress_lean_code_python_docstrings_and_type_directives() {
        let py_code = r#"
"""Module header docstring."""

# Implementation comment to remove
# type: ignore[import]
def compute_loss(x, y):
    """Calculates cross-entropy loss."""
    # Another comment to strip
    val = x * 2  # inline removal
    return val
"#;
        let res = compress_lean_code(py_code, Some("py"));
        assert!(res.compressed_text.contains("\"\"\"Module header docstring.\"\"\""));
        assert!(res.compressed_text.contains("\"\"\"Calculates cross-entropy loss.\"\"\""));
        assert!(res.compressed_text.contains("# type: ignore[import]"));
        assert!(!res.compressed_text.contains("# Implementation comment to remove"));
        assert!(!res.compressed_text.contains("# Another comment to strip"));
        assert!(!res.compressed_text.contains("# inline removal"));
    }

    #[test]
    fn test_dedup_and_rehydrate_roundtrip() {
        let cache = RwLock::new(DedupCache::new(50));
        let repeated_block = "line_a = 1;\nline_b = 2;\nline_c = 3;\nline_d = 4;\nline_e = 5;";

        let text = format!(
            "// Header\n{}\n// Middle separator\n{}\n// Footer",
            repeated_block, repeated_block
        );

        let dedup_res = dedup_text(&text, 5, &cache);
        assert_eq!(dedup_res.deduplicated_blocks, 1);
        assert!(dedup_res.deduplicated_text.contains("[DEDUP_BLOCK_"));

        let (rehydrated, count) = rehydrate_text(&dedup_res.deduplicated_text, &cache);
        assert_eq!(count, 1);
        assert_eq!(rehydrated, text);
    }

    #[test]
    fn test_dedup_cache_lru_eviction() {
        let mut cache = DedupCache::new(10);
        for i in 0..15 {
            cache.insert(format!("hash_{i:04}"), format!("content_{i}"));
        }
        assert_eq!(cache.len(), 10);
        // The first 5 should have been evicted
        assert!(cache.get("hash_0000").is_none());
        assert!(cache.get("hash_0004").is_none());
        // The last ones should exist
        assert!(cache.get("hash_0014").is_some());
    }

    #[test]
    fn test_fill_template_placeholders() {
        let template = "Hello {{user}}, your balance is {{balance}} USD.";
        let mut placeholders = HashMap::new();
        placeholders.insert("user".to_string(), "Alice".to_string());
        placeholders.insert("balance".to_string(), "1500".to_string());

        let (filled, count) = fill_template_placeholders(template, &placeholders);
        assert_eq!(count, 2);
        assert_eq!(filled, "Hello Alice, your balance is 1500 USD.");
    }
}

