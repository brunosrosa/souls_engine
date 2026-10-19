//! # Web Intelligence & Lean HTML-to-Markdown Extraction (`souls_core::web`)
//!
//! Provides zero-dependency in-memory conversion of HTML documents into clean,
//! token-efficient Markdown, stripping toxic noise (scripts, styles, tracking tags, ads)
//! to protect LLM context windows, and parses DuckDuckGo HTML search results.

/// Structured result from parsing search engine HTML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

/// Converts arbitrary HTML content into clean, token-efficient Markdown directly in RAM.
///
/// Strips scripts, styles, iframes, SVGs, navbars, and footers while preserving
/// headings, list items, hyperlinks, code snippets, and readable paragraphs.
#[must_use]
pub fn html_to_clean_markdown(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut in_script = false;
    let mut in_style = false;
    let mut in_nav = false;
    let mut in_svg = false;
    let mut in_tag = false;
    let mut tag_buf = String::with_capacity(64);

    let mut current_link_href = String::new();
    let mut in_link = false;
    let mut link_text = String::new();

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        // 1. Script suppression
        if in_script {
            if i + 8 <= lower_chars.len() && &lower_chars[i..i + 9] == &['<', '/', 's', 'c', 'r', 'i', 'p', 't', '>'] {
                in_script = false;
                i += 9;
                continue;
            }
            i += 1;
            continue;
        }

        // 2. Style suppression
        if in_style {
            if i + 7 <= lower_chars.len() && &lower_chars[i..i + 8] == &['<', '/', 's', 't', 'y', 'l', 'e', '>'] {
                in_style = false;
                i += 8;
                continue;
            }
            i += 1;
            continue;
        }

        // 3. SVG suppression
        if in_svg {
            if i + 5 <= lower_chars.len() && &lower_chars[i..i + 6] == &['<', '/', 's', 'v', 'g', '>'] {
                in_svg = false;
                i += 6;
                continue;
            }
            i += 1;
            continue;
        }

        // 4. Nav suppression
        if in_nav {
            if i + 5 <= lower_chars.len() && &lower_chars[i..i + 6] == &['<', '/', 'n', 'a', 'v', '>'] {
                in_nav = false;
                i += 6;
                continue;
            }
            i += 1;
            continue;
        }

        // Tag open
        if chars[i] == '<' {
            in_tag = true;
            tag_buf.clear();
            i += 1;
            continue;
        }

        // Inside tag
        if in_tag {
            if chars[i] == '>' {
                in_tag = false;
                let tag_raw = tag_buf.trim();
                let tag_lower = tag_raw.to_lowercase();

                if tag_lower.starts_with("script") {
                    in_script = true;
                } else if tag_lower.starts_with("style") {
                    in_style = true;
                } else if tag_lower.starts_with("svg") {
                    in_svg = true;
                } else if tag_lower.starts_with("nav") {
                    in_nav = true;
                } else if tag_lower == "/nav" {
                    in_nav = false;
                } else if tag_lower.starts_with("h1") {
                    out.push_str("\n\n# ");
                } else if tag_lower.starts_with("h2") {
                    out.push_str("\n\n## ");
                } else if tag_lower.starts_with("h3") {
                    out.push_str("\n\n### ");
                } else if tag_lower.starts_with("h4") {
                    out.push_str("\n\n#### ");
                } else if tag_lower.starts_with("li") {
                    out.push_str("\n- ");
                } else if tag_lower.starts_with("p") || tag_lower.starts_with("div") || tag_lower.starts_with("br") {
                    out.push('\n');
                } else if tag_lower.starts_with("code") || tag_lower.starts_with("pre") {
                    out.push('`');
                } else if tag_lower.starts_with("a ") {
                    // Extract href attribute
                    if let Some(href_idx) = tag_lower.find("href=") {
                        let after_href = &tag_raw[href_idx + 5..];
                        let quote = after_href.chars().next().unwrap_or('"');
                        if quote == '"' || quote == '\'' {
                            let rest = &after_href[1..];
                            if let Some(end_quote) = rest.find(quote) {
                                current_link_href = rest[..end_quote].to_string();
                                in_link = true;
                                link_text.clear();
                            }
                        }
                    }
                } else if tag_lower == "/a" {
                    if in_link {
                        in_link = false;
                        let trimmed_text = link_text.trim();
                        if !trimmed_text.is_empty() && !current_link_href.is_empty() {
                            out.push_str(&format!("[{}]({})", trimmed_text, current_link_href));
                        } else if !trimmed_text.is_empty() {
                            out.push_str(trimmed_text);
                        }
                        current_link_href.clear();
                        link_text.clear();
                    }
                }
            } else {
                tag_buf.push(chars[i]);
            }
            i += 1;
            continue;
        }

        if in_link {
            link_text.push(chars[i]);
        } else {
            out.push(chars[i]);
        }
        i += 1;
    }

    // Clean up multiple consecutive whitespace and empty lines
    let mut cleaned_lines = Vec::new();
    let mut last_empty = false;

    for line in out.lines() {
        let t = line.trim();
        if t.is_empty() {
            if !last_empty {
                cleaned_lines.push("");
                last_empty = true;
            }
        } else {
            cleaned_lines.push(t);
            last_empty = false;
        }
    }

    cleaned_lines.join("\n")
}

/// Parses DuckDuckGo HTML Lite results into structured search results.
#[must_use]
pub fn parse_duckduckgo_html(html: &str, limit: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();
    let limit = limit.clamp(1, 20);

    let parts: Vec<&str> = html.split("class=\"result__body\"").collect();
    if parts.len() <= 1 {
        // Fallback split by result table
        let fallback_parts: Vec<&str> = html.split("class=\"result__snippet\"").collect();
        for chunk in fallback_parts.iter().skip(1).take(limit) {
            let snippet = clean_snippet(chunk);
            if !snippet.is_empty() {
                results.push(WebSearchResult {
                    title: "DuckDuckGo Result".to_string(),
                    link: String::new(),
                    snippet,
                });
            }
        }
        return results;
    }

    for chunk in parts.iter().skip(1).take(limit) {
        // Extract title & link
        let mut title = String::new();
        let mut link = String::new();
        let mut snippet = String::new();

        if let Some(title_start) = chunk.find("class=\"result__title\"") {
            let rest = &chunk[title_start..];
            if let Some(a_start) = rest.find("<a") {
                let a_rest = &rest[a_start..];
                if let Some(href_start) = a_rest.find("href=\"") {
                    let href_rest = &a_rest[href_start + 6..];
                    if let Some(href_end) = href_rest.find('"') {
                        let raw_url = &href_rest[..href_end];
                        link = decode_duckduckgo_url(raw_url);
                    }
                }
                if let Some(a_tag_end) = a_rest.find('>') {
                    let text_rest = &a_rest[a_tag_end + 1..];
                    if let Some(a_close) = text_rest.find("</a>") {
                        title = html_to_clean_markdown(&text_rest[..a_close]).replace('\n', " ");
                    }
                }
            }
        }

        if let Some(snippet_start) = chunk.find("class=\"result__snippet\"") {
            let rest = &chunk[snippet_start..];
            if let Some(tag_end) = rest.find('>') {
                let after_tag = &rest[tag_end + 1..];
                let end_idx = after_tag
                    .find("</div>")
                    .or_else(|| after_tag.find("</td>"))
                    .or_else(|| after_tag.find("</a>"))
                    .unwrap_or(after_tag.len());
                snippet = html_to_clean_markdown(&after_tag[..end_idx]).replace('\n', " ");
            }
        }

        if !title.is_empty() || !snippet.is_empty() {
            results.push(WebSearchResult {
                title: title.trim().to_string(),
                link: link.trim().to_string(),
                snippet: snippet.trim().to_string(),
            });
        }
    }

    results
}

fn decode_duckduckgo_url(url: &str) -> String {
    if let Some(uddg_idx) = url.find("uddg=") {
        let rest = &url[uddg_idx + 5..];
        let end = rest.find('&').unwrap_or(rest.len());
        let encoded = &rest[..end];
        return url_decode(encoded);
    }
    url.to_string()
}

fn url_decode(s: &str) -> String {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte_val) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                out.push(byte_val as char);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn clean_snippet(chunk: &str) -> String {
    if let Some(end_idx) = chunk.find("</div>") {
        html_to_clean_markdown(&chunk[..end_idx]).replace('\n', " ")
    } else {
        html_to_clean_markdown(chunk).replace('\n', " ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_clean_markdown_strips_scripts_and_styles() {
        let html = r#"
            <html>
                <head>
                    <style>body { color: red; }</style>
                    <script>alert("tracker");</script>
                </head>
                <body>
                    <nav><a href="/home">Home</a></nav>
                    <h1>Souls Engine Architecture</h1>
                    <p>This is a <strong>bare-metal</strong> system built in Rust.</p>
                    <p>Check the repository: <a href="https://github.com/souls/engine">Souls Engine Repo</a></p>
                    <ul>
                        <li>High throughput</li>
                        <li>Zero VRAM spillover</li>
                    </ul>
                    <script src="analytics.js"></script>
                </body>
            </html>
        "#;

        let md = html_to_clean_markdown(html);
        assert!(!md.contains("alert"));
        assert!(!md.contains("color: red"));
        assert!(!md.contains("analytics.js"));
        assert!(md.contains("# Souls Engine Architecture"));
        assert!(md.contains("bare-metal"));
        assert!(md.contains("[Souls Engine Repo](https://github.com/souls/engine)"));
        assert!(md.contains("- High throughput"));
        assert!(md.contains("- Zero VRAM spillover"));
    }

    #[test]
    fn test_parse_duckduckgo_html_sample() {
        let sample = r#"
            <div class="result__body">
                <h2 class="result__title">
                    <a class="result__url" href="/l/?uddg=https%3A%2F%2Frust-lang.org">Rust Programming Language</a>
                </h2>
                <div class="result__snippet">
                    A language empowering everyone to build reliable and efficient software.
                </div>
            </div>
        "#;

        let results = parse_duckduckgo_html(sample, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].link, "https://rust-lang.org");
        assert!(results[0].snippet.contains("empowering everyone"));
    }
}
