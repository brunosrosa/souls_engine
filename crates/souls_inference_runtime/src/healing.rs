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
    let trimmed = raw.trim();
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

/// Strips markdown code fences (```json ... ``` or ``` ... ```).
fn strip_markdown_fences(input: &str) -> &str {
    let mut s = input;
    if let Some(start) = s.find("```") {
        let after_ticks = &s[start + 3..];
        // Skip language tag (e.g. "json\n")
        let content_start = if let Some(newline_pos) = after_ticks.find('\n') {
            newline_pos + 1
        } else {
            0
        };
        s = &after_ticks[content_start..];
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
}
