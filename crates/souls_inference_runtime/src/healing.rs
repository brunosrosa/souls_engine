//! Deterministic stack-based JSON response healing pipeline.
//!
//! Repairs truncated, cut-off, or malformed JSON payloads in flight:
//! - Strips markdown code blocks (` ```json ` fences).
//! - Strips leading and trailing non-JSON noise.
//! - Closes unterminated string literals.
//! - Eliminates trailing commas before closing braces/brackets.
//! - Automatically closes unclosed arrays and objects using a deterministic LIFO delimiter stack.
//! - Corrects truncated booleans/null literals (`tru`, `fals`, `nul`).

use serde_json::Value;
use crate::error::InferenceError;

/// Heals and parses a raw LLM output into a valid `serde_json::Value`.
pub fn heal_json_response(raw: &str) -> Result<Value, InferenceError> {
    let unescaped = strip_ansi_escapes(raw);
    let cleaned_input = unescaped.replace("[end of text]", "").replace("[start of text]", "");
    let trimmed = cleaned_input.trim();
    if trimmed.is_empty() {
        return Err(InferenceError::JsonHealingError("Empty input".to_string()));
    }

    // 1. Direct parse attempt (fast path)
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return Ok(v);
    }

    // 2. Strip Markdown code fences if present
    let unwrapped = strip_markdown_fences(trimmed);

    // 3. Extract candidate JSON substring starting at first '{' or '['
    let start_idx = match unwrapped.find(['{', '[']) {
        Some(idx) => idx,
        None => {
            return Err(InferenceError::JsonHealingError(
                "No JSON object or array delimiter found".to_string(),
            ));
        }
    };
    let candidate = &unwrapped[start_idx..];

    // 4. Try parsing extracted candidate directly
    if let Ok(v) = serde_json::from_str::<Value>(candidate) {
        return Ok(v);
    }

    // 5. Try extracting balanced JSON boundary if conversational trailers follow
    if let Some(balanced) = find_balanced_json_slice(candidate) {
        if let Ok(v) = serde_json::from_str::<Value>(balanced) {
            return Ok(v);
        }
        let cleaned = remove_trailing_commas(balanced);
        let healed = apply_stack_healing(&cleaned);
        if let Ok(v) = serde_json::from_str::<Value>(&healed) {
            return Ok(v);
        }
    }

    let cleaned = remove_trailing_commas(candidate);
    let healed_str = apply_stack_healing(&cleaned);

    // 6. Parse repaired string
    serde_json::from_str::<Value>(&healed_str).map_err(|e| {
        InferenceError::JsonHealingError(format!(
            "Healing failed to produce valid JSON: {}. Repaired content: '{}'",
            e, healed_str
        ))
    })
}

/// Identifies the first cleanly balanced JSON object or array slice.
fn find_balanced_json_slice(input: &str) -> Option<&str> {
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut started = false;

    for (idx, c) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => in_string = true,
                '{' => {
                    started = true;
                    stack.push('}');
                }
                '[' => {
                    started = true;
                    stack.push(']');
                }
                '}' | ']' => {
                    if let Some(expected) = stack.pop() {
                        if expected == c && stack.is_empty() && started {
                            let end_idx = idx + c.len_utf8();
                            return Some(&input[..end_idx]);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Strips markdown code fences (```json ... ``` or ``` ... ```).
fn strip_markdown_fences(input: &str) -> &str {
    let trimmed = input.trim();
    let first_json = trimmed.find(['{', '[']);
    let first_fence = trimmed.find("```");

    let mut s = trimmed;
    if let Some(fence_idx) = first_fence {
        if first_json.map_or(true, |j| fence_idx < j) {
            // Fence is genuinely before the JSON start
            let after_ticks = &s[fence_idx + 3..];
            let content_start = if let Some(newline_pos) = after_ticks.find('\n') {
                newline_pos + 1
            } else {
                0
            };
            s = &after_ticks[content_start..];
        }
    }

    if let Some(end) = s.rfind("```") {
        s = &s[..end];
    }

    s.trim()
}

/// Removes internal and trailing commas before closing brackets or braces.
fn remove_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            out.push(c);
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if c == ',' {
            // Check next non-whitespace character
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // Skip trailing comma
            } else if j >= chars.len() {
                // Trailing comma at the end of input - skip
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

/// Deterministically heals broken or truncated JSON.
fn apply_stack_healing(input: &str) -> String {
    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut chars: Vec<char> = input.chars().collect();

    // Check if ends with trailing unescaped backslash
    if let Some(&'\\') = chars.last() {
        chars.pop();
    }

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => in_string = true,
                '{' => stack.push('}'),
                '[' => stack.push(']'),
                '}' => {
                    if let Some(&expected) = stack.last() {
                        if expected == '}' {
                            stack.pop();
                        }
                    }
                }
                ']' => {
                    if let Some(&expected) = stack.last() {
                        if expected == ']' {
                            stack.pop();
                        }
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    let mut repaired = chars.into_iter().collect::<String>();

    // If still in an open string literal, close it
    if in_string {
        repaired.push('"');
    }

    // Clean up trailing commas and incomplete literals at the cut-off boundary
    repaired = clean_boundary(repaired);

    // Close all open delimiters in LIFO order
    while let Some(closing_delimiter) = stack.pop() {
        repaired.push(closing_delimiter);
    }

    repaired
}

/// Cleans up trailing commas, colons, or incomplete tokens before closing structures.
fn clean_boundary(s: String) -> String {
    let mut trimmed = s.trim_end().to_string();

    // Fix truncated booleans/nulls
    if trimmed.ends_with(": tru") || trimmed.ends_with(": fals") {
        trimmed.push('e');
    } else if trimmed.ends_with(": nul") {
        trimmed.push('l');
    } else if trimmed.ends_with(':') {
        // Truncated key without value -> supply null
        trimmed.push_str(" null");
    }

    // Remove any trailing comma
    while trimmed.ends_with(',') || trimmed.ends_with(", ") {
        let idx = trimmed.rfind(',').unwrap_or(trimmed.len());
        trimmed.truncate(idx);
        trimmed = trimmed.trim_end().to_string();
    }

    trimmed
}

/// Strips ANSI terminal escape codes (e.g. \x1b[0m or \x1b[32m).
pub fn strip_ansi_escapes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                while let Some(&next_c) = chars.peek() {
                    chars.next();
                    if next_c.is_ascii_alphabetic() || next_c == 'm' {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heal_valid_json_untouched() {
        let raw = r#"{"status": "ok", "count": 42}"#;
        let val = heal_json_response(raw).expect("Valid JSON failed");
        assert_eq!(val["status"], "ok");
        assert_eq!(val["count"], 42);
    }

    #[test]
    fn test_heal_markdown_fenced_json() {
        let raw = "Here is the response:\n```json\n{\"ok\": true, \"items\": [1, 2, 3]}\n```\nHope that helps!";
        let val = heal_json_response(raw).expect("Markdown fenced JSON failed");
        assert_eq!(val["ok"], true);
        assert_eq!(val["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_heal_truncated_object() {
        let raw = r#"{"name": "souls_engine", "version": "v7", "status": "act"#;
        let val = heal_json_response(raw).expect("Truncated object healing failed");
        assert_eq!(val["name"], "souls_engine");
        assert_eq!(val["version"], "v7");
        assert_eq!(val["status"], "act");
    }

    #[test]
    fn test_heal_truncated_nested_arrays_and_objects() {
        let raw = r#"{"matrix": [{"id": 1, "tokens": ["tok1", "tok2"#;
        let val = heal_json_response(raw).expect("Nested truncation healing failed");
        assert_eq!(val["matrix"][0]["id"], 1);
        assert_eq!(val["matrix"][0]["tokens"][0], "tok1");
        assert_eq!(val["matrix"][0]["tokens"][1], "tok2");
    }

    #[test]
    fn test_heal_trailing_comma() {
        let raw = r#"{"items": [1, 2, 3,], "active": true,}"#;
        let val = heal_json_response(raw).expect("Trailing comma healing failed");
        assert_eq!(val["active"], true);
    }

    #[test]
    fn test_heal_trailing_colon() {
        let raw = r#"{"id": 100, "pending_key":"#;
        let val = heal_json_response(raw).expect("Trailing colon healing failed");
        assert_eq!(val["id"], 100);
        assert!(val["pending_key"].is_null());
    }

    #[test]
    fn test_heal_conversational_trailer() {
        let raw = "{\n  \"action\": \"refactor\",\n  \"target\": \"crates/souls_ast\",\n  \"confidence\": 0.98\n}\n```\n\nThis is a valid JSON object.";
        let val = heal_json_response(raw).expect("Conversational trailer healing failed");
        assert_eq!(val["action"], "refactor");
        assert_eq!(val["target"], "crates/souls_ast");
        assert_eq!(val["confidence"], 0.98);
    }

    #[test]
    fn test_heal_ansi_escapes_and_eos_token() {
        let raw = "\x1b[0m\n\n{\n  \"action\": \"refactor\",\n  \"target\": \"crates/souls_ast\",\n  \"confidence\": 0.98\n} [end of text]";
        let val = heal_json_response(raw).expect("ANSI escapes and EOS token healing failed");
        assert_eq!(val["action"], "refactor");
        assert_eq!(val["target"], "crates/souls_ast");
        assert_eq!(val["confidence"], 0.98);
    }
}
