//! Pure O(N) ANSI escape code filter (VT100 / CSI sequences).
//!
//! Transplanted from `_donor/souls_mc_core/src/cognition/context/ansi_filter.rs`
//! with zero-regex allocation-conscious scanning.

/// Strips ANSI escape sequences (`\x1b[...m`, `\x1b[2K`, `\x1b[H`, etc.) from the input string.
///
/// Returns pure, clean text without any terminal formatting codes.
/// Fast-path optimization: if no `\x1b` byte is found, returns immediately.
pub fn strip_ansi_escapes(input: &str) -> String {
    if !input.contains('\x1b') {
        return input.to_string();
    }
    let mut result = String::with_capacity(input.len());
    let mut in_escape = false;
    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}

/// Alias for backward compatibility with donor codebase.
#[inline]
pub fn strip_ansi(s: &str) -> String {
    strip_ansi_escapes(s)
}

/// Returns the density of ANSI escape characters in a string (range 0.0 to 1.0).
/// Returns 0.0 if the string is empty.
pub fn ansi_density(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let escape_count = s.chars().filter(|&c| c == '\x1b').count();
    escape_count as f64 / s.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_escapes_powershell_dense() {
        // Simulates dense PowerShell terminal output with colors, cursor movements, and clearing
        let ps_output = "\x1b[38;5;10mPS Z:\\souls_engine>\x1b[0m \x1b[93mcargo test\x1b[0m\r\n\x1b[2K\x1b[1A\x1b[32m   Compiling\x1b[0m souls_core v0.1.0\r\n\x1b[32m    Finished\x1b[0m `test` profile [unoptimized + debuginfo] in 1.23s\r\n\x1b[?25h";
        let cleaned = strip_ansi_escapes(ps_output);
        assert_eq!(
            cleaned,
            "PS Z:\\souls_engine> cargo test\r\n   Compiling souls_core v0.1.0\r\n    Finished `test` profile [unoptimized + debuginfo] in 1.23s\r\n"
        );
        assert!(!cleaned.contains('\x1b'));
    }

    #[test]
    fn test_strip_ansi_escapes_no_escapes() {
        let plain = "Clean string without any escapes.";
        assert_eq!(strip_ansi_escapes(plain), plain);
    }

    #[test]
    fn test_ansi_density() {
        assert_eq!(ansi_density(""), 0.0);
        assert_eq!(ansi_density("clean"), 0.0);
        let with_esc = "\x1b[31mred\x1b[0m";
        let density = ansi_density(with_esc);
        assert!(density > 0.15 && density < 0.20);
    }
}
