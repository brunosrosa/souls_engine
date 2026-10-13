//! Linear PII Redactor with reversible opaque token mapping for Souls Engine (v7).
//!
//! Sanitizes inbound prompt bodies and text buffers before upstream cloud dispatch,
//! mapping sensitive paths, tokens, and credentials into reversible opaque tokens
//! formatted as `__SOULS_PII_<id>__`.
//!
//! Conforms to CRATES_SPECIFICATION.md Section 2.2 and ADR-041.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

/// Canonical patterns recognized for PII detection.
pub const CANONICAL_PII_PATTERNS: &[&str] = &[
    "Bearer ",
    "bearer ",
    "api_key:",
    "api-key:",
    "apikey:",
    "token:",
    "secret:",
    "password:",
    "API_KEY=",
    "TOKEN=",
    "C:\\Users\\",
    "C:\\Program Files\\",
    "D:\\",
    "/home/",
    "/root/",
    "/etc/passwd",
    "/var/log/",
    ".ssh/id_rsa",
    ".aws/credentials",
    ".env",
];

/// Redactor engine managing reversible tokens.
#[derive(Clone)]
pub struct PiiRedactor {
    token_store: Arc<RwLock<HashMap<u64, String>>>,
    enabled: bool,
}

impl PiiRedactor {
    /// Creates a new redactor with the specified enablement status.
    /// By default, PII redaction is disabled unless explicitly activated.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            token_store: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }

    /// Checks if redaction is currently active.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the active redaction status.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Redacts raw input bytes.
    ///
    /// Fail-soft: if disabled or invalid UTF-8, returns the original input unchanged.
    pub fn redact(&self, body: &[u8]) -> Vec<u8> {
        if !self.enabled {
            return body.to_vec();
        }

        let text = match std::str::from_utf8(body) {
            Ok(s) => s,
            Err(_) => return body.to_vec(),
        };

        let mut out = String::with_capacity(text.len());
        let mut i = 0;
        let bytes = text.as_bytes();
        let len = bytes.len();

        while i < len {
            if let Some((pattern_len, is_prefix)) = match_pattern_at(&text[i..]) {
                let start = i;
                let end = if is_prefix {
                    expand_match_boundary(text, start, start + pattern_len)
                } else {
                    start + pattern_len
                };

                let original = &text[start..end];
                let token_id = hash_pii_token(original);
                let token = format!("__SOULS_PII_{:016x}__", token_id);

                if let Ok(mut store) = self.token_store.write() {
                    store.entry(token_id).or_insert_with(|| original.to_string());
                }

                out.push_str(&token);
                i = end;
            } else {
                // Advance to next char boundary safely
                let next_char = text[i..].chars().next().map_or(1, |c| c.len_utf8());
                out.push_str(&text[i..i + next_char]);
                i += next_char;
            }
        }

        out.into_bytes()
    }

    /// Reverses an opaque token back to its original value if present in the token store.
    #[must_use]
    pub fn reverse(&self, token: &str) -> Option<String> {
        let id_str = token
            .strip_prefix("__SOULS_PII_")?
            .strip_suffix("__")?;
        let id = u64::from_str_radix(id_str, 16).ok()?;

        let store = self.token_store.read().ok()?;
        store.get(&id).cloned()
    }

    /// Replaces all `__SOULS_PII_<id>__` tokens in the provided text with their original values.
    #[must_use]
    pub fn unredact_text(&self, text: &str) -> String {
        let store = match self.token_store.read() {
            Ok(s) => s,
            Err(_) => return text.to_string(),
        };

        let mut result = text.to_string();
        for (id, original) in store.iter() {
            let token = format!("__SOULS_PII_{:016x}__", id);
            result = result.replace(&token, original);
        }
        result
    }

    /// Number of distinct tokens currently tracked in memory.
    #[must_use]
    pub fn token_count(&self) -> usize {
        self.token_store.read().map(|m| m.len()).unwrap_or(0)
    }
}

impl Default for PiiRedactor {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Matches any canonical pattern at the prefix of slice.
/// Returns (matched_length, needs_expansion_flag).
fn match_pattern_at(slice: &str) -> Option<(usize, bool)> {
    for &pattern in CANONICAL_PII_PATTERNS {
        if slice.len() >= pattern.len() {
            let candidate = &slice[..pattern.len()];
            if candidate.eq_ignore_ascii_case(pattern) {
                // If it ends with separator or whitespace, it's a prefix to expand
                let ends_with_sep = pattern.ends_with(' ')
                    || pattern.ends_with(':')
                    || pattern.ends_with('=')
                    || pattern.ends_with('\\')
                    || pattern.ends_with('/');
                return Some((pattern.len(), ends_with_sep));
            }
        }
    }
    None
}

/// Expands match boundary to capture associated value (tokens, paths, keys).
fn expand_match_boundary(text: &str, _start: usize, end: usize) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    if end >= len {
        return end;
    }

    let mut i = end;
    // Consume optional whitespace
    while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }

    // Read characters until terminal delimiter (space, quotes, comma, closing braces, newline)
    while i < len {
        let c = bytes[i];
        if c == b' ' || c == b'"' || c == b'\'' || c == b',' || c == b'\n' || c == b'\r' || c == b'}' || c == b']' {
            break;
        }
        i += 1;
    }

    i
}

/// FNV-1a 64-bit hash for deterministic token identifiers.
fn hash_pii_token(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled_returns_input_unchanged() {
        let r = PiiRedactor::new(false);
        let body = br#"{"api_key": "sk-abc123def456ghi789jkl"}"#;
        let out = r.redact(body);
        assert_eq!(out, body, "Disabled redactor must return unmodified body");
    }

    #[test]
    fn test_enabled_redacts_bearer_token() {
        let mut r = PiiRedactor::new(false);
        r.set_enabled(true);
        let body = br#"{"auth": "Bearer abc123def456ghi789jklmno"}"#;
        let out = r.redact(body);
        let out_str = std::str::from_utf8(&out).unwrap();

        assert!(out_str.contains("__SOULS_PII_"));
        assert!(!out_str.contains("abc123def456ghi789jklmno"));

        // Reverse check via 30-char token (__SOULS_PII_ [12] + hash [16] + __ [2])
        let token_start = out_str.find("__SOULS_PII_").unwrap();
        let token = &out_str[token_start..token_start + 30];
        let reversed = r.reverse(token);
        assert!(reversed.is_some());
        assert!(reversed.unwrap().contains("abc123def456ghi789jklmno"));

        // Full unredact_text roundtrip
        let rehydrated = r.unredact_text(out_str);
        assert!(rehydrated.contains("Bearer abc123def456ghi789jklmno"));
    }

    #[test]
    fn test_enabled_redacts_windows_path() {
        let r = PiiRedactor::new(true);
        let body = b"{\"path\": \"C:\\Users\\alice\\Documents\\secret.txt\"}";
        let out = r.redact(body);
        let out_str = std::str::from_utf8(&out).unwrap();

        assert!(out_str.contains("__SOULS_PII_"));
        assert!(!out_str.contains("secret.txt"));
    }

    #[test]
    fn test_token_idempotency() {
        let r = PiiRedactor::new(true);
        let body1 = br#"Bearer token123"#;
        let body2 = br#"Bearer token123"#;

        let out1 = r.redact(body1);
        let out2 = r.redact(body2);

        assert_eq!(out1, out2);
        assert_eq!(r.token_count(), 1);
    }
}
