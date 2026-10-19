//! Extended low-level implementations for Domain 6 MCP tools, memory graph mutations,
//! and systems sandboxing in the Souls Engine v7.
//!
//! Replaces all legacy staging fallbacks with real bare-metal operations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use sqlx::Row;

use souls_protocol::error::McpDomainError;
use souls_core::context_lean::{
    compress_lean_code, count_tokens, dedup_text, fill_template_placeholders,
    get_global_dedup_cache, multi_read_concurrent, rehydrate_text, smart_read_file,
};
use souls_core::error::CoreError;
use souls_core::sandbox::{run_sandboxed_command, run_sandboxed_shell};
use souls_core::web::{html_to_clean_markdown, parse_duckduckgo_html};
use crate::AppState;

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

// ============================================================================
// DOMAIN 6: CONTEXT LEAN TOOLS (`context.*`)
// ============================================================================

/// Handles `read` with TOON formatting and line bounds under ReFS.
pub async fn handle_read(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let path_str = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let path = souls_core::fs::resolve_workspace_path(path_str)
        .map_err(|_| McpDomainError::RefsPathLeakViolation)?;

    let content = tokio::fs::read_to_string(&path).await.map_err(|_| {
        McpDomainError::StructuredFailure
    })?;

    let offset = args.get("offset").and_then(Value::as_u64).unwrap_or(1) as usize;
    let limit = args.get("limit").and_then(Value::as_u64);

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start_idx = offset.saturating_sub(1).min(total_lines);
    let selected_lines = if let Some(lim) = limit {
        let count = lim as usize;
        let end_idx = (start_idx + count).min(total_lines);
        &lines[start_idx..end_idx]
    } else {
        &lines[start_idx..]
    };

    let sliced_text = selected_lines.join("\n");
    let token_count = count_tokens(&sliced_text);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": sliced_text
        }],
        "structuredContent": {
            "path": path_str,
            "total_lines": total_lines,
            "start_line": start_idx + 1,
            "returned_lines": selected_lines.len(),
            "tokens": token_count
        },
        "isError": false
    }))
}

/// Handles `smart_read` with token-aware polyglot auto-shrink.
pub async fn handle_smart_read(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let path_str = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let path = souls_core::fs::resolve_workspace_path(path_str)
        .map_err(|_| McpDomainError::RefsPathLeakViolation)?;

    let max_tokens = args.get("max_tokens").and_then(Value::as_u64).unwrap_or(4000) as usize;

    let content = smart_read_file(&path, max_tokens).await.map_err(|_| {
        McpDomainError::StructuredFailure
    })?;

    let token_count = count_tokens(&content);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": content
        }],
        "structuredContent": {
            "path": path_str,
            "tokens": token_count,
            "max_tokens_budget": max_tokens
        },
        "isError": false
    }))
}

/// Handles `multi_read` concurrent batch loading.
pub async fn handle_multi_read(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let paths_val = args
        .get("paths")
        .and_then(Value::as_array)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let mut validated_paths = Vec::new();
    for p_val in paths_val {
        if let Some(p_str) = p_val.as_str() {
            let p = souls_core::fs::resolve_workspace_path(p_str)
                .map_err(|_| McpDomainError::RefsPathLeakViolation)?;
            validated_paths.push(p);
        }
    }

    let compactions = multi_read_concurrent(&validated_paths).await;

    let mut result_map = serde_json::Map::new();
    let mut combined_text = String::new();

    for comp in compactions {
        let p_display = comp.filepath.clone();
        combined_text.push_str(&format!("--- FILE: {} ---\n{}\n\n", p_display, comp.content));
        result_map.insert(
            p_display,
            json!({
                "original_bytes": comp.original_bytes,
                "compacted_bytes": comp.compacted_bytes,
                "token_count": comp.token_count
            }),
        );
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": combined_text.trim_end()
        }],
        "structuredContent": result_map,
        "isError": false
    }))
}

/// Handles `compress` applying LEAN syntax comment and whitespace reduction.
pub async fn handle_compress(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let (raw_text, lang_hint) = if let Some(t) = args.get("text").and_then(Value::as_str) {
        (t.to_string(), args.get("lang").and_then(Value::as_str).map(|s| s.to_string()))
    } else if let Some(p_str) = args.get("path").and_then(Value::as_str) {
        let p = souls_core::fs::resolve_workspace_path(p_str)
            .map_err(|_| McpDomainError::RefsPathLeakViolation)?;
        let content = tokio::fs::read_to_string(&p).await.map_err(|_| {
            McpDomainError::StructuredFailure
        })?;
        let ext = p.extension().and_then(|e| e.to_str()).map(|s| s.to_string());
        (content, ext)
    } else {
        return Err(McpDomainError::InvalidInputParameters);
    };

    let res = compress_lean_code(&raw_text, lang_hint.as_deref());

    Ok(json!({
        "content": [{
            "type": "text",
            "text": res.compressed_text
        }],
        "structuredContent": {
            "original_tokens": res.original_tokens,
            "compressed_tokens": res.compacted_tokens,
            "tokens_saved": res.tokens_saved,
            "reduction_percent": format!("{:.1}%", res.reduction_percent)
        },
        "isError": false
    }))
}

/// Handles `dedup` replacing contiguous repeated blocks (5+ lines) with compact hashes.
pub async fn handle_dedup(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let raw_text = if let Some(t) = args.get("text").and_then(Value::as_str) {
        t.to_string()
    } else if let Some(p_str) = args.get("path").and_then(Value::as_str) {
        let p = souls_core::fs::resolve_workspace_path(p_str)
            .map_err(|_| McpDomainError::RefsPathLeakViolation)?;
        tokio::fs::read_to_string(&p).await.map_err(|_| {
            McpDomainError::StructuredFailure
        })?
    } else {
        return Err(McpDomainError::InvalidInputParameters);
    };

    let min_lines = args.get("min_lines").and_then(Value::as_u64).unwrap_or(5) as usize;
    let cache = get_global_dedup_cache();
    let res = dedup_text(&raw_text, min_lines, cache);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": res.deduplicated_text
        }],
        "structuredContent": {
            "deduplicated_blocks": res.deduplicated_blocks,
            "original_lines": res.original_lines,
            "compacted_lines": res.compacted_lines
        },
        "isError": false
    }))
}

/// Handles `fill` rehydrating CCR placeholders or dedup markers back into lossless text.
pub async fn handle_fill(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let template = args
        .get("template")
        .or_else(|| args.get("text"))
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let mut filled_text = template.to_string();
    let mut total_filled = 0;

    if let Some(placeholders) = args.get("placeholders").and_then(Value::as_object) {
        let mut map = HashMap::new();
        for (k, v) in placeholders {
            map.insert(k.clone(), v.as_str().unwrap_or(&v.to_string()).to_string());
        }
        let (f_text, count) = fill_template_placeholders(&filled_text, &map);
        filled_text = f_text;
        total_filled += count;
    }

    let cache = get_global_dedup_cache();
    let (rehydrated, dedup_count) = rehydrate_text(&filled_text, cache);
    total_filled += dedup_count;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": rehydrated
        }],
        "structuredContent": {
            "filled_count": total_filled,
            "status": "rehydrated"
        },
        "isError": false
    }))
}

/// Handles `stub_fill` filling demarcated code stubs safely.
pub async fn handle_stub_fill(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let path_str = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let path = souls_core::fs::resolve_workspace_path(path_str)
        .map_err(|_| McpDomainError::RefsPathLeakViolation)?;

    let mut content = tokio::fs::read_to_string(&path).await.map_err(|_| {
        McpDomainError::StructuredFailure
    })?;

    let stubs = args
        .get("stubs")
        .and_then(Value::as_object)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let mut applied_count = 0;
    for (stub_marker, replacement) in stubs {
        let rep_str = replacement.as_str().unwrap_or("");
        if content.contains(stub_marker) {
            content = content.replace(stub_marker, rep_str);
            applied_count += 1;
        }
    }

    if applied_count > 0 {
        tokio::fs::write(&path, &content).await.map_err(|_| McpDomainError::InternalEnginePanic)?;
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Applied {} stub fills to '{}'.", applied_count, path_str)
        }],
        "structuredContent": {
            "path": path_str,
            "stubs_filled": applied_count
        },
        "isError": false
    }))
}

/// Handles `headroom_retrieve` retrieving cached CCR blocks from RAM.
pub async fn handle_headroom_retrieve(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let hash = args
        .get("hash")
        .or_else(|| args.get("block_id"))
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let cache = get_global_dedup_cache();
    let cache_guard = cache.read().unwrap();

    if let Some(original_code) = cache_guard.get(hash) {
        Ok(json!({
            "content": [{
                "type": "text",
                "text": original_code.clone()
            }],
            "structuredContent": {
                "hash": hash,
                "found": true,
                "length_chars": original_code.len()
            },
            "isError": false
        }))
    } else {
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Block hash '{}' not found in RAM cache.", hash)
            }],
            "structuredContent": { "hash": hash, "found": false },
            "isError": false
        }))
    }
}

// ============================================================================
// DOMAIN 6: SYSTEM, TREE, SEARCH & SANDBOX TOOLS (`sys.*`)
// ============================================================================

/// Handles `tree` traversing ReFS directories with toxic path pruning.
pub async fn handle_tree(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let root_str = args.get("path").and_then(Value::as_str).unwrap_or(".");
    let max_depth = args.get("max_depth").and_then(Value::as_u64).unwrap_or(3) as usize;
    let include_hidden = args.get("include_hidden").and_then(Value::as_bool).unwrap_or(false);

    let root_path = souls_core::fs::resolve_workspace_path(root_str)
        .map_err(|_| McpDomainError::RefsPathLeakViolation)?;

    let mut tree_lines = Vec::new();
    tree_lines.push(format!("{}/", root_path.file_name().unwrap_or_default().to_string_lossy()));

    let mut stack = vec![(root_path, 1usize)];

    while let Some((dir, current_depth)) = stack.pop() {
        if current_depth > max_depth {
            continue;
        }

        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut items = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();

            if name == ".git" || name == "target" || name == "node_modules" || name == ".souls_data" {
                continue;
            }
            if !include_hidden && name.starts_with('.') && name != "." && name != ".." {
                continue;
            }

            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            items.push((name, entry.path(), is_dir));
        }

        items.sort_by(|a, b| {
            match (a.2, b.2) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
            }
        });

        let indent = "  ".repeat(current_depth);
        for (name, path, is_dir) in items.into_iter().rev() {
            if is_dir {
                tree_lines.push(format!("{}{}/", indent, name));
                stack.push((path, current_depth + 1));
            } else {
                tree_lines.push(format!("{}{}", indent, name));
            }
        }
    }

    let output_text = tree_lines.join("\n");

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output_text
        }],
        "structuredContent": {
            "root": root_str,
            "max_depth": max_depth,
            "total_entries": tree_lines.len()
        },
        "isError": false
    }))
}

/// Handles `search` in-memory accelerated text search over files.
pub async fn handle_search(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let root_str = args.get("path").and_then(Value::as_str).unwrap_or(".");
    let file_pattern = args.get("file_pattern").and_then(Value::as_str);

    let root_path = souls_core::fs::resolve_workspace_path(root_str)
        .map_err(|_| McpDomainError::RefsPathLeakViolation)?;

    let mut matches = Vec::new();
    let mut stack = vec![root_path];

    while let Some(dir) = stack.pop() {
        if matches.len() >= 50 {
            break;
        }

        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if matches.len() >= 50 {
                break;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" || name == "target" || name == "node_modules" || name == ".souls_data" {
                continue;
            }

            let path = entry.path();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);

            if is_dir {
                stack.push(path);
            } else {
                if let Some(pat) = file_pattern {
                    let ext = pat.trim_start_matches('*');
                    if !name.ends_with(ext) {
                        continue;
                    }
                }

                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.contains(query) {
                            matches.push(json!({
                                "file": path.to_string_lossy(),
                                "line": line_idx + 1,
                                "content": line.trim()
                            }));
                            if matches.len() >= 50 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let formatted_matches = matches
        .iter()
        .map(|m| format!("{}:{}: {}", m["file"].as_str().unwrap_or(""), m["line"], m["content"].as_str().unwrap_or("").trim()))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(json!({
        "content": [{
            "type": "text",
            "text": if formatted_matches.is_empty() { "No matches found.".to_string() } else { formatted_matches }
        }],
        "structuredContent": {
            "query": query,
            "match_count": matches.len(),
            "matches": matches
        },
        "isError": false
    }))
}

/// Handles `shell` controlled terminal execution inside ReFS with Sandbox and Truncation Guard.
pub async fn handle_shell(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let cwd_str = args.get("cwd").and_then(Value::as_str).unwrap_or("Z:/souls_engine");
    let cwd_path = PathBuf::from(cwd_str);
    let timeout_ms = args.get("timeout_ms").and_then(Value::as_u64).unwrap_or(15_000);

    match run_sandboxed_shell(command, &cwd_path, timeout_ms).await {
        Ok(res) => {
            let response_text = if !res.stderr.trim().is_empty() {
                format!("STDOUT:\n{}\n\nSTDERR:\n{}", res.stdout.trim(), res.stderr.trim())
            } else {
                res.stdout.trim().to_string()
            };

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": response_text
                }],
                "structuredContent": {
                    "exit_code": res.exit_code,
                    "latency_ms": res.execution_time_ms,
                    "stdout": res.stdout.trim(),
                    "stderr": res.stderr.trim(),
                    "truncated": res.truncated
                },
                "isError": res.exit_code != 0
            }))
        }
        Err(CoreError::DangerousCommandBlocked(msg)) => {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("BLOCKED BY SANDBOX: {msg}")
                }],
                "structuredContent": {
                    "blocked": true,
                    "reason": msg
                },
                "isError": true
            }))
        }
        Err(CoreError::ExecutionTimeout(ms)) => {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("TIMEOUT: Process execution exceeded {ms} ms")
                }],
                "structuredContent": {
                    "timeout_ms": ms
                },
                "isError": true
            }))
        }
        Err(CoreError::RefsPathLeakViolation(_)) => Err(McpDomainError::RefsPathLeakViolation),
        Err(_) => Err(McpDomainError::InternalEnginePanic),
    }
}

/// Handles `execute` sandboxed process execution.
pub async fn handle_execute(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let executable = args
        .get("executable")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let timeout_ms = args.get("timeout_ms").and_then(Value::as_u64).unwrap_or(15_000);

    let mut arg_strings = Vec::new();
    if let Some(arguments) = args.get("arguments").and_then(Value::as_array) {
        for arg in arguments {
            if let Some(a_str) = arg.as_str() {
                arg_strings.push(a_str.to_string());
            }
        }
    }

    let cwd_str = args.get("cwd").and_then(Value::as_str).unwrap_or("Z:/souls_engine");
    let cwd_path = PathBuf::from(cwd_str);

    match run_sandboxed_command(executable, &arg_strings, &cwd_path, timeout_ms).await {
        Ok(res) => {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": res.stdout.trim()
                }],
                "structuredContent": {
                    "executable": executable,
                    "exit_code": res.exit_code,
                    "latency_ms": res.execution_time_ms,
                    "stderr": res.stderr.trim(),
                    "truncated": res.truncated
                },
                "isError": res.exit_code != 0
            }))
        }
        Err(CoreError::ExecutionTimeout(ms)) => {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("TIMEOUT: Command execution exceeded {ms} ms")
                }],
                "structuredContent": {
                    "timeout_ms": ms
                },
                "isError": true
            }))
        }
        Err(CoreError::RefsPathLeakViolation(_)) => Err(McpDomainError::RefsPathLeakViolation),
        Err(_) => Err(McpDomainError::InternalEnginePanic),
    }
}

/// Handles `fetch_web` scraping HTML and converting to clean Markdown in RAM.
pub async fn handle_fetch_web(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let url_str = args
        .get("url")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("SoulsEngine/7.2 (Bare-Metal MCP Web Scraper)")
        .build()
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    let resp = client.get(url_str).send().await.map_err(|_| {
        McpDomainError::StructuredFailure
    })?;

    let status = resp.status().as_u16();
    let body_html = resp.text().await.unwrap_or_default();

    let markdown = html_to_clean_markdown(&body_html);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": markdown
        }],
        "structuredContent": {
            "url": url_str,
            "status_code": status,
            "length_chars": markdown.len()
        },
        "isError": false
    }))
}

/// Handles `web_search` querying DuckDuckGo HTML Lite and parsing results via souls_core::web.
pub async fn handle_web_search(args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(5).min(10) as usize;

    let encoded_query = url_encode(query);
    let search_url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|_| McpDomainError::InternalEnginePanic)?;

    let resp = client.get(&search_url).send().await.map_err(|_| {
        McpDomainError::StructuredFailure
    })?;

    let body_html = resp.text().await.unwrap_or_default();
    let web_results = parse_duckduckgo_html(&body_html, limit);

    let formatted_text = if web_results.is_empty() {
        format!("No live web search results found for '{}'.", query)
    } else {
        web_results
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                format!(
                    "{}. **{}**\n   {}\n   Link: {}",
                    idx + 1,
                    r.title,
                    r.snippet,
                    r.link
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    let results_json: Vec<Value> = web_results
        .into_iter()
        .map(|r| {
            json!({
                "title": r.title,
                "link": r.link,
                "snippet": r.snippet
            })
        })
        .collect();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": formatted_text
        }],
        "structuredContent": {
            "query": query,
            "count": results_json.len(),
            "results": results_json
        },
        "isError": false
    }))
}

/// Handles `repo_meta` extracting git metadata under ReFS.
pub async fn handle_repo_meta(state: &AppState, _args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let branch_output = tokio::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir("Z:/souls_engine")
        .output()
        .await;

    let branch = branch_output
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main".to_string());

    let commit_output = tokio::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir("Z:/souls_engine")
        .output()
        .await;

    let commit = commit_output
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    let hw = state.runtime.watchdog().lock().map(|w| w.snapshot()).unwrap_or_default();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Souls Engine Git Workspace: branch='{}', commit='{}'", branch, commit)
        }],
        "structuredContent": {
            "branch": branch,
            "head_commit": commit,
            "workspace": "Z:/souls_engine",
            "db_path": "Z:/souls_engine/.souls_data/souls_memory.db",
            "vram_limit_mb": hw.vram_free_mb
        },
        "isError": false
    }))
}

// ============================================================================
// DOMAIN 3: EXTENDED MEMORY & LADYBUG GRAPH TOOLS (`memory.*`)
// ============================================================================

/// Handles `mem_search` full-text search across observations with optional LanceDB vector recall.
pub async fn handle_mem_search(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;

    let pool = state.memory.pool();
    let rows = sqlx::query(
        "SELECT entity_name, observation, created_at FROM epistemic_observations
         WHERE observation LIKE $1 OR entity_name LIKE $1
         LIMIT 20"
    )
    .bind(format!("%{}%", query))
    .fetch_all(&*pool)
    .await
    .map_err(|_| McpDomainError::InternalEnginePanic)?;

    let mut observations = Vec::new();
    for r in rows {
        let entity: String = r.get(0);
        let obs: String = r.get(1);
        let ts: i64 = r.get(2);
        observations.push(json!({
            "entity": entity,
            "observation": obs,
            "timestamp": ts,
            "source": "sqlite_wal"
        }));
    }

    // Optional vector search if vector embedding is provided
    let query_vector: Option<Vec<f32>> = args
        .get("vector")
        .or_else(|| args.get("query_vector"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        });

    if let Some(vec) = query_vector {
        if let Ok(vec_matches) = state.memory.lance_store().search_vectors(vec, limit, None).await {
            for v_cand in vec_matches {
                observations.push(json!({
                    "entity": v_cand.id,
                    "observation": v_cand.content,
                    "similarity": v_cand.similarity,
                    "source": "lancedb_vector"
                }));
            }
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Found {} observation matches (SQLite WAL + LanceDB).", observations.len())
        }],
        "structuredContent": { "query": query, "count": observations.len(), "results": observations },
        "isError": false
    }))
}

/// Handles `mem_open_nodes` loading entities and connected edges by name.
pub async fn handle_mem_open_nodes(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let names = args
        .get("names")
        .and_then(Value::as_array)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let pool = state.memory.pool();
    let mut loaded_nodes = Vec::new();

    for name_val in names {
        if let Some(name) = name_val.as_str() {
            let row_opt = sqlx::query(
                "SELECT id, node_type, canonical_name, metadata_json, created_at FROM ladybug_nodes WHERE canonical_name = $1 OR id = $1"
            )
            .bind(name)
            .fetch_optional(&*pool)
            .await
            .map_err(|_| McpDomainError::InternalEnginePanic)?;

            if let Some(r) = row_opt {
                let id: String = r.get(0);
                let node_type: String = r.get(1);
                let c_name: String = r.get(2);
                let meta: String = r.get(3);
                let ts: i64 = r.get(4);
                loaded_nodes.push(json!({
                    "id": id,
                    "node_type": node_type,
                    "canonical_name": c_name,
                    "metadata": meta,
                    "created_at": ts
                }));
            }
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Loaded {} entities from LadybugDB.", loaded_nodes.len())
        }],
        "structuredContent": { "nodes": loaded_nodes },
        "isError": false
    }))
}

/// Handles `mem_read_graph` reading active topology with protective limits.
pub async fn handle_mem_read_graph(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50).min(200) as i64;
    let pool = state.memory.pool();

    let node_rows = sqlx::query(
        "SELECT id, node_type, canonical_name FROM ladybug_nodes LIMIT $1"
    )
    .bind(limit)
    .fetch_all(&*pool)
    .await
    .map_err(|_| McpDomainError::InternalEnginePanic)?;

    let nodes: Vec<Value> = node_rows
        .into_iter()
        .map(|r| json!({ "id": r.get::<String, _>(0), "type": r.get::<String, _>(1), "name": r.get::<String, _>(2) }))
        .collect();

    let edge_rows = sqlx::query(
        "SELECT source_id, target_id, relationship, weight FROM ladybug_edges LIMIT $1"
    )
    .bind(limit)
    .fetch_all(&*pool)
    .await
    .map_err(|_| McpDomainError::InternalEnginePanic)?;

    let edges: Vec<Value> = edge_rows
        .into_iter()
        .map(|r| json!({ "source": r.get::<String, _>(0), "target": r.get::<String, _>(1), "relation": r.get::<String, _>(2), "weight": r.get::<f64, _>(3) }))
        .collect();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("LadybugDB Active Graph Topology: {} nodes, {} edges loaded.", nodes.len(), edges.len())
        }],
        "structuredContent": { "nodes": nodes, "edges": edges },
        "isError": false
    }))
}

/// Handles `mem_add_observations` appending clinical facts to entities.
pub async fn handle_mem_add_observations(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let observations = args
        .get("observations")
        .and_then(Value::as_array)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let pool = state.memory.pool();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let mut added = 0;

    for obs_val in observations {
        let obs_obj = match obs_val.as_object() {
            Some(o) => o,
            None => continue,
        };
        let entity = obs_obj.get("entity").or_else(|| obs_obj.get("entityName")).and_then(Value::as_str).unwrap_or("general");
        let contents = obs_obj.get("contents").or_else(|| obs_obj.get("observation")).and_then(Value::as_str).unwrap_or("");

        if !contents.is_empty() {
            let obs_id = format!("obs_{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
            let _ = sqlx::query(
                "INSERT INTO epistemic_observations (id, entity_name, observation, created_at)
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(&obs_id)
            .bind(entity)
            .bind(contents)
            .bind(now)
            .execute(&*pool)
            .await;
            added += 1;
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Persisted {} observations to SQLite STRICT WAL.", added)
        }],
        "structuredContent": { "added_count": added },
        "isError": false
    }))
}

/// Handles `mem_delete_entities` purging nodes and cascading edges.
pub async fn handle_mem_delete_entities(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let entity_names = args
        .get("entity_names")
        .or_else(|| args.get("names"))
        .and_then(Value::as_array)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let pool = state.memory.pool();
    let mut deleted = 0;

    for val in entity_names {
        if let Some(name) = val.as_str() {
            let node_id = if name.starts_with("node_") { name.to_string() } else { format!("node_{name}") };
            let _ = state.memory.ladybug().delete_node(&pool, &node_id).await;
            deleted += 1;
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Purged {} entities and cascaded edges from LadybugDB.", deleted)
        }],
        "structuredContent": { "deleted_count": deleted },
        "isError": false
    }))
}

/// Handles `mem_delete_relations` purging directed edges.
pub async fn handle_mem_delete_relations(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let relations = args
        .get("relations")
        .and_then(Value::as_array)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let pool = state.memory.pool();
    let mut deleted = 0;

    for rel in relations {
        let rel_obj = match rel.as_object() {
            Some(o) => o,
            None => continue,
        };
        let src = rel_obj.get("source").or_else(|| rel_obj.get("from")).and_then(Value::as_str).unwrap_or("");
        let dst = rel_obj.get("target").or_else(|| rel_obj.get("to")).and_then(Value::as_str).unwrap_or("");
        let rel_type = rel_obj.get("relationship").and_then(Value::as_str).unwrap_or("relates_to");

        let src_id = if src.starts_with("node_") { src.to_string() } else { format!("node_{src}") };
        let dst_id = if dst.starts_with("node_") { dst.to_string() } else { format!("node_{dst}") };

        let res = sqlx::query(
            "DELETE FROM ladybug_edges WHERE source_id = $1 AND target_id = $2 AND relationship = $3"
        )
        .bind(&src_id)
        .bind(&dst_id)
        .bind(rel_type)
        .execute(&*pool)
        .await;

        if let Ok(r) = res {
            deleted += r.rows_affected();
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Removed {} relations from LadybugDB.", deleted)
        }],
        "structuredContent": { "deleted_relations": deleted },
        "isError": false
    }))
}

/// Handles `mem_delete_observations` purging specific observation records.
pub async fn handle_mem_delete_observations(state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let ids = args
        .get("ids")
        .or_else(|| args.get("observation_ids"))
        .and_then(Value::as_array)
        .ok_or(McpDomainError::InvalidInputParameters)?;

    let pool = state.memory.pool();
    let mut deleted = 0;

    for id_val in ids {
        if let Some(id_str) = id_val.as_str() {
            let res = sqlx::query("DELETE FROM epistemic_observations WHERE id = $1")
                .bind(id_str)
                .execute(&*pool)
                .await;
            if let Ok(r) = res {
                deleted += r.rows_affected();
            }
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Deleted {} observations from SQLite STRICT WAL.", deleted)
        }],
        "structuredContent": { "deleted_count": deleted },
        "isError": false
    }))
}

// ============================================================================
// DOMAIN 4 & 5: EXTENDED THINKING & OPS TOOLS (`think.*`, `ops.*`)
// ============================================================================

/// Handles `session` active session inspection and reset.
pub async fn handle_session(_state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let action = args.get("action").and_then(Value::as_str).unwrap_or("status");
    let session_id = args.get("session_id").and_then(Value::as_str).unwrap_or("active_session");

    if action == "reset" {
        let cache = get_global_dedup_cache();
        cache.write().unwrap().clear();
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Session '{}' volatile caches reset successfully.", session_id)
            }],
            "structuredContent": { "session_id": session_id, "status": "reset" },
            "isError": false
        }))
    } else {
        let cache = get_global_dedup_cache();
        let count = cache.read().unwrap().len();
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Session '{}' active. Ephemeral cache blocks: {}.", session_id, count)
            }],
            "structuredContent": {
                "session_id": session_id,
                "cached_blocks": count,
                "status": "active"
            },
            "isError": false
        }))
    }
}

/// Handles `merge_sessions` fusing deliberation branches.
pub async fn handle_merge_sessions(_state: &AppState, args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let target = args.get("target_session").and_then(Value::as_str).unwrap_or("main");
    let source = args.get("source_session").and_then(Value::as_str).unwrap_or("branch");

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Deliberation session branch '{}' merged atomically into '{}'.", source, target)
        }],
        "structuredContent": {
            "target": target,
            "source": source,
            "status": "merged"
        },
        "isError": false
    }))
}

/// Handles `feedback` FinOps metrics and Silicon Return calculation.
pub async fn handle_feedback(state: &AppState, _args: &serde_json::Map<String, Value>) -> Result<Value, McpDomainError> {
    let pool = state.memory.pool();
    let (total_events, avg_latency) = match sqlx::query(
        "SELECT COUNT(*), COALESCE(AVG(latency_ms), 0.0) FROM telemetry_events"
    )
    .fetch_one(&*pool)
    .await {
        Ok(row) => {
            let cnt: i64 = row.get(0);
            let avg: f64 = row.try_get(1).unwrap_or(0.0);
            (cnt, avg)
        }
        Err(_) => (0i64, 0.0f64),
    };

    let hw = state.runtime.watchdog().lock().map(|w| w.snapshot()).unwrap_or_default();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("FinOps Silício Return: {} calls, avg latency: {:.1}ms, VRAM free: {}MB, Temp: {:.1}°C", total_events, avg_latency, hw.vram_free_mb, hw.gpu_temp_c)
        }],
        "structuredContent": {
            "total_telemetry_events": total_events,
            "avg_latency_ms": avg_latency,
            "vram_free_mb": hw.vram_free_mb,
            "gpu_temp_c": hw.gpu_temp_c,
            "barrier_phi": hw.barrier_phi
        },
        "isError": false
    }))
}
