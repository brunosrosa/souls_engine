//! Semantic Myers Diff implementation using `similar`.
//!
//! Provides deterministic minimal diff computation with mandatory CRLF -> LF
//! sanitization and closure invariant guarantees.

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::error::AstError;

/// Statistical summary of a computed Myers diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyersDiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: bool,
}

/// Classification of an individual diff chunk line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffTag {
    Insert,
    Delete,
    Equal,
}

/// Detailed line change entry with 1-indexed coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffChange {
    pub tag: DiffTag,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub line_no: Option<usize>,
    pub text: String,
}

/// Structured Myers patch container conforming to `CRATES_SPECIFICATION.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyersPatch {
    pub diff_text: String,
    pub stats: MyersDiffStats,
    pub changes: Vec<DiffChange>,
}

/// Sanitizes text by converting all CRLF (`\r\n`) and isolated CR (`\r`) to LF (`\n`).
#[inline]
pub fn sanitize_lf(input: &str) -> String {
    if !input.contains('\r') {
        return input.to_string();
    }
    let mut clean = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            clean.push('\n');
        } else {
            clean.push(ch);
        }
    }
    clean
}

/// Computes structured Myers difference between `old_content` and `new_content`.
///
/// Both inputs are automatically sanitized to canonical LF prior to computation.
pub fn compute_safe_myers_diff(old_content: &str, new_content: &str) -> Result<MyersPatch, AstError> {
    let clean_old = sanitize_lf(old_content);
    let clean_new = sanitize_lf(new_content);

    if clean_old == clean_new {
        return Ok(MyersPatch {
            diff_text: "(no changes)".to_string(),
            stats: MyersDiffStats {
                additions: 0,
                deletions: 0,
                unchanged: true,
            },
            changes: Vec::new(),
        });
    }

    let diff = TextDiff::from_lines(&clean_old, &clean_new);
    let mut formatted_lines: Vec<String> = Vec::new();
    let mut structured_changes: Vec<DiffChange> = Vec::new();
    let mut additions = 0usize;
    let mut deletions = 0usize;

    for change in diff.iter_all_changes() {
        let old_line = change.old_index().map(|i| i + 1);
        let new_line = change.new_index().map(|i| i + 1);
        let line_no = new_line.or(old_line);
        let text = change.value().trim_end_matches('\n').to_string();

        match change.tag() {
            ChangeTag::Insert => {
                additions += 1;
                if let Some(n) = line_no {
                    formatted_lines.push(format!("+{n}: {text}"));
                }
                structured_changes.push(DiffChange {
                    tag: DiffTag::Insert,
                    old_line,
                    new_line,
                    line_no,
                    text,
                });
            }
            ChangeTag::Delete => {
                deletions += 1;
                if let Some(n) = line_no {
                    formatted_lines.push(format!("-{n}: {text}"));
                }
                structured_changes.push(DiffChange {
                    tag: DiffTag::Delete,
                    old_line,
                    new_line,
                    line_no,
                    text,
                });
            }
            ChangeTag::Equal => {
                structured_changes.push(DiffChange {
                    tag: DiffTag::Equal,
                    old_line,
                    new_line,
                    line_no,
                    text,
                });
            }
        }
    }

    if additions == 0 && deletions == 0 {
        return Ok(MyersPatch {
            diff_text: "(no changes)".to_string(),
            stats: MyersDiffStats {
                additions: 0,
                deletions: 0,
                unchanged: true,
            },
            changes: Vec::new(),
        });
    }

    formatted_lines.push(format!("\ndiff +{additions}/-{deletions} lines"));
    let diff_text = formatted_lines.join("\n");

    Ok(MyersPatch {
        diff_text,
        stats: MyersDiffStats {
            additions,
            deletions,
            unchanged: false,
        },
        changes: structured_changes,
    })
}

/// Convenience function returning human-readable diff string.
pub fn myers_diff(before: &str, after: &str) -> Result<String, AstError> {
    compute_safe_myers_diff(before, after).map(|patch| patch.diff_text)
}

/// Convenience function returning formatted diff string along with statistics.
pub fn myers_diff_with_stats(before: &str, after: &str) -> Result<(String, MyersDiffStats), AstError> {
    compute_safe_myers_diff(before, after).map(|patch| (patch.diff_text, patch.stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myers_safe_closure_identical_content() {
        let text = "alpha\nbeta\ngamma\n";
        let patch = compute_safe_myers_diff(text, text).expect("diff computation must succeed");
        assert_eq!(patch.diff_text, "(no changes)");
        assert!(patch.stats.unchanged);
        assert_eq!(patch.stats.additions, 0);
        assert_eq!(patch.stats.deletions, 0);
        assert!(patch.changes.is_empty());
    }

    #[test]
    fn test_myers_safe_closure_crlf_normalization() {
        let lf_text = "fn main() {\n    println!(\"hello\");\n}\n";
        let crlf_text = "fn main() {\r\n    println!(\"hello\");\r\n}\r\n";
        let patch = compute_safe_myers_diff(lf_text, crlf_text).expect("diff computation must succeed");
        assert_eq!(patch.diff_text, "(no changes)");
        assert!(patch.stats.unchanged);
    }

    #[test]
    fn test_myers_safe_closure_single_insertion() {
        let before = "line 1\nline 3\n";
        let after = "line 1\nline 2 (inserted)\nline 3\n";

        let patch = compute_safe_myers_diff(before, after).expect("diff computation must succeed");
        assert!(!patch.stats.unchanged);
        assert_eq!(patch.stats.additions, 1);
        assert_eq!(patch.stats.deletions, 0);
        assert!(patch.diff_text.contains("+2: line 2 (inserted)"));
        assert!(patch.diff_text.contains("diff +1/-0 lines"));

        let insert_change = patch
            .changes
            .iter()
            .find(|c| c.tag == DiffTag::Insert)
            .expect("should have insert change");
        assert_eq!(insert_change.new_line, Some(2));
        assert_eq!(insert_change.text, "line 2 (inserted)");
    }

    #[test]
    fn test_myers_safe_closure_single_deletion() {
        let before = "first\nsecond\nthird\n";
        let after = "first\nthird\n";

        let patch = compute_safe_myers_diff(before, after).expect("diff computation must succeed");
        assert!(!patch.stats.unchanged);
        assert_eq!(patch.stats.additions, 0);
        assert_eq!(patch.stats.deletions, 1);
        assert!(patch.diff_text.contains("-2: second"));
        assert!(patch.diff_text.contains("diff +0/-1 lines"));

        let del_change = patch
            .changes
            .iter()
            .find(|c| c.tag == DiffTag::Delete)
            .expect("should have delete change");
        assert_eq!(del_change.old_line, Some(2));
        assert_eq!(del_change.text, "second");
    }

    #[test]
    fn test_myers_safe_closure_coordinate_precision() {
        let before = "a\nb\nc\nd\n";
        let after = "a\nB_MODIFIED\nc\nd\ne_ADDED\n";

        let patch = compute_safe_myers_diff(before, after).expect("diff computation must succeed");
        assert_eq!(patch.stats.additions, 2);
        assert_eq!(patch.stats.deletions, 1);

        // Verification of changed line coordinates
        let del = patch.changes.iter().find(|c| c.tag == DiffTag::Delete).unwrap();
        assert_eq!(del.old_line, Some(2));
        assert_eq!(del.text, "b");

        let inserts: Vec<_> = patch.changes.iter().filter(|c| c.tag == DiffTag::Insert).collect();
        assert_eq!(inserts.len(), 2);
        assert_eq!(inserts[0].new_line, Some(2));
        assert_eq!(inserts[0].text, "B_MODIFIED");
        assert_eq!(inserts[1].new_line, Some(5));
        assert_eq!(inserts[1].text, "e_ADDED");
    }

    #[test]
    fn test_myers_safe_closure_empty_boundary_conditions() {
        let (empty_diff, empty_stats) = myers_diff_with_stats("", "").unwrap();
        assert_eq!(empty_diff, "(no changes)");
        assert!(empty_stats.unchanged);

        let patch_from_empty = compute_safe_myers_diff("", "single line\n").unwrap();
        assert_eq!(patch_from_empty.stats.additions, 1);
        assert_eq!(patch_from_empty.stats.deletions, 0);
        assert!(patch_from_empty.diff_text.contains("+1: single line"));

        let patch_to_empty = compute_safe_myers_diff("single line\n", "").unwrap();
        assert_eq!(patch_to_empty.stats.additions, 0);
        assert_eq!(patch_to_empty.stats.deletions, 1);
        assert!(patch_to_empty.diff_text.contains("-1: single line"));
    }
}
