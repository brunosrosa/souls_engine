//! Adaptive token-aware smart reading and context lean pipeline.
//!
//! Provides CPU-based BPE token counting with `tiktoken` (cl100k_base)
//! protected via `tokio::task::spawn_blocking`, polyglot signature-preserving
//! auto-shrink pruning (Brace, Indent, Block paradigms), and concurrent multi-file reads.

use std::path::Path;
use tiktoken::get_encoding;
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

/// Fast synchronous token count using `cl100k_base` encoding on CPU.
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    match get_encoding("cl100k_base") {
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
        } else if trimmed.starts_with("class ") {
            out.push(line.to_string());
        } else if trimmed.starts_with("import ") || trimmed.starts_with("from ") || trimmed.starts_with('#') || current_indent == 0 {
            out.push(line.to_string());
        } else if !suppressing_body {
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
        } else if trimmed.starts_with("use ") || trimmed.starts_with("import ") || trimmed.starts_with("alias ") || trimmed.starts_with('#') {
            out.push(line.to_string());
        } else if !suppressing_body {
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
}
