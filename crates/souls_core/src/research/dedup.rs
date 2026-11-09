//! # Deterministic SipHash URL & Content Deduplication (`souls_core::research::dedup`)
//!
//! Provides canonical URL normalization (stripping tracking parameters, schemes, and anchors)
//! and rapid SipHash fingerprinting to eliminate duplicate citations and mirrored web content.

use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Set of known tracking query parameter prefixes and names to purge.
const PURGE_QUERY_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "fbclid",
    "gclid",
    "ref",
    "source",
    "ref_src",
    "igshid",
    "mc_cid",
    "mc_eid",
    "_hsenc",
    "_hsmi",
];

/// Normalizes arbitrary raw web URL into a canonical deduplication key.
///
/// Strips:
/// - Protocols (`http://`, `https://`)
/// - `www.` prefix
/// - Anchor fragments (`#heading`)
/// - Marketing tracking parameters (`utm_*`, `fbclid`, `gclid`, etc.)
/// - Trailing slashes
#[must_use]
pub fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 1. Remove scheme
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);

    // 2. Remove anchor fragment
    let without_anchor = without_scheme.split('#').next().unwrap_or(without_scheme);

    // 3. Separate host/path from query string
    let (base, query_opt) = match without_anchor.find('?') {
        Some(pos) => (&without_anchor[..pos], Some(&without_anchor[pos + 1..])),
        None => (without_anchor, None),
    };

    // 4. Normalize base: lowercase host, strip www., strip trailing slash
    let mut parts = base.splitn(2, '/');
    let host = parts.next().unwrap_or("").to_lowercase();
    let host_clean = host.strip_prefix("www.").unwrap_or(&host);

    let path = parts.next().unwrap_or("");
    let path_clean = path.trim_end_matches('/');

    let mut canonical = if path_clean.is_empty() {
        host_clean.to_string()
    } else {
        format!("{host_clean}/{path_clean}")
    };

    // 5. Clean query parameters
    if let Some(query) = query_opt {
        let mut clean_params = Vec::new();
        for param in query.split('&') {
            if param.is_empty() {
                continue;
            }
            let key = param.split('=').next().unwrap_or("").to_lowercase();
            if !PURGE_QUERY_PARAMS.contains(&key.as_str()) {
                clean_params.push(param);
            }
        }
        if !clean_params.is_empty() {
            canonical.push('?');
            canonical.push_str(&clean_params.join("&"));
        }
    }

    canonical
}

/// Computes a deterministic 64-bit SipHash for a normalized URL.
#[inline]
#[must_use]
pub fn hash_url(raw_url: &str) -> u64 {
    let norm = normalize_url(raw_url);
    let mut hasher = DefaultHasher::new();
    norm.hash(&mut hasher);
    hasher.finish()
}

/// Computes an invariant content fingerprint by hashing normalized text.
#[must_use]
pub fn compute_content_fingerprint(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    for word in text.split_whitespace() {
        word.to_lowercase().hash(&mut hasher);
    }
    hasher.finish()
}

/// In-memory deduplication registry tracking processed URLs and passage content hashes.
#[derive(Debug, Clone, Default)]
pub struct ResearchDedupRegistry {
    seen_urls: HashSet<u64>,
    seen_fingerprints: HashSet<u64>,
}

impl ResearchDedupRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if a URL has already been processed without registering it.
    #[must_use]
    pub fn is_url_seen(&self, raw_url: &str) -> bool {
        let h = hash_url(raw_url);
        self.seen_urls.contains(&h)
    }

    /// Registers a URL. Returns `true` if newly registered, `false` if already seen (duplicate).
    pub fn register_url(&mut self, raw_url: &str) -> bool {
        let h = hash_url(raw_url);
        self.seen_urls.insert(h)
    }

    /// Registers a content fingerprint. Returns `true` if new, `false` if duplicate.
    pub fn register_content(&mut self, text: &str) -> bool {
        let fp = compute_content_fingerprint(text);
        self.seen_fingerprints.insert(fp)
    }

    /// Number of unique URLs tracked.
    #[must_use]
    pub fn len_urls(&self) -> usize {
        self.seen_urls.len()
    }

    /// Number of unique content fingerprints tracked.
    #[must_use]
    pub fn len_contents(&self) -> usize {
        self.seen_fingerprints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url_purges_tracking_and_anchors() {
        let u1 = "https://www.docs.rs/tokio/1.0/index.html?utm_source=twitter&utm_medium=cpc#io";
        let u2 = "http://docs.rs/tokio/1.0/index.html";
        assert_eq!(normalize_url(u1), "docs.rs/tokio/1.0/index.html");
        assert_eq!(normalize_url(u1), normalize_url(u2));
        assert_eq!(hash_url(u1), hash_url(u2));
    }

    #[test]
    fn test_dedup_registry_lifecycle() {
        let mut reg = ResearchDedupRegistry::new();
        let u1 = "https://github.com/tokio-rs/tokio?utm_content=readme";
        let u2 = "http://www.github.com/tokio-rs/tokio/";

        assert!(reg.register_url(u1));
        assert!(!reg.register_url(u2)); // Same canonical URL
        assert_eq!(reg.len_urls(), 1);

        let c1 = "O Tokio opera sob I/O assíncrono nativo.";
        let c2 = "  o tokio   opera SOB i/o assíncrono   nativo.  ";
        assert!(reg.register_content(c1));
        assert!(!reg.register_content(c2)); // Invariant content fingerprint
        assert_eq!(reg.len_contents(), 1);
    }
}

