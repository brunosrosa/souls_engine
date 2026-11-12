//! # personas_hub.rs — Hermes Personas, Privacy Alignment Guardian & Persona Studio (v0.6.0)
//!
//! Provides complete visual and programmatic governance over Hermes Agent profiles
//! (`default` + `%LOCALAPPDATA%\hermes\profiles\*`), enforcing:
//! - Strict YAML block surgery (`AGENTS.md` §8.1 — zero naive `starts_with` mutations).
//! - Constitutional Read-Only sovereignty over `MEMORY.md` and `USER.md` (`AGENTS.md` §4).
//! - Zero exposure of `.env` or `auth.json` credentials over HTTP/JSON.
//! - Strict LF (`\n`) normalization (`AGENTS.md` §3) and automatic versioned backups.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use souls_ast::{compute_safe_myers_diff, DiffTag};
use souls_inference_runtime::GenParams;

use crate::skills_hub::{self, SkillsHubRoots, VisualDiffLine};
use crate::state::AppState;

// ============================================================================
// PORTABLE ROOTS & DIRECTORY RESOLUTION
// ============================================================================

/// Resolved filesystem roots for Hermes Personas and internal Souls Hub history/archives.
#[derive(Debug, Clone)]
pub struct PersonasHubRoots {
    /// Internal Souls Engine storage (`.souls_data/personas_hub`).
    pub hub_data_dir: PathBuf,
    /// Hermes root directory (`%LOCALAPPDATA%\hermes` or `SOULS_HERMES_ROOT`).
    pub hermes_root_dir: PathBuf,
}

impl PersonasHubRoots {
    /// Resolves roots dynamically across Dev Workspace and Installed App environments.
    pub fn from_system() -> Self {
        let local_app = std::env::var("LOCALAPPDATA")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("USERPROFILE")
                    .ok()
                    .map(|u| PathBuf::from(u).join("AppData").join("Local"))
            })
            .unwrap_or_else(|| PathBuf::from("."));

        let user_home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let data_base = if let Ok(explicit_data) = std::env::var("SOULS_DATA_DIR") {
            let trimmed = explicit_data.trim();
            if !trimmed.is_empty() {
                PathBuf::from(trimmed)
            } else {
                Self::default_data_base(&local_app)
            }
        } else {
            Self::default_data_base(&local_app)
        };

        let hub_data_dir = data_base.join("personas_hub");

        let hermes_root_dir = if let Ok(explicit_hermes) = std::env::var("SOULS_HERMES_ROOT") {
            let trimmed = explicit_hermes.trim();
            if !trimmed.is_empty() {
                PathBuf::from(trimmed)
            } else {
                local_app.join("hermes")
            }
        } else {
            let win_hermes = local_app.join("hermes");
            if win_hermes.is_dir() {
                win_hermes
            } else {
                let unix_hermes = user_home.join(".hermes");
                if unix_hermes.is_dir() {
                    unix_hermes
                } else {
                    win_hermes
                }
            }
        };

        Self {
            hub_data_dir,
            hermes_root_dir,
        }
    }

    fn default_data_base(local_app: &Path) -> PathBuf {
        let z_dev = PathBuf::from(r"Z:\souls_engine\.souls_data");
        if Path::new(r"Z:\souls_engine\Cargo.toml").is_file() || z_dev.is_dir() {
            return z_dev;
        }
        if let Ok(cwd) = std::env::current_dir() {
            if cwd.join(".souls_data").is_dir() {
                return cwd.join(".souls_data");
            }
        }
        local_app.join("SoulsEngine").join("data")
    }

    /// Returns `true` when Hermes Agent is present on this host.
    pub fn is_hermes_installed(&self) -> bool {
        self.hermes_root_dir.join("config.yaml").is_file()
            || self.hermes_root_dir.join("SOUL.md").is_file()
            || self.hermes_root_dir.join("profiles").is_dir()
    }

    pub fn history_dir(&self) -> PathBuf {
        self.hub_data_dir.join("history")
    }

    pub fn archived_dir(&self) -> PathBuf {
        self.hub_data_dir.join("archived")
    }

    /// Resolves the directory of an active Persona (`"default"` or `profiles/<slug>`).
    pub fn persona_dir(&self, slug: &str) -> Result<PathBuf, String> {
        let clean = normalize_persona_slug(slug);
        if clean.is_empty() {
            return Err("Identificador de Persona inválido.".to_string());
        }
        if clean == "default" {
            Ok(self.hermes_root_dir.clone())
        } else {
            Ok(self.hermes_root_dir.join("profiles").join(clean))
        }
    }
}

// ============================================================================
// DATA CONTRACTS & SERIALIZABLE MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulSectionSummary {
    pub heading: String,
    pub line_start: usize,
    pub bullet_count: usize,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoredPathRef {
    pub raw_path: String,
    pub resolved_path: String,
    pub kind: String, // "contract_agents_md", "wiki_note", "raw_vault", "directory", "other"
    pub exists_on_host: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulAnatomy {
    pub headline: String,
    pub opening_mission: String,
    pub sections: Vec<SoulSectionSummary>,
    pub requires_strict_privacy: bool,
    pub privacy_keywords_found: Vec<String>,
    pub estimated_tokens: usize,
    pub anchored_paths: Vec<AnchoredPathRef>,
    pub truncated_duplicate_lines: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaGovernanceFinding {
    pub code: String,
    pub severity: String, // "critical", "warning", "info"
    pub title: String,
    pub explanation: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadOnlyMemoryNote {
    pub filename: String,
    pub rel_path: String,
    pub size_bytes: u64,
    pub content_preview: String,
    pub read_only_constitutional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoredContractPreview {
    pub label: String,
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub content_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSkillBrief {
    pub slug: String,
    pub description: String,
    pub enabled: bool,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSummaryRecord {
    pub slug: String,
    pub is_default: bool,
    pub display_title: String,
    pub subtitle_role: String,
    pub description: String,
    pub accent_color: String,
    pub has_custom_avatar: bool,
    pub root_path: String,
    // Model & behavior config
    pub model_default: String,
    pub model_provider: String,
    pub model_base_url: String,
    pub reasoning_effort: String,
    pub max_turns: u64,
    pub memory_enabled: bool,
    pub souls_mcp_connected: bool,
    pub has_free_tier_exposure: bool,
    // Identity & Skills metrics
    pub requires_strict_privacy: bool,
    pub soul_tokens: usize,
    pub soul_sections_count: usize,
    pub active_skills_count: usize,
    pub paused_skills_count: usize,
    pub anchored_paths_count: usize,
    pub history_snapshots_count: usize,
    // Governance radar
    pub alignment_score: u8,
    pub status_badge: String, // "aligned", "privacy_alert", "needs_attention"
    pub status_label: String,
    pub governance_summary: String,
    pub findings: Vec<PersonaGovernanceFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedPersonaItem {
    pub slug: String,
    pub display_title: String,
    pub description: String,
    pub archived_at_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonasEcosystemOverview {
    pub hermes_installed: bool,
    pub hermes_root_dir: String,
    pub total_active_personas: usize,
    pub archived_personas_count: usize,
    pub souls_connected_count: usize,
    pub privacy_alerts_count: usize,
    pub attention_needed_count: usize,
    pub avg_alignment_score: f64,
    pub total_persona_skills_links: usize,
    pub recommended_private_model: String,
    pub recommended_private_provider: String,
    pub personas: Vec<PersonaSummaryRecord>,
    pub archived_personas: Vec<ArchivedPersonaItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaHistorySnapshot {
    pub snapshot_id: String,
    pub slug: String,
    pub target_file: String, // "SOUL.md", "config.yaml", "profile.yaml"
    pub timestamp_epoch: u64,
    pub size_bytes: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaFileDiffPreview {
    pub file_name: String,
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: bool,
    pub lines: Vec<VisualDiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaUpdateRequest {
    pub slug: String,
    pub display_title: Option<String>,
    pub description: Option<String>,
    pub accent_color: Option<String>,
    pub model_default: Option<String>,
    pub model_provider: Option<String>,
    pub reasoning_effort: Option<String>,
    pub max_turns: Option<u64>,
    pub memory_enabled: Option<bool>,
    pub souls_mcp_connected: Option<bool>,
    pub sanitize_free_fallbacks: Option<bool>,
    pub soul_content: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaAlignRequest {
    pub slug: String,
    #[serde(default)]
    pub dry_run: bool,
    pub preferred_model: Option<String>,
    pub preferred_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaCreateRequest {
    pub slug: String,
    pub display_title: String,
    pub description: String,
    pub template_kind: Option<String>, // "finance", "health", "studies", "coding", "creative", "custom"
    pub goal_prompt: String,
    pub privacy_mode: Option<String>, // "strict_private" | "standard"
    pub engine: Option<String>,       // "deterministic_scaffold" | "local_silicon" | "cloud_worker"
    pub clone_from: Option<String>,   // defaults to "default"
    pub model_default: Option<String>,
    pub model_provider: Option<String>,
    pub connect_souls_mcp: Option<bool>,
    pub initial_skills: Option<Vec<String>>,
    pub accent_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSoulRefineRequest {
    pub slug: String,
    pub current_soul: String,
    pub instruction: Option<String>,
    pub use_local_model: Option<bool>,
}

// ============================================================================
// SANITIZATION & SLUG HELPERS
// ============================================================================

pub fn sanitize_lf(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn normalize_persona_slug(raw: &str) -> String {
    let trimmed = raw.trim().to_lowercase();
    if trimmed == "default" || trimmed == "principal" || trimmed == "main" {
        return "default".to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_dash = false;
    for ch in trimmed.chars() {
        let mapped = match ch {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            c if c.is_ascii_alphanumeric() => c,
            '-' | '_' | ' ' => '-',
            _ => continue,
        };
        if mapped == '-' {
            if !out.is_empty() && !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        } else {
            out.push(mapped);
            prev_dash = false;
        }
    }
    out.trim_matches('-').chars().take(42).collect()
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn unquote_yaml_scalar(raw: &str) -> String {
    let s = raw.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn format_yaml_quoted(raw: &str) -> String {
    let clean = raw.replace('\r', "").replace('\n', " ").trim().to_string();
    format!("\"{}\"", clean.replace('\\', "\\\\").replace('"', "\\\""))
}

// ============================================================================
// HIERARCHICAL YAML BLOCK SURGEON (AGENTS.md §8.1 COMPLIANT)
// ============================================================================

/// Reads a scalar value at an exact hierarchical YAML key path (e.g. `&["model", "default"]`
/// or `&["ui_meta", "hermes-bots", "title"]`), supporting multi-line folded scalars at leaf level.
pub fn yaml_get_nested(yaml_raw: &str, key_path: &[&str]) -> Option<String> {
    if key_path.is_empty() {
        return None;
    }
    let clean = sanitize_lf(yaml_raw);
    let lines: Vec<&str> = clean.lines().collect();
    let mut stack: Vec<(usize, String)> = Vec::new();

    for idx in 0..lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start_matches(' ').len();
        if trimmed.starts_with("- ") {
            // List items belong inside current parent but do not form mapping keys
            continue;
        }
        let Some(colon_pos) = trimmed.find(':') else {
            continue;
        };
        let key = trimmed[..colon_pos].trim().trim_matches(['"', '\'']);
        let after_colon = trimmed[colon_pos + 1..].trim();

        while let Some(&(top_indent, _)) = stack.last() {
            if top_indent >= indent {
                stack.pop();
            } else {
                break;
            }
        }
        stack.push((indent, key.to_string()));

        if stack.len() == key_path.len()
            && stack
                .iter()
                .zip(key_path.iter())
                .all(|((_, k), target)| k == target)
        {
            if !after_colon.is_empty() && after_colon != "|" && after_colon != ">" {
                let mut val = unquote_yaml_scalar(after_colon);
                // Check for YAML plain multi-line continuation lines (indented deeper without `:` mapping)
                let mut next_idx = idx + 1;
                while next_idx < lines.len() {
                    let nl = lines[next_idx];
                    let nt = nl.trim();
                    if nt.is_empty() {
                        break;
                    }
                    let n_indent = nl.len() - nl.trim_start_matches(' ').len();
                    if n_indent <= indent {
                        break;
                    }
                    // If continuation line looks like a child key `foo: bar`, stop unless inside quotes
                    if let Some(cp) = nt.find(':') {
                        let maybe_key = &nt[..cp];
                        if !maybe_key.contains(' ') && !maybe_key.contains('`') {
                            break;
                        }
                    }
                    val.push(' ');
                    val.push_str(nt);
                    next_idx += 1;
                }
                return Some(val.trim().to_string());
            } else if after_colon == "|" || after_colon == ">" {
                let mut parts = Vec::new();
                let mut next_idx = idx + 1;
                while next_idx < lines.len() {
                    let nl = lines[next_idx];
                    let nt = nl.trim();
                    if !nt.is_empty() {
                        let n_indent = nl.len() - nl.trim_start_matches(' ').len();
                        if n_indent <= indent {
                            break;
                        }
                        parts.push(nt);
                    }
                    next_idx += 1;
                }
                return Some(parts.join(" ").trim().to_string());
            } else {
                return Some(String::new());
            }
        }
    }
    None
}

/// Checks whether a nested YAML block path exists (e.g. `&["mcp_servers", "souls_engine"]`).
pub fn yaml_has_block_path(yaml_raw: &str, key_path: &[&str]) -> bool {
    if key_path.is_empty() {
        return false;
    }
    let clean = sanitize_lf(yaml_raw);
    let mut stack: Vec<(usize, String)> = Vec::new();

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("- ") {
            continue;
        }
        let Some(colon_pos) = trimmed.find(':') else {
            continue;
        };
        let indent = line.len() - line.trim_start_matches(' ').len();
        let key = trimmed[..colon_pos].trim().trim_matches(['"', '\'']);

        while let Some(&(top_indent, _)) = stack.last() {
            if top_indent >= indent {
                stack.pop();
            } else {
                break;
            }
        }
        stack.push((indent, key.to_string()));

        if stack.len() == key_path.len()
            && stack
                .iter()
                .zip(key_path.iter())
                .all(|((_, k), target)| k == target)
        {
            return true;
        }
    }
    false
}

/// Safely mutates a 2-level nested scalar (`parent.child = formatted_value`) inside a YAML file
/// by navigating strictly to the `parent:` block at indent 0 and updating/inserting `  child:`
/// without touching any other blocks or comments (`AGENTS.md` §8.1).
pub fn yaml_set_nested_scalar(
    yaml_raw: &str,
    parent_key: &str,
    child_key: &str,
    formatted_yaml_val: &str,
) -> String {
    let clean = sanitize_lf(yaml_raw);
    let lines: Vec<&str> = clean.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 4);

    let mut idx = 0usize;
    let mut parent_found = false;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start_matches(' ').len();

        let is_target_parent = indent == 0
            && !trimmed.starts_with('#')
            && trimmed
                .strip_prefix(parent_key)
                .map(|rest| {
                    let r = rest.trim_start();
                    r.starts_with(':') && (r[1..].trim().is_empty() || r[1..].trim().starts_with('#'))
                })
                .unwrap_or(false);

        if is_target_parent && !parent_found {
            parent_found = true;
            out.push(line.to_string());
            idx += 1;

            let mut child_updated = false;
            let mut child_indent_str = "  ".to_string();

            while idx < lines.len() {
                let b_line = lines[idx];
                let b_trim = b_line.trim();
                let b_indent = b_line.len() - b_line.trim_start_matches(' ').len();

                // End of `parent:` block reached when a non-empty, non-comment line at indent 0 appears
                if !b_trim.is_empty() && !b_trim.starts_with('#') && b_indent == 0 {
                    break;
                }

                if !b_trim.is_empty() && !b_trim.starts_with('#') && b_indent > 0 && child_indent_str == "  " {
                    // Detect actual direct-child indentation (usually 2 spaces)
                    if b_indent <= 4 && !b_trim.starts_with("- ") {
                        child_indent_str = " ".repeat(b_indent);
                    }
                }

                let direct_indent_len = child_indent_str.len();
                let is_target_child = b_indent == direct_indent_len
                    && !b_trim.starts_with('#')
                    && !b_trim.starts_with("- ")
                    && b_trim
                        .strip_prefix(child_key)
                        .map(|rest| rest.trim_start().starts_with(':'))
                        .unwrap_or(false);

                if is_target_child && !child_updated {
                    out.push(format!("{child_indent_str}{child_key}: {formatted_yaml_val}"));
                    child_updated = true;
                    idx += 1;
                    // Skip any wrapped multi-line scalar continuation lines that belonged to this child
                    while idx < lines.len() {
                        let cont = lines[idx];
                        let cont_t = cont.trim();
                        if cont_t.is_empty() {
                            break;
                        }
                        let cont_ind = cont.len() - cont.trim_start_matches(' ').len();
                        if cont_ind <= direct_indent_len {
                            break;
                        }
                        if let Some(cp) = cont_t.find(':') {
                            let mk = &cont_t[..cp];
                            if !mk.contains(' ') && !mk.contains('`') {
                                break;
                            }
                        }
                        idx += 1;
                    }
                    continue;
                }

                out.push(b_line.to_string());
                idx += 1;
            }

            if !child_updated {
                // Insert right before trailing blank lines of the parent block
                let mut insert_pos = out.len();
                while insert_pos > 0 && out[insert_pos - 1].trim().is_empty() {
                    insert_pos -= 1;
                }
                out.insert(
                    insert_pos,
                    format!("{child_indent_str}{child_key}: {formatted_yaml_val}"),
                );
            }
            continue;
        }

        out.push(line.to_string());
        idx += 1;
    }

    if !parent_found {
        if !out.is_empty() && !out.last().unwrap().trim().is_empty() {
            out.push(String::new());
        }
        out.push(format!("{parent_key}:"));
        out.push(format!("  {child_key}: {formatted_yaml_val}"));
    }

    let mut joined = out.join("\n");
    if !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

/// Idempotently ensures `mcp_servers.souls_engine.url = "http://127.0.0.1:9123/mcp"` is configured
/// inside `config.yaml` without clobbering any sibling MCP servers (`AGENTS.md` §8.1).
pub fn yaml_set_souls_mcp_connection(yaml_raw: &str, enabled: bool) -> String {
    let clean = sanitize_lf(yaml_raw);
    let has_souls = yaml_has_block_path(&clean, &["mcp_servers", "souls_engine"]);
    if enabled && has_souls {
        return if clean.ends_with('\n') {
            clean
        } else {
            format!("{clean}\n")
        };
    }
    if !enabled && !has_souls {
        return if clean.ends_with('\n') {
            clean
        } else {
            format!("{clean}\n")
        };
    }

    let lines: Vec<&str> = clean.lines().collect();
    if enabled {
        // Check if top-level `mcp_servers:` block already exists
        if yaml_has_block_path(&clean, &["mcp_servers"]) {
            let mut out = Vec::with_capacity(lines.len() + 3);
            let mut inserted = false;
            for line in lines {
                let trimmed = line.trim();
                let indent = line.len() - line.trim_start_matches(' ').len();
                out.push(line.to_string());
                if !inserted && indent == 0 && trimmed.starts_with("mcp_servers:") {
                    out.push("  souls_engine:".to_string());
                    out.push("    url: \"http://127.0.0.1:9123/mcp\"".to_string());
                    inserted = true;
                }
            }
            let mut res = out.join("\n");
            if !res.ends_with('\n') {
                res.push('\n');
            }
            res
        } else {
            let header = "mcp_servers:\n  souls_engine:\n    url: \"http://127.0.0.1:9123/mcp\"\n";
            let mut res = format!("{header}{}", clean.trim_start_matches('\n'));
            if !res.ends_with('\n') {
                res.push('\n');
            }
            res
        }
    } else {
        // Remove only the `  souls_engine:` sub-block inside `mcp_servers:`
        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut in_mcp = false;
        let mut skipping_souls = false;
        let mut souls_indent = 2usize;

        for line in &lines {
            let trimmed = line.trim();
            let indent = line.len() - line.trim_start_matches(' ').len();

            if !trimmed.is_empty() && !trimmed.starts_with('#') && indent == 0 {
                in_mcp = trimmed.starts_with("mcp_servers:");
                skipping_souls = false;
            } else if in_mcp {
                if skipping_souls {
                    if !trimmed.is_empty() && !trimmed.starts_with('#') && indent <= souls_indent {
                        skipping_souls = false;
                    } else {
                        continue;
                    }
                }
                if !trimmed.is_empty()
                    && !trimmed.starts_with('#')
                    && indent > 0
                    && trimmed.starts_with("souls_engine:")
                {
                    skipping_souls = true;
                    souls_indent = indent;
                    continue;
                }
            }
            out.push((*line).to_string());
        }

        // If `mcp_servers:` became completely empty, remove the empty header cleanly
        let mut cleaned_out: Vec<String> = Vec::with_capacity(out.len());
        let mut i = 0usize;
        while i < out.len() {
            let t = out[i].trim();
            let ind = out[i].len() - out[i].trim_start_matches(' ').len();
            if ind == 0 && t == "mcp_servers:" {
                let mut j = i + 1;
                let mut has_children = false;
                while j < out.len() {
                    let jt = out[j].trim();
                    if !jt.is_empty() && !jt.starts_with('#') {
                        let j_ind = out[j].len() - out[j].trim_start_matches(' ').len();
                        has_children = j_ind > 0;
                        break;
                    }
                    j += 1;
                }
                if !has_children {
                    i += 1;
                    continue;
                }
            }
            cleaned_out.push(out[i].clone());
            i += 1;
        }

        let mut res = cleaned_out.join("\n");
        if !res.ends_with('\n') {
            res.push('\n');
        }
        res
    }
}

/// Removes `- provider: opencode-free` (and any child properties of that list entry)
/// from YAML routing/fallback lists when enforcing strict privacy.
pub fn yaml_remove_free_fallbacks(yaml_raw: &str) -> String {
    let clean = sanitize_lf(yaml_raw);
    let lines: Vec<&str> = clean.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start_matches(' ').len();

        let is_free_item = trimmed.starts_with("- provider:")
            && (trimmed.contains("opencode-free") || trimmed.ends_with(":free"));

        if is_free_item {
            idx += 1;
            // Skip child properties belonging to this YAML sequence item (`indent > list_indent`)
            while idx < lines.len() {
                let next_l = lines[idx];
                let next_t = next_l.trim();
                if next_t.is_empty() {
                    break;
                }
                let next_ind = next_l.len() - next_l.trim_start_matches(' ').len();
                if next_ind <= indent {
                    break;
                }
                idx += 1;
            }
            continue;
        }

        out.push(line.to_string());
        idx += 1;
    }

    let mut res = out.join("\n");
    if !res.ends_with('\n') {
        res.push('\n');
    }
    res
}

/// Updates `profile.yaml` (`description`, `display_name` / `ui_meta.hermes-bots.title`, and `color`)
/// cleanly while preserving `shape`, `imageKind`, `created` and `_ui_meta_revisions`.
pub fn yaml_update_profile_metadata(
    profile_yaml_raw: &str,
    is_default: bool,
    display_title: Option<&str>,
    description: Option<&str>,
    accent_color: Option<&str>,
) -> String {
    let mut doc = sanitize_lf(profile_yaml_raw);
    if doc.trim().is_empty() {
        doc = "description_auto: false\nui_meta:\n  hermes-bots:\n    title: Persona\n".to_string();
    }

    // 1. Update top-level `description:` cleanly (handling multi-line plain continuation lines)
    if let Some(desc) = description {
        let clean_desc = desc.replace('\r', "").replace('\n', " ").trim().to_string();
        if !clean_desc.is_empty() {
            let lines: Vec<&str> = doc.lines().collect();
            let mut out: Vec<String> = Vec::with_capacity(lines.len() + 2);
            let mut found_desc = false;
            let mut idx = 0usize;

            while idx < lines.len() {
                let line = lines[idx];
                let trimmed = line.trim();
                let indent = line.len() - line.trim_start_matches(' ').len();

                if indent == 0 && trimmed.starts_with("description:") && !found_desc {
                    found_desc = true;
                    out.push(format!("description: {}", format_yaml_quoted(&clean_desc)));
                    idx += 1;
                    // Skip wrapped continuation lines of previous multiline `description:`
                    while idx < lines.len() {
                        let nl = lines[idx];
                        let nt = nl.trim();
                        if nt.is_empty() {
                            break;
                        }
                        let n_ind = nl.len() - nl.trim_start_matches(' ').len();
                        if n_ind == 0 {
                            break;
                        }
                        idx += 1;
                    }
                    continue;
                }
                out.push(line.to_string());
                idx += 1;
            }

            if !found_desc {
                out.insert(0, format!("description: {}", format_yaml_quoted(&clean_desc)));
            }
            doc = out.join("\n") + "\n";
        }
    }

    // 2. Update `display_name` (if default) and `ui_meta -> hermes-bots -> title / color`
    if let Some(title) = display_title {
        let clean_t = title.trim();
        if !clean_t.is_empty() {
            if is_default && yaml_get_nested(&doc, &["display_name"]).is_some() {
                // Replace top-level display_name
                let lines: Vec<String> = doc
                    .lines()
                    .map(|l| {
                        let ind = l.len() - l.trim_start_matches(' ').len();
                        if ind == 0 && l.trim().starts_with("display_name:") {
                            format!("display_name: {}", format_yaml_quoted(clean_t))
                        } else {
                            l.to_string()
                        }
                    })
                    .collect();
                doc = lines.join("\n") + "\n";
            }
            doc = yaml_set_three_level_scalar(
                &doc,
                "ui_meta",
                "hermes-bots",
                "title",
                &format_yaml_quoted(clean_t),
            );
        }
    }

    if let Some(col) = accent_color {
        let clean_c = col.trim();
        if clean_c.starts_with('#') && (clean_c.len() == 7 || clean_c.len() == 4) {
            doc = yaml_set_three_level_scalar(
                &doc,
                "ui_meta",
                "hermes-bots",
                "color",
                &format!("'{clean_c}'"),
            );
        }
    }

    if !doc.ends_with('\n') {
        doc.push('\n');
    }
    doc
}

fn yaml_set_three_level_scalar(
    yaml_raw: &str,
    l1: &str,
    l2: &str,
    l3: &str,
    formatted_val: &str,
) -> String {
    let clean = sanitize_lf(yaml_raw);
    if !yaml_has_block_path(&clean, &[l1, l2]) {
        let mut out = clean.trim_end().to_string();
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("{l1}:\n  {l2}:\n    {l3}: {formatted_val}\n"));
        return out;
    }

    let lines: Vec<&str> = clean.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 2);
    let mut idx = 0usize;
    let mut updated = false;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start_matches(' ').len();

        if !updated && indent == 0 && trimmed.starts_with(&format!("{l1}:")) {
            out.push(line.to_string());
            idx += 1;
            while idx < lines.len() {
                let l2_line = lines[idx];
                let l2_trim = l2_line.trim();
                let l2_ind = l2_line.len() - l2_line.trim_start_matches(' ').len();
                if !l2_trim.is_empty() && !l2_trim.starts_with('#') && l2_ind == 0 {
                    break;
                }
                if !updated && l2_ind > 0 && l2_trim.starts_with(&format!("{l2}:")) {
                    out.push(l2_line.to_string());
                    idx += 1;
                    let child_indent = " ".repeat(l2_ind + 2);
                    let mut found_l3 = false;
                    while idx < lines.len() {
                        let l3_line = lines[idx];
                        let l3_trim = l3_line.trim();
                        let l3_ind = l3_line.len() - l3_line.trim_start_matches(' ').len();
                        if !l3_trim.is_empty() && !l3_trim.starts_with('#') && l3_ind <= l2_ind {
                            break;
                        }
                        if !found_l3
                            && l3_ind == l2_ind + 2
                            && l3_trim
                                .strip_prefix(l3)
                                .map(|r| r.trim_start().starts_with(':'))
                                .unwrap_or(false)
                        {
                            out.push(format!("{child_indent}{l3}: {formatted_val}"));
                            found_l3 = true;
                            updated = true;
                            idx += 1;
                            continue;
                        }
                        out.push(l3_line.to_string());
                        idx += 1;
                    }
                    if !found_l3 {
                        out.push(format!("{child_indent}{l3}: {formatted_val}"));
                        updated = true;
                    }
                    continue;
                }
                out.push(l2_line.to_string());
                idx += 1;
            }
            continue;
        }

        out.push(line.to_string());
        idx += 1;
    }

    let mut res = out.join("\n");
    if !res.ends_with('\n') {
        res.push('\n');
    }
    res
}

// ============================================================================
// SOUL.MD PARSER, ANCHORED PATHS & HYGIENE HEALER
// ============================================================================

pub fn parse_soul_markdown(soul_raw: &str) -> SoulAnatomy {
    let clean = sanitize_lf(soul_raw);
    let lines: Vec<&str> = clean.lines().collect();

    let mut headline = String::new();
    let mut opening_mission = String::new();
    let mut sections: Vec<SoulSectionSummary> = Vec::new();
    let mut current_sec: Option<(String, usize, usize, String)> = None;

    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("# ") && headline.is_empty() {
            headline = t.trim_start_matches("# ").trim().to_string();
            continue;
        }
        if t.starts_with("## ") {
            if let Some((h, ls, bc, prev)) = current_sec.take() {
                sections.push(SoulSectionSummary {
                    heading: h,
                    line_start: ls,
                    bullet_count: bc,
                    preview: prev,
                });
            }
            current_sec = Some((
                t.trim_start_matches("## ").trim().to_string(),
                i + 1,
                0,
                String::new(),
            ));
            continue;
        }
        if current_sec.is_none() && !t.is_empty() && !t.starts_with('#') && opening_mission.is_empty()
        {
            opening_mission = strip_markdown_emphasis(t);
        }
        if let Some((_, _, ref mut bc, ref mut prev)) = current_sec {
            if t.starts_with("- ") || t.starts_with("* ") {
                *bc += 1;
                if prev.is_empty() {
                    let cleaned_bullet = strip_markdown_emphasis(t.trim_start_matches(['-', '*', ' ']));
                    *prev = cleaned_bullet.chars().take(120).collect();
                }
            } else if !t.is_empty() && prev.is_empty() {
                *prev = strip_markdown_emphasis(t).chars().take(120).collect();
            }
        }
    }
    if let Some((h, ls, bc, prev)) = current_sec {
        sections.push(SoulSectionSummary {
            heading: h,
            line_start: ls,
            bullet_count: bc,
            preview: prev,
        });
    }

    // Detect strict privacy posture from SOUL.md instructions
    let lower = clean.to_lowercase();
    let mut privacy_keywords_found = Vec::new();
    for (needle, label) in [
        ("privacidade absoluta", "Privacidade Absoluta"),
        ("free tier", "Proibição de Free Tier"),
        ("extratos bancários", "Dados Bancários"),
        ("faturas", "Faturas Financeiras"),
        ("exames médicos", "Exames Médicos"),
        ("prontuários", "Prontuários de Saúde"),
        ("pii", "Proteção de PII"),
    ] {
        if lower.contains(needle) {
            privacy_keywords_found.push(label.to_string());
        }
    }
    let requires_strict_privacy = lower.contains("privacidade absoluta")
        || (lower.contains("nunca envie") && lower.contains("free tier"))
        || (lower.contains("100% localmente")
            && (lower.contains("extratos") || lower.contains("exames")));

    // Detect truncated duplicate adjacent lines (e.g. an unfinished bullet followed immediately by its completed version)
    let mut truncated_duplicate_lines = Vec::new();
    for i in 0..lines.len().saturating_sub(1) {
        let cur = lines[i].trim().replace('`', "");
        let nxt = lines[i + 1].trim().replace('`', "");
        if cur.len() >= 25 && nxt.len() > cur.len() + 8 && nxt.starts_with(&cur) {
            truncated_duplicate_lines.push(i + 1);
        }
    }

    // Extract anchored backtick paths (`C:\...`, `Z:\...`, `wiki/...`, `raw/...`)
    let anchored_paths = extract_anchored_paths(&clean);
    let estimated_tokens = (clean.chars().count() / 4).max(1);

    SoulAnatomy {
        headline,
        opening_mission,
        sections,
        requires_strict_privacy,
        privacy_keywords_found,
        estimated_tokens,
        anchored_paths,
        truncated_duplicate_lines,
    }
}

fn strip_markdown_emphasis(s: &str) -> String {
    s.replace("**", "").replace('`', "").trim().to_string()
}

fn extract_anchored_paths(soul_clean: &str) -> Vec<AnchoredPathRef> {
    let mut raw_candidates = Vec::new();
    let mut seen = BTreeSet::new();

    for segment in soul_clean.split('`').skip(1).step_by(2) {
        let cand = segment.trim();
        if cand.len() < 4 || cand.len() > 220 || cand.contains('\n') {
            continue;
        }
        let looks_like_path = cand.contains(":\\")
            || cand.contains(":/")
            || cand.starts_with("wiki/")
            || cand.starts_with("wiki\\")
            || cand.starts_with("raw/")
            || cand.starts_with("raw\\")
            || cand.ends_with("AGENTS.md");
        if looks_like_path && seen.insert(cand.to_string()) {
            raw_candidates.push(cand.to_string());
        }
    }

    // Find anchored absolute vault root if any (e.g. `C:\Users\...\Obsidian DB\`)
    let vault_root: Option<PathBuf> = raw_candidates.iter().find_map(|p| {
        if p.contains(":\\") || p.contains(":/") {
            let pb = PathBuf::from(p);
            if p.ends_with("AGENTS.md") {
                pb.parent().map(|par| par.to_path_buf())
            } else if let Some(idx) = p.to_lowercase().find("\\wiki\\") {
                Some(PathBuf::from(&p[..idx]))
            } else if pb.is_dir() {
                Some(pb)
            } else {
                None
            }
        } else {
            None
        }
    });

    raw_candidates
        .into_iter()
        .take(12)
        .map(|raw_p| {
            let resolved_pb = if raw_p.contains(":\\") || raw_p.contains(":/") {
                PathBuf::from(&raw_p)
            } else if let Some(ref vr) = vault_root {
                vr.join(raw_p.replace('/', "\\"))
            } else {
                PathBuf::from(&raw_p)
            };

            let exists_on_host = resolved_pb.exists();
            let lower = raw_p.to_lowercase();
            let kind = if lower.ends_with("agents.md") {
                "contract_agents_md"
            } else if lower.contains("wiki") {
                "wiki_note"
            } else if lower.contains("raw") {
                "raw_vault"
            } else if resolved_pb.is_dir() || raw_p.ends_with('\\') || raw_p.ends_with('/') {
                "directory"
            } else {
                "other"
            };

            AnchoredPathRef {
                raw_path: raw_p,
                resolved_path: resolved_pb.to_string_lossy().to_string(),
                kind: kind.to_string(),
                exists_on_host,
            }
        })
        .collect()
}

/// Removes truncated duplicate prefix lines in `SOUL.md` while keeping 100% of real rules intact.
pub fn heal_soul_truncated_duplicates(soul_raw: &str) -> (String, usize) {
    let clean = sanitize_lf(soul_raw);
    let lines: Vec<&str> = clean.lines().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut removed = 0usize;

    for i in 0..lines.len() {
        if i + 1 < lines.len() {
            let cur = lines[i].trim().replace('`', "");
            let nxt = lines[i + 1].trim().replace('`', "");
            if cur.len() >= 25 && nxt.len() > cur.len() + 8 && nxt.starts_with(&cur) {
                removed += 1;
                continue;
            }
        }
        out.push(lines[i]);
    }

    let mut res = out.join("\n");
    if !res.ends_with('\n') {
        res.push('\n');
    }
    (res, removed)
}

/// Synthesizes a friendly Portuguese description (`<= 180` chars) from `SOUL.md` when `profile.yaml` has a bare slug.
pub fn synthesize_description_from_soul(slug: &str, anatomy: &SoulAnatomy) -> String {
    let clean_h = anatomy
        .headline
        .trim_start_matches("SOUL.md")
        .trim_start_matches(['—', '-', ':', ' '])
        .trim();

    if !anatomy.opening_mission.is_empty() {
        let first_sentence = anatomy
            .opening_mission
            .split(". ")
            .next()
            .unwrap_or(&anatomy.opening_mission)
            .trim()
            .trim_end_matches('.');
        if !clean_h.is_empty() && !first_sentence.to_lowercase().contains(&clean_h.to_lowercase()) {
            let combined = format!("{clean_h} — {first_sentence}.");
            if combined.chars().count() <= 210 {
                return combined;
            }
        }
        let clipped: String = first_sentence.chars().take(185).collect();
        return format!("{clipped}.");
    }

    if !clean_h.is_empty() {
        return format!("Assistente especializado: {clean_h}.");
    }

    format!("Assistente especializado para o contexto de {slug}.")
}

// ============================================================================
// GOVERNANCE & PRIVACY RADAR EVALUATION
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn evaluate_persona_governance(
    slug: &str,
    is_default: bool,
    description: &str,
    model_default: &str,
    souls_mcp_connected: bool,
    has_free_tier_exposure: bool,
    anatomy: &SoulAnatomy,
) -> (u8, String, String, String, Vec<PersonaGovernanceFinding>) {
    let mut score: i32 = 100;
    let mut findings = Vec::new();

    // 1. Critical: Strict Privacy required by SOUL.md vs Free-Tier model/fallback in config.yaml
    if anatomy.requires_strict_privacy && has_free_tier_exposure {
        score -= 40;
        findings.push(PersonaGovernanceFinding {
            code: "PRIVACY_FREE_MODEL_CONFLICT".to_string(),
            severity: "critical".to_string(),
            title: "Modelo gratuito ativo em Persona de dados sensíveis".to_string(),
            explanation: format!(
                "O SOUL.md de '{}' proíbe enviar dados sensíveis para serviços gratuitos (Free Tier), mas o modelo atual é '{}' (ou há fallback gratuito ativo).",
                slug,
                if model_default.is_empty() { "não definido" } else { model_default }
            ),
            auto_fixable: true,
        });
    }

    // 2. Warning: Souls Engine MCP server not linked in config.yaml
    if !souls_mcp_connected {
        score -= 22;
        findings.push(PersonaGovernanceFinding {
            code: "SOULS_MCP_DISCONNECTED".to_string(),
            severity: "warning".to_string(),
            title: "Desconectada da aceleração local do Souls Engine".to_string(),
            explanation: format!(
                "A Persona '{}' ainda não possui o vínculo com o Souls Engine (`127.0.0.1:9123/mcp`), ficando sem memória profunda, leitura cirúrgica de código e roteamento local.",
                slug
            ),
            auto_fixable: true,
        });
    }

    // 3. Info: Generic or empty description in profile.yaml (for non-default personas)
    let desc_trim = description.trim();
    if !is_default && (desc_trim.is_empty() || desc_trim.eq_ignore_ascii_case(slug) || desc_trim.chars().count() < 16) {
        score -= 10;
        findings.push(PersonaGovernanceFinding {
            code: "PROFILE_DESC_GENERIC".to_string(),
            severity: "info".to_string(),
            title: "Resumo do cartão em branco ou genérico".to_string(),
            explanation: format!(
                "O arquivo profile.yaml de '{}' está apenas com '{}'. Podemos preencher automaticamente um resumo claro a partir do SOUL.md.",
                slug,
                if desc_trim.is_empty() { "(vazio)" } else { desc_trim }
            ),
            auto_fixable: true,
        });
    }

    // 4. Info: Truncated duplicate line in SOUL.md
    if !anatomy.truncated_duplicate_lines.is_empty() {
        score -= 6;
        let ln = anatomy.truncated_duplicate_lines[0];
        findings.push(PersonaGovernanceFinding {
            code: "SOUL_DUPLICATE_TRUNCATED_LINE".to_string(),
            severity: "info".to_string(),
            title: format!("Frase incompleta repetida no SOUL.md (linha {ln})"),
            explanation: "Encontramos uma linha cortada logo acima da versão completa da mesma frase. A limpeza remove apenas a sobra sem alterar nenhuma regra.".to_string(),
            auto_fixable: true,
        });
    }

    // 5. Info: Missing anchored file/folder on host (only report non-raw concrete files like AGENTS.md / wiki)
    for ap in &anatomy.anchored_paths {
        if !ap.exists_on_host && ap.kind == "contract_agents_md" {
            score -= 8;
            findings.push(PersonaGovernanceFinding {
                code: "ANCHORED_PATH_MISSING".to_string(),
                severity: "info".to_string(),
                title: "Contrato referenciado não encontrado neste PC".to_string(),
                explanation: format!(
                    "O SOUL.md faz referência a '{}', mas esse arquivo não foi localizado no disco atual.",
                    ap.raw_path
                ),
                auto_fixable: false,
            });
            break;
        }
    }

    let final_score = score.clamp(20, 100) as u8;
    let has_crit = findings.iter().any(|f| f.severity == "critical");
    let has_warn = findings.iter().any(|f| f.severity == "warning");

    let (status_badge, status_label, summary) = if has_crit {
        (
            "privacy_alert".to_string(),
            "🔒 Alerta de Privacidade".to_string(),
            findings[0].explanation.clone(),
        )
    } else if has_warn {
        (
            "needs_attention".to_string(),
            "🔌 Falta Conectar Motor".to_string(),
            findings[0].explanation.clone(),
        )
    } else if !findings.is_empty() {
        (
            "aligned".to_string(),
            "✓ Alinhada (1 dica)".to_string(),
            findings[0].explanation.clone(),
        )
    } else {
        (
            "aligned".to_string(),
            "✓ 100% Alinhada".to_string(),
            "Identidade, modelo, privacidade e aceleração local em perfeita harmonia.".to_string(),
        )
    };

    (final_score, status_badge, status_label, summary, findings)
}

// ============================================================================
// ECOSYSTEM SCANNER & PERSONA DETAIL INSPECTOR
// ============================================================================

fn pick_default_accent_color(slug: &str, configured: Option<String>) -> String {
    if let Some(c) = configured {
        let trimmed = c.trim();
        if trimmed.starts_with('#') {
            return trimmed.to_string();
        }
    }
    match slug {
        "default" => "#8b5cf6".to_string(),
        "financas" => "#10b981".to_string(),
        "saude" => "#06b6d4".to_string(),
        "hiperfocos" => "#f59e0b".to_string(),
        _ => "#38bdf8".to_string(),
    }
}

fn count_persona_skills(skills_dir: &Path) -> (usize, usize, Vec<PersonaSkillBrief>) {
    let mut active = 0usize;
    let mut paused = 0usize;
    let mut briefs = Vec::new();

    if !skills_dir.is_dir() {
        return (0, 0, briefs);
    }

    let mut stack = vec![(skills_dir.to_path_buf(), None::<String>, 0usize)];
    while let Some((dir, cat, depth)) = stack.pop() {
        if depth > 3 {
            continue;
        }
        let Ok(rd) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if p.is_dir() {
                let active_md = p.join("SKILL.md");
                let disabled_md = p.join("SKILL.md.disabled");
                if active_md.is_file() || disabled_md.is_file() {
                    let is_en = active_md.is_file();
                    let target_md = if is_en { active_md } else { disabled_md };
                    let raw = fs::read_to_string(&target_md).unwrap_or_default();
                    let anat = skills_hub::parse_skill_markdown(&raw);
                    let slug = skills_hub::normalize_skill_slug(&name);
                    if is_en {
                        active += 1;
                    } else {
                        paused += 1;
                    }
                    briefs.push(PersonaSkillBrief {
                        slug,
                        description: anat.description,
                        enabled: is_en,
                        category: cat.clone(),
                    });
                } else {
                    let next_cat = if depth == 0 { Some(name) } else { cat.clone() };
                    stack.push((p, next_cat, depth + 1));
                }
            }
        }
    }

    briefs.sort_by(|a, b| a.slug.cmp(&b.slug));
    (active, paused, briefs)
}

fn scan_single_persona_dir(
    roots: &PersonasHubRoots,
    slug: &str,
    dir: &Path,
    is_default: bool,
) -> PersonaSummaryRecord {
    let profile_yaml = fs::read_to_string(dir.join("profile.yaml")).unwrap_or_default();
    let config_yaml = fs::read_to_string(dir.join("config.yaml")).unwrap_or_default();
    let soul_raw = fs::read_to_string(dir.join("SOUL.md")).unwrap_or_default();

    let anatomy = parse_soul_markdown(&soul_raw);

    let ui_title = yaml_get_nested(&profile_yaml, &["ui_meta", "hermes-bots", "title"])
        .or_else(|| yaml_get_nested(&profile_yaml, &["display_name"]))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if is_default {
                "Hermes Principal".to_string()
            } else {
                slug.to_string()
            }
        });

    let raw_desc = yaml_get_nested(&profile_yaml, &["description"]).unwrap_or_default();
    let subtitle_role = if !anatomy.headline.is_empty() {
        anatomy
            .headline
            .replace("&amp;", "&")
            .trim_start_matches("SOUL.md")
            .trim_start_matches(['—', '-', ':', ' '])
            .trim()
            .to_string()
    } else if is_default {
        "Orquestrador Principal & Identidade Global".to_string()
    } else {
        "Persona Especializada".to_string()
    };

    let display_desc = if !raw_desc.trim().is_empty() && !raw_desc.trim().eq_ignore_ascii_case(slug)
    {
        raw_desc.replace("&amp;", "&")
    } else if !anatomy.opening_mission.is_empty() {
        anatomy
            .opening_mission
            .replace("&amp;", "&")
            .chars()
            .take(190)
            .collect()
    } else {
        subtitle_role.clone()
    };

    let accent_color = pick_default_accent_color(
        slug,
        yaml_get_nested(&profile_yaml, &["ui_meta", "hermes-bots", "color"]),
    );

    let has_custom_avatar = dir.join("assets").join("avatar.png").is_file();

    let model_default = yaml_get_nested(&config_yaml, &["model", "default"]).unwrap_or_default();
    let model_provider = yaml_get_nested(&config_yaml, &["model", "provider"]).unwrap_or_default();
    let model_base_url = yaml_get_nested(&config_yaml, &["model", "base_url"]).unwrap_or_default();
    let reasoning_effort = yaml_get_nested(&config_yaml, &["agent", "reasoning_effort"])
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "medium".to_string());
    let max_turns = yaml_get_nested(&config_yaml, &["agent", "max_turns"])
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(500);
    let memory_enabled = yaml_get_nested(&config_yaml, &["memory", "memory_enabled"])
        .map(|v| v == "true")
        .unwrap_or(true);

    let souls_mcp_connected = yaml_has_block_path(&config_yaml, &["mcp_servers", "souls_engine"]);
    let has_free_tier_exposure = model_default.to_lowercase().contains(":free")
        || config_yaml.contains("provider: opencode-free")
        || config_yaml.contains("provider: \"opencode-free\"");

    let (active_skills_count, paused_skills_count, _) = count_persona_skills(&dir.join("skills"));
    let history_snapshots_count = list_persona_history(roots, slug).len();

    let (alignment_score, status_badge, status_label, governance_summary, findings) =
        evaluate_persona_governance(
            slug,
            is_default,
            &raw_desc,
            &model_default,
            souls_mcp_connected,
            has_free_tier_exposure,
            &anatomy,
        );

    PersonaSummaryRecord {
        slug: slug.to_string(),
        is_default,
        display_title: ui_title,
        subtitle_role,
        description: display_desc,
        accent_color,
        has_custom_avatar,
        root_path: dir.to_string_lossy().to_string(),
        model_default,
        model_provider,
        model_base_url,
        reasoning_effort,
        max_turns,
        memory_enabled,
        souls_mcp_connected,
        has_free_tier_exposure,
        requires_strict_privacy: anatomy.requires_strict_privacy,
        soul_tokens: anatomy.estimated_tokens,
        soul_sections_count: anatomy.sections.len(),
        active_skills_count,
        paused_skills_count,
        anchored_paths_count: anatomy.anchored_paths.len(),
        history_snapshots_count,
        alignment_score,
        status_badge,
        status_label,
        governance_summary,
        findings,
    }
}

/// Scans all active and archived Hermes Personas on the host.
pub fn scan_personas_ecosystem(roots: &PersonasHubRoots) -> PersonasEcosystemOverview {
    let hermes_installed = roots.is_hermes_installed();
    let mut personas = Vec::new();

    if hermes_installed {
        // 1. Default (Master) profile
        personas.push(scan_single_persona_dir(
            roots,
            "default",
            &roots.hermes_root_dir,
            true,
        ));

        // 2. Specialized profiles under `profiles/*`
        let profs_dir = roots.hermes_root_dir.join("profiles");
        if let Ok(rd) = fs::read_dir(&profs_dir) {
            let mut slugs: Vec<(String, PathBuf)> = rd
                .flatten()
                .filter_map(|e| {
                    let p = e.path();
                    let name = e.file_name().to_string_lossy().to_string();
                    if p.is_dir()
                        && !name.starts_with('.')
                        && (p.join("SOUL.md").is_file()
                            || p.join("config.yaml").is_file()
                            || p.join("profile.yaml").is_file())
                    {
                        Some((name, p))
                    } else {
                        None
                    }
                })
                .collect();
            slugs.sort_by(|a, b| a.0.cmp(&b.0));

            for (slug, path) in slugs {
                personas.push(scan_single_persona_dir(roots, &slug, &path, false));
            }
        }
    }

    // Recommended safe private model: inherit from `default` if non-free, else `deepseek/deepseek-chat`
    let (rec_model, rec_provider) = personas
        .first()
        .filter(|d| !d.model_default.is_empty() && !d.model_default.contains(":free"))
        .map(|d| {
            (
                d.model_default.clone(),
                if d.model_provider.is_empty() {
                    "nous".to_string()
                } else {
                    d.model_provider.clone()
                },
            )
        })
        .unwrap_or_else(|| ("deepseek/deepseek-chat".to_string(), "nous".to_string()));

    let archived_personas = list_archived_personas(roots);

    let total_active = personas.len();
    let souls_connected_count = personas.iter().filter(|p| p.souls_mcp_connected).count();
    let privacy_alerts_count = personas
        .iter()
        .filter(|p| p.status_badge == "privacy_alert")
        .count();
    let attention_needed_count = personas
        .iter()
        .map(|p| p.findings.len())
        .sum::<usize>();
    let avg_alignment_score = if total_active > 0 {
        personas.iter().map(|p| p.alignment_score as f64).sum::<f64>() / (total_active as f64)
    } else {
        100.0
    };
    let total_persona_skills_links = personas.iter().map(|p| p.active_skills_count).sum();

    PersonasEcosystemOverview {
        hermes_installed,
        hermes_root_dir: roots.hermes_root_dir.to_string_lossy().to_string(),
        total_active_personas: total_active,
        archived_personas_count: archived_personas.len(),
        souls_connected_count,
        privacy_alerts_count,
        attention_needed_count,
        avg_alignment_score,
        total_persona_skills_links,
        recommended_private_model: rec_model,
        recommended_private_provider: rec_provider,
        personas,
        archived_personas,
    }
}

/// Reads passive Read-Only memory files (`MEMORY.md`, `USER.md`, `memories/*`).
/// CONSTITUTIONAL LAW (`AGENTS.md` §4): NEVER writes to these files!
fn collect_read_only_memories(persona_dir: &Path) -> Vec<ReadOnlyMemoryNote> {
    let mut notes = Vec::new();
    let mut seen = BTreeSet::new();

    for candidate in [
        persona_dir.join("MEMORY.md"),
        persona_dir.join("USER.md"),
        persona_dir.join("memories").join("MEMORY.md"),
        persona_dir.join("memories").join("USER.md"),
    ] {
        if candidate.is_file() {
            let full = candidate.to_string_lossy().to_string();
            if seen.insert(full) {
                if let Ok(raw) = fs::read_to_string(&candidate) {
                    let sz = candidate.metadata().map(|m| m.len()).unwrap_or(0);
                    let rel = candidate
                        .strip_prefix(persona_dir)
                        .unwrap_or(&candidate)
                        .to_string_lossy()
                        .to_string();
                    notes.push(ReadOnlyMemoryNote {
                        filename: candidate
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        rel_path: rel,
                        size_bytes: sz,
                        content_preview: sanitize_lf(&raw).chars().take(2400).collect(),
                        read_only_constitutional: true,
                    });
                }
            }
        }
    }

    let mem_dir = persona_dir.join("memories");
    if let Ok(rd) = fs::read_dir(&mem_dir) {
        for entry in rd.flatten().take(8) {
            let p = entry.path();
            if p.is_file() {
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if matches!(ext.as_str(), "md" | "txt") {
                    let full = p.to_string_lossy().to_string();
                    if seen.insert(full) {
                        if let Ok(raw) = fs::read_to_string(&p) {
                            let sz = p.metadata().map(|m| m.len()).unwrap_or(0);
                            notes.push(ReadOnlyMemoryNote {
                                filename: entry.file_name().to_string_lossy().to_string(),
                                rel_path: format!("memories/{}", entry.file_name().to_string_lossy()),
                                size_bytes: sz,
                                content_preview: sanitize_lf(&raw).chars().take(2400).collect(),
                                read_only_constitutional: true,
                            });
                        }
                    }
                }
            }
        }
    }

    notes
}

fn collect_anchored_contracts(anatomy: &SoulAnatomy) -> Vec<AnchoredContractPreview> {
    let mut contracts = Vec::new();
    for ap in &anatomy.anchored_paths {
        if ap.kind == "contract_agents_md" || ap.kind == "wiki_note" {
            let pb = PathBuf::from(&ap.resolved_path);
            let (sz, preview) = if pb.is_file() {
                let s = pb.metadata().map(|m| m.len()).unwrap_or(0);
                let txt = fs::read_to_string(&pb)
                    .map(|r| sanitize_lf(&r).chars().take(2200).collect())
                    .unwrap_or_default();
                (s, txt)
            } else {
                (0, String::new())
            };
            contracts.push(AnchoredContractPreview {
                label: pb
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| ap.raw_path.clone()),
                path: ap.resolved_path.clone(),
                exists: ap.exists_on_host,
                size_bytes: sz,
                content_preview: preview,
            });
        }
    }
    contracts
}

/// Inspects full details of a Persona for the Dual-Mode Workbench.
pub fn inspect_persona_detail(roots: &PersonasHubRoots, slug: &str) -> Result<Value, String> {
    let clean_slug = normalize_persona_slug(slug);
    let is_default = clean_slug == "default";
    let dir = roots.persona_dir(&clean_slug)?;
    if !dir.is_dir() {
        return Err(format!(
            "A Persona '{clean_slug}' não foi encontrada em {}.",
            dir.display()
        ));
    }

    let summary = scan_single_persona_dir(roots, &clean_slug, &dir, is_default);
    let soul_raw = sanitize_lf(&fs::read_to_string(dir.join("SOUL.md")).unwrap_or_default());
    let profile_yaml = sanitize_lf(&fs::read_to_string(dir.join("profile.yaml")).unwrap_or_default());
    let anatomy = parse_soul_markdown(&soul_raw);
    let (_, _, persona_skills) = count_persona_skills(&dir.join("skills"));
    let memories = collect_read_only_memories(&dir);
    let anchored_contracts = collect_anchored_contracts(&anatomy);
    let history = list_persona_history(roots, &clean_slug);

    Ok(json!({
        "status": "success",
        "persona": summary,
        "raw_profile_description": yaml_get_nested(&profile_yaml, &["description"]).unwrap_or_default(),
        "soul_content": soul_raw,
        "anatomy": anatomy,
        "skills": persona_skills,
        "memories_read_only": memories,
        "anchored_contracts": anchored_contracts,
        "history": history
    }))
}

/// Safely reads avatar PNG bytes for a Persona (`assets/avatar.png`).
pub fn read_persona_avatar_bytes(roots: &PersonasHubRoots, slug: &str) -> Option<Vec<u8>> {
    let clean_slug = normalize_persona_slug(slug);
    let dir = roots.persona_dir(&clean_slug).ok()?;
    let avatar_path = dir.join("assets").join("avatar.png");
    if avatar_path.is_file() {
        fs::read(avatar_path).ok()
    } else {
        None
    }
}

// ============================================================================
// SNAPSHOT HISTORY, MYERS DIFF, ATOMIC UPDATE & 1-CLICK ALIGNMENT
// ============================================================================

pub fn compute_persona_text_diff(
    file_name: &str,
    before_raw: &str,
    after_raw: &str,
) -> PersonaFileDiffPreview {
    let before = sanitize_lf(before_raw);
    let after = sanitize_lf(after_raw);

    let myers = compute_safe_myers_diff(&before, &after).ok();
    let (unchanged, additions, deletions) = match &myers {
        Some(p) => (p.stats.unchanged, p.stats.additions, p.stats.deletions),
        None => (before == after, 0, 0),
    };

    let mut deleted_old = BTreeSet::new();
    let mut inserted_new = BTreeSet::new();
    if let Some(ref patch) = myers {
        for ch in &patch.changes {
            match ch.tag {
                DiffTag::Delete => {
                    if let Some(ol) = ch.old_line {
                        deleted_old.insert(ol);
                    }
                }
                DiffTag::Insert => {
                    if let Some(nl) = ch.new_line {
                        inserted_new.insert(nl);
                    }
                }
                DiffTag::Equal => {}
            }
        }
    }

    let old_vec: Vec<&str> = before.lines().collect();
    let new_vec: Vec<&str> = after.lines().collect();
    let mut lines = Vec::new();
    let mut i = 1usize;
    let mut j = 1usize;

    while (i <= old_vec.len() || j <= new_vec.len()) && lines.len() < 360 {
        if i <= old_vec.len() && deleted_old.contains(&i) {
            lines.push(VisualDiffLine {
                tag: "delete".to_string(),
                old_line: Some(i),
                new_line: None,
                text: old_vec[i - 1].to_string(),
            });
            i += 1;
        } else if j <= new_vec.len() && inserted_new.contains(&j) {
            lines.push(VisualDiffLine {
                tag: "insert".to_string(),
                old_line: None,
                new_line: Some(j),
                text: new_vec[j - 1].to_string(),
            });
            j += 1;
        } else if i <= old_vec.len() && j <= new_vec.len() {
            lines.push(VisualDiffLine {
                tag: "equal".to_string(),
                old_line: Some(i),
                new_line: Some(j),
                text: new_vec[j - 1].to_string(),
            });
            i += 1;
            j += 1;
        } else {
            break;
        }
    }

    PersonaFileDiffPreview {
        file_name: file_name.to_string(),
        additions,
        deletions,
        unchanged,
        lines,
    }
}

pub fn record_persona_snapshot(
    roots: &PersonasHubRoots,
    slug: &str,
    target_file: &str,
    content: &str,
    reason: &str,
) -> Result<String, String> {
    let clean_slug = normalize_persona_slug(slug);
    let safe_file = match target_file {
        "SOUL.md" => "SOUL.md",
        "config.yaml" => "config.yaml",
        "profile.yaml" => "profile.yaml",
        _ => return Err("Arquivo de snapshot não suportado.".to_string()),
    };
    let clean_reason: String = reason
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .take(24)
        .collect();

    let dir = roots.history_dir().join(&clean_slug);
    fs::create_dir_all(&dir).map_err(|e| format!("Erro ao criar pasta de histórico: {e}"))?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let snapshot_id = format!("{ts}__{safe_file}__{clean_reason}.bak");
    let full_path = dir.join(&snapshot_id);

    let lf = sanitize_lf(content);
    fs::write(&full_path, lf.as_bytes())
        .map_err(|e| format!("Falha ao gravar snapshot de Persona: {e}"))?;
    Ok(snapshot_id)
}

pub fn list_persona_history(roots: &PersonasHubRoots, slug: &str) -> Vec<PersonaHistorySnapshot> {
    let clean_slug = normalize_persona_slug(slug);
    let dir = roots.history_dir().join(&clean_slug);
    let Ok(rd) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut list = Vec::new();
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let fname = entry.file_name().to_string_lossy().to_string();
        if !fname.ends_with(".bak") {
            continue;
        }
        let stem = fname.trim_end_matches(".bak").to_string();
        let parts: Vec<&str> = stem.splitn(3, "__").collect();
        if parts.len() < 3 {
            continue;
        }
        let ts_ms = parts[0].parse::<u64>().unwrap_or(0);
        let sz = p.metadata().map(|m| m.len()).unwrap_or(0);
        list.push(PersonaHistorySnapshot {
            snapshot_id: fname,
            slug: clean_slug.clone(),
            target_file: parts[1].to_string(),
            timestamp_epoch: ts_ms / 1000,
            size_bytes: sz,
            reason: parts[2].to_string(),
        });
    }

    list.sort_by(|a, b| b.snapshot_id.cmp(&a.snapshot_id));
    list
}

/// Applies safe, atomic updates to a Persona (`profile.yaml`, `config.yaml`, and/or `SOUL.md`),
/// recording recovery snapshots beforehand and NEVER touching `MEMORY.md` / `USER.md` (`AGENTS.md` §4).
pub fn update_persona(roots: &PersonasHubRoots, req: PersonaUpdateRequest) -> Result<Value, String> {
    let clean_slug = normalize_persona_slug(&req.slug);
    let is_default = clean_slug == "default";
    let dir = roots.persona_dir(&clean_slug)?;
    if !dir.is_dir() {
        return Err(format!("A Persona '{clean_slug}' não existe."));
    }

    let reason = req.reason.as_deref().unwrap_or("painel_visual");
    let mut updated_files = Vec::new();

    // 1. `profile.yaml` updates
    if req.display_title.is_some() || req.description.is_some() || req.accent_color.is_some() {
        let prof_path = dir.join("profile.yaml");
        let before = fs::read_to_string(&prof_path).unwrap_or_default();
        let after = yaml_update_profile_metadata(
            &before,
            is_default,
            req.display_title.as_deref(),
            req.description.as_deref(),
            req.accent_color.as_deref(),
        );
        if sanitize_lf(&before) != after {
            if !before.is_empty() {
                let _ = record_persona_snapshot(roots, &clean_slug, "profile.yaml", &before, reason);
            }
            fs::write(&prof_path, after.as_bytes())
                .map_err(|e| format!("Falha ao salvar profile.yaml: {e}"))?;
            updated_files.push("profile.yaml");
        }
    }

    // 2. `config.yaml` hierarchical block updates (`AGENTS.md` §8.1)
    let wants_cfg_change = req.model_default.is_some()
        || req.model_provider.is_some()
        || req.reasoning_effort.is_some()
        || req.max_turns.is_some()
        || req.memory_enabled.is_some()
        || req.souls_mcp_connected.is_some()
        || req.sanitize_free_fallbacks == Some(true);

    if wants_cfg_change {
        let cfg_path = dir.join("config.yaml");
        let before = fs::read_to_string(&cfg_path).unwrap_or_default();
        let mut cur = sanitize_lf(&before);

        if let Some(ref mdl) = req.model_default {
            let trimmed = mdl.trim();
            if !trimmed.is_empty() {
                cur = yaml_set_nested_scalar(&cur, "model", "default", &format_yaml_quoted(trimmed));
            }
        }
        if let Some(ref prov) = req.model_provider {
            let trimmed = prov.trim();
            if !trimmed.is_empty() {
                cur = yaml_set_nested_scalar(&cur, "model", "provider", &format_yaml_quoted(trimmed));
            }
        }
        if let Some(ref effort) = req.reasoning_effort {
            let clean_eff = match effort.trim().to_lowercase().as_str() {
                "low" | "medium" | "high" => effort.trim().to_lowercase(),
                _ => "high".to_string(),
            };
            cur = yaml_set_nested_scalar(&cur, "agent", "reasoning_effort", &clean_eff);
        }
        if let Some(turns) = req.max_turns {
            let clamped = turns.clamp(10, 2000);
            cur = yaml_set_nested_scalar(&cur, "agent", "max_turns", &clamped.to_string());
        }
        if let Some(mem_on) = req.memory_enabled {
            cur = yaml_set_nested_scalar(
                &cur,
                "memory",
                "memory_enabled",
                if mem_on { "true" } else { "false" },
            );
        }
        if let Some(mcp_on) = req.souls_mcp_connected {
            cur = yaml_set_souls_mcp_connection(&cur, mcp_on);
        }
        if req.sanitize_free_fallbacks == Some(true) {
            cur = yaml_remove_free_fallbacks(&cur);
        }

        if sanitize_lf(&before) != cur {
            if !before.is_empty() {
                let _ = record_persona_snapshot(roots, &clean_slug, "config.yaml", &before, reason);
            }
            fs::write(&cfg_path, cur.as_bytes())
                .map_err(|e| format!("Falha ao salvar config.yaml: {e}"))?;
            updated_files.push("config.yaml");
        }
    }

    // 3. `SOUL.md` updates
    if let Some(ref soul_md) = req.soul_content {
        let soul_path = dir.join("SOUL.md");
        let before = fs::read_to_string(&soul_path).unwrap_or_default();
        let mut after = sanitize_lf(soul_md);
        if !after.ends_with('\n') {
            after.push('\n');
        }
        if sanitize_lf(&before) != after {
            if !before.is_empty() {
                let _ = record_persona_snapshot(roots, &clean_slug, "SOUL.md", &before, reason);
            }
            fs::write(&soul_path, after.as_bytes())
                .map_err(|e| format!("Falha ao salvar SOUL.md: {e}"))?;
            updated_files.push("SOUL.md");
        }
    }

    let summary = scan_single_persona_dir(roots, &clean_slug, &dir, is_default);
    Ok(json!({
        "status": "success",
        "slug": clean_slug,
        "updated_files": updated_files,
        "persona": summary
    }))
}

/// Previews (`dry_run: true`) or applies (`dry_run: false`) 1-Click Privacy & Symbiosis Alignment!
pub fn preview_or_apply_persona_alignment(
    roots: &PersonasHubRoots,
    req: PersonaAlignRequest,
) -> Result<Value, String> {
    let clean_slug = normalize_persona_slug(&req.slug);
    let is_default = clean_slug == "default";
    let dir = roots.persona_dir(&clean_slug)?;
    if !dir.is_dir() {
        return Err(format!("A Persona '{clean_slug}' não foi encontrada."));
    }

    let overview = scan_personas_ecosystem(roots);
    let target_model = req
        .preferred_model
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(overview.recommended_private_model);
    let target_provider = req
        .preferred_provider
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(overview.recommended_private_provider);

    let before_cfg = sanitize_lf(&fs::read_to_string(dir.join("config.yaml")).unwrap_or_default());
    let before_prof = sanitize_lf(&fs::read_to_string(dir.join("profile.yaml")).unwrap_or_default());
    let before_soul = sanitize_lf(&fs::read_to_string(dir.join("SOUL.md")).unwrap_or_default());

    let anatomy = parse_soul_markdown(&before_soul);
    let cur_model = yaml_get_nested(&before_cfg, &["model", "default"]).unwrap_or_default();
    let has_free = cur_model.to_lowercase().contains(":free")
        || before_cfg.contains("provider: opencode-free")
        || before_cfg.contains("provider: \"opencode-free\"");

    let mut actions_explained = Vec::new();
    let mut after_cfg = before_cfg.clone();

    // Fix 1: Privacy conflict on Free Tier
    if anatomy.requires_strict_privacy && has_free {
        if cur_model.to_lowercase().contains(":free") || cur_model.is_empty() {
            after_cfg = yaml_set_nested_scalar(
                &after_cfg,
                "model",
                "default",
                &format_yaml_quoted(&target_model),
            );
            after_cfg = yaml_set_nested_scalar(
                &after_cfg,
                "model",
                "provider",
                &format_yaml_quoted(&target_provider),
            );
        }
        after_cfg = yaml_remove_free_fallbacks(&after_cfg);
        actions_explained.push(format!(
            "Substituir modelo gratuito por '{}' ({}) e remover provedores Free Tier dos fallbacks.",
            target_model, target_provider
        ));
    }

    // Fix 2: Connect Souls Engine MCP
    if !yaml_has_block_path(&after_cfg, &["mcp_servers", "souls_engine"]) {
        after_cfg = yaml_set_souls_mcp_connection(&after_cfg, true);
        actions_explained.push(
            "Conectar o servidor local Souls Engine (`http://127.0.0.1:9123/mcp`) para liberar memória e ferramentas rápidas.".to_string(),
        );
    }

    // Fix 3: Synthesize missing/generic description in profile.yaml
    let cur_desc = yaml_get_nested(&before_prof, &["description"]).unwrap_or_default();
    let mut after_prof = before_prof.clone();
    if !is_default
        && (cur_desc.trim().is_empty()
            || cur_desc.trim().eq_ignore_ascii_case(&clean_slug)
            || cur_desc.trim().chars().count() < 16)
    {
        let synth = synthesize_description_from_soul(&clean_slug, &anatomy);
        after_prof = yaml_update_profile_metadata(&before_prof, is_default, None, Some(&synth), None);
        actions_explained.push(format!(
            "Preencher resumo de apresentação no cartão da Persona: \"{}\"",
            synth
        ));
    }

    // Fix 4: Clean truncated duplicate line in SOUL.md
    let (after_soul, removed_dups) = heal_soul_truncated_duplicates(&before_soul);
    if removed_dups > 0 {
        actions_explained.push(format!(
            "Remover {} linha(s) incompleta(s)/duplicada(s) no SOUL.md preservando a regra completa.",
            removed_dups
        ));
    }

    let mut file_diffs = Vec::new();
    if before_cfg != after_cfg {
        file_diffs.push(compute_persona_text_diff("config.yaml", &before_cfg, &after_cfg));
    }
    if before_prof != after_prof {
        file_diffs.push(compute_persona_text_diff("profile.yaml", &before_prof, &after_prof));
    }
    if before_soul != after_soul {
        file_diffs.push(compute_persona_text_diff("SOUL.md", &before_soul, &after_soul));
    }

    if !req.dry_run && !file_diffs.is_empty() {
        if before_cfg != after_cfg {
            let _ = record_persona_snapshot(roots, &clean_slug, "config.yaml", &before_cfg, "auto_align");
            fs::write(dir.join("config.yaml"), after_cfg.as_bytes())
                .map_err(|e| format!("Erro ao gravar config.yaml: {e}"))?;
        }
        if before_prof != after_prof {
            let _ = record_persona_snapshot(roots, &clean_slug, "profile.yaml", &before_prof, "auto_align");
            fs::write(dir.join("profile.yaml"), after_prof.as_bytes())
                .map_err(|e| format!("Erro ao gravar profile.yaml: {e}"))?;
        }
        if before_soul != after_soul {
            let _ = record_persona_snapshot(roots, &clean_slug, "SOUL.md", &before_soul, "auto_align");
            fs::write(dir.join("SOUL.md"), after_soul.as_bytes())
                .map_err(|e| format!("Erro ao gravar SOUL.md: {e}"))?;
        }
    }

    let updated_persona = scan_single_persona_dir(roots, &clean_slug, &dir, is_default);

    Ok(json!({
        "status": "success",
        "slug": clean_slug,
        "dry_run": req.dry_run,
        "actions_explained": actions_explained,
        "changed_files": file_diffs,
        "persona": updated_persona
    }))
}

// ============================================================================
// PERSONA STUDIO: CREATE / CLONE, SOUL REFINEMENT, ARCHIVE & ROLLBACK
// ============================================================================

pub fn build_deterministic_soul_markdown(
    display_title: &str,
    description: &str,
    template_kind: &str,
    goal_prompt: &str,
    strict_privacy: bool,
) -> String {
    let title = if display_title.trim().is_empty() {
        "Assistente Especializado"
    } else {
        display_title.trim()
    };
    let goal = if goal_prompt.trim().is_empty() {
        description.trim()
    } else {
        goal_prompt.trim()
    };

    let domain_bullets = match template_kind {
        "finance" => "- Auditar receitas, despesas fixas, assinaturas recorrentes e metas de reserva/investimento.\n- Classificar valores confirmados como **FACTO** e projeções futuras como **ESTIMATIVA**.",
        "health" => "- Acompanhar indicadores de sono, energia, exames e hábitos preventivos sem alarmismo.\n- Propor micro-ajustes práticos baseados em evidências e histórico pessoal.",
        "studies" => "- Estruturar planos de estudo em ciclos claros, resumos enxutos e conexões entre conceitos.\n- Testar compreensão com perguntas práticas e registrar sínteses limpas.",
        "coding" => "- Priorizar leitura cirúrgica de código (AST outline/symbol), testes automatizados e diffs mínimos.\n- Nunca introduzir mudanças sem validar compilação e efeitos colaterais.",
        "creative" => "- Ajudar a desenvolver ideias originais, roteiros e textos claros mantendo a voz autêntica do autor.\n- Organizar rascunhos em etapas revisáveis sem perder referências anteriores.",
        _ => "- Atuar com proatividade, clareza e foco prático no objetivo definido.\n- Transformar pedidos abertos em passos simples, verificáveis e bem organizados.",
    };

    let privacy_block = if strict_privacy {
        "## 2. Privacidade Absoluta & Soberania de Dados\n\
- **Privacidade Absoluta (FACTO)**: Dados pessoais, documentos sensíveis, registros médicos ou financeiros **NUNCA** devem ser enviados para APIs externas do Free Tier.\n\
- Priorize modelos privados autenticados ou processamento 100% local via GPU/SLM do **Souls Engine**."
    } else {
        "## 2. Governança de Contexto & Eficiência\n\
- Utilize as ferramentas locais do **Souls Engine** (`127.0.0.1:9123/mcp`) para buscar memórias relevantes e economizar contexto.\n\
- Antes de delegar subtarefas longas (`delegate_task`), consulte `route_subtask` para escolher o modelo ideal."
    };

    format!(
        "# SOUL.md — {title}\n\n\
Você é **{title}**, um assistente dedicado e parceiro estratégico. Sua missão central: {goal}\n\n\
## 1. Postura & Diretrizes de Atuação\n\
{domain_bullets}\n\n\
{privacy_block}\n\n\
## 3. Estilo de Comunicação\n\
- Responda sempre em **Português do Brasil (pt-BR)**, de forma direta, acolhedora e objetiva, destacando **conceitos-chave em negrito**.\n\
- Quando houver ambiguidade importante, faça perguntas curtas de alinhamento antes de agir.\n"
    )
}

/// Creates or clones a full Hermes Persona in `profiles/<slug>/`, wiring `profile.yaml`, `config.yaml`,
/// `SOUL.md`, internal credentials link (without exposing secrets), and optional initial Skills!
pub async fn create_or_clone_persona(
    state: &AppState,
    roots: &PersonasHubRoots,
    skills_roots: &SkillsHubRoots,
    req: PersonaCreateRequest,
) -> Result<Value, String> {
    let clean_slug = normalize_persona_slug(&req.slug);
    if clean_slug.is_empty() || clean_slug == "default" {
        return Err("Escolha um identificador válido e único para a nova Persona (diferente de 'default').".to_string());
    }

    let target_dir = roots.hermes_root_dir.join("profiles").join(&clean_slug);
    if target_dir.exists() {
        return Err(format!(
            "Já existe uma Persona chamada '{clean_slug}' nos seus perfis."
        ));
    }

    let donor_slug = req
        .clone_from
        .as_deref()
        .map(normalize_persona_slug)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());
    let donor_dir = roots.persona_dir(&donor_slug)?;

    fs::create_dir_all(target_dir.join("skills"))
        .map_err(|e| format!("Falha ao criar pasta da Persona: {e}"))?;
    let _ = fs::create_dir_all(target_dir.join("memories"));
    let _ = fs::create_dir_all(target_dir.join("assets"));
    let _ = fs::create_dir_all(target_dir.join("workspace"));

    // Internal server-side copy of `.env` from donor if present so Hermes Desktop can authenticate
    // immediately without the user touching hidden files — NEVER reading or returning `.env` content!
    if donor_dir.is_dir() {
        let donor_env = donor_dir.join(".env");
        if donor_env.is_file() {
            let _ = fs::copy(&donor_env, target_dir.join(".env"));
        }
    }

    // Build `config.yaml` from donor (or clean minimal config) + enforce Souls Engine MCP + model choice
    let base_cfg = if donor_dir.join("config.yaml").is_file() {
        fs::read_to_string(donor_dir.join("config.yaml")).unwrap_or_default()
    } else {
        "model:\n  default: \"deepseek/deepseek-chat\"\n  provider: \"nous\"\nagent:\n  max_turns: 500\n  reasoning_effort: high\nmemory:\n  memory_enabled: true\n".to_string()
    };

    let strict_privacy = req.privacy_mode.as_deref() == Some("strict_private")
        || matches!(req.template_kind.as_deref(), Some("finance") | Some("health"));

    let overview = scan_personas_ecosystem(roots);
    let chosen_model = req
        .model_default
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(overview.recommended_private_model);
    let chosen_provider = req
        .model_provider
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(overview.recommended_private_provider);

    let mut cfg = yaml_set_nested_scalar(&base_cfg, "model", "default", &format_yaml_quoted(&chosen_model));
    cfg = yaml_set_nested_scalar(&cfg, "model", "provider", &format_yaml_quoted(&chosen_provider));
    let connect_mcp = req.connect_souls_mcp.unwrap_or(true);
    cfg = yaml_set_souls_mcp_connection(&cfg, connect_mcp);
    if strict_privacy {
        cfg = yaml_remove_free_fallbacks(&cfg);
    }
    fs::write(target_dir.join("config.yaml"), cfg.as_bytes())
        .map_err(|e| format!("Falha ao gravar config.yaml da nova Persona: {e}"))?;

    // Build `profile.yaml`
    let created_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let color = req
        .accent_color
        .as_deref()
        .filter(|c| c.starts_with('#'))
        .unwrap_or("#38bdf8");
    let desc_final = if req.description.trim().is_empty() {
        req.goal_prompt.trim().chars().take(180).collect()
    } else {
        req.description.trim().to_string()
    };
    let profile_yaml = format!(
        "description: {}\ndescription_auto: false\nui_meta:\n  hermes-bots:\n    shape: pill\n    color: '{}'\n    imageKind: shape\n    title: {}\n    created: {}\n_ui_meta_revisions:\n  hermes-bots: 1\n",
        format_yaml_quoted(&desc_final),
        color,
        format_yaml_quoted(req.display_title.trim()),
        created_ms
    );
    fs::write(target_dir.join("profile.yaml"), profile_yaml.as_bytes())
        .map_err(|e| format!("Falha ao gravar profile.yaml: {e}"))?;

    // Build `SOUL.md` (Deterministic scaffold or optional Local Model synthesis)
    let template_kind = req.template_kind.as_deref().unwrap_or("custom");
    let mut soul_md = build_deterministic_soul_markdown(
        &req.display_title,
        &desc_final,
        template_kind,
        &req.goal_prompt,
        strict_privacy,
    );
    let mut engine_used = "deterministic_scaffold";

    if req.engine.as_deref() == Some("local_silicon") {
        let prompt = format!(
            "Escreva um arquivo SOUL.md conciso em Português do Brasil (Markdown puro, iniciando com `# SOUL.md — `) definindo missão, diretrizes práticas, privacidade e estilo de comunicação.\n\nTítulo: {}\nObjetivo: {}\nPrivacidade estrita: {}\n\nMarkdown final:\n",
            req.display_title, req.goal_prompt, strict_privacy
        );
        let params = GenParams {
            max_tokens: 700,
            temperature: 0.25,
            top_p: 0.9,
            stop_sequences: Vec::new(),
            json_schema: None,
        };
        if let Ok(Ok(gen_txt)) = tokio::time::timeout(
            std::time::Duration::from_secs(6),
            state.runtime.infer_tier1_generate(&prompt, &params),
        )
        .await
        {
            let cleaned = sanitize_lf(gen_txt.trim().trim_matches('`').trim());
            if cleaned.contains("# ") && cleaned.len() > 120 {
                soul_md = format!("{cleaned}\n");
                engine_used = "local_silicon";
            }
        }
    }

    fs::write(target_dir.join("SOUL.md"), soul_md.as_bytes())
        .map_err(|e| format!("Falha ao gravar SOUL.md: {e}"))?;
    let _ = record_persona_snapshot(roots, &clean_slug, "SOUL.md", &soul_md, "created");

    // Copy selected initial skills from installed Skills Hub into `profiles/<clean_slug>/skills/`
    let mut installed_skills = Vec::new();
    if let Some(ref skill_slugs) = req.initial_skills {
        let persona_target = vec![format!("persona:{clean_slug}")];
        for sk in skill_slugs {
            if let Ok(det) = skills_hub::inspect_skill_detail(skills_roots, sk, None) {
                if let Some(content) = det.get("content").and_then(|v| v.as_str()) {
                    let cat = det
                        .get("record")
                        .and_then(|r| r.get("category"))
                        .and_then(|v| v.as_str());
                    if skills_hub::sync_skill_to_targets(
                        skills_roots,
                        sk,
                        content,
                        &persona_target,
                        cat,
                        "persona_studio_init",
                    )
                    .is_ok()
                    {
                        installed_skills.push(sk.clone());
                    }
                }
            }
        }
    }

    let summary = scan_single_persona_dir(roots, &clean_slug, &target_dir, false);
    Ok(json!({
        "status": "success",
        "slug": clean_slug,
        "engine_used": engine_used,
        "installed_skills": installed_skills,
        "persona": summary
    }))
}

/// Refines `SOUL.md` hygiene & formatting (cleaning truncated duplicate lines, normalizing spacing/LF,
/// or applying optional local-model polish) while returning a preview diff before saving.
pub async fn refine_persona_soul_markdown(
    state: &AppState,
    req: PersonaSoulRefineRequest,
) -> Value {
    let (mut refined, removed_dups) = heal_soul_truncated_duplicates(&req.current_soul);
    let mut notes = Vec::new();
    if removed_dups > 0 {
        notes.push(format!(
            "Removida(s) {removed_dups} linha(s) incompleta(s)/repetida(s)."
        ));
    }

    // Ensure clean H1 header if missing
    if !refined.trim_start().starts_with("# ") {
        refined = format!("# SOUL.md — {}\n\n{}", req.slug, refined.trim_start());
        notes.push("Adicionado título principal padronizado (`# SOUL.md`).".to_string());
    }

    if let Some(ref instr) = req.instruction {
        let trimmed_i = instr.trim();
        if !trimmed_i.is_empty() {
            if req.use_local_model.unwrap_or(false) {
                let prompt = format!(
                    "Aprimore o SOUL.md abaixo aplicando o pedido do usuário, preservando 100% dos caminhos de arquivos em crases e regras de privacidade. Responda apenas com o Markdown final.\n\nPedido: {trimmed_i}\n\nSOUL.md atual:\n{refined}\n\nMarkdown atualizado:\n"
                );
                let params = GenParams {
                    max_tokens: 900,
                    temperature: 0.2,
                    top_p: 0.9,
                    stop_sequences: Vec::new(),
                    json_schema: None,
                };
                if let Ok(Ok(gen_txt)) = tokio::time::timeout(
                    std::time::Duration::from_secs(6),
                    state.runtime.infer_tier1_generate(&prompt, &params),
                )
                .await
                {
                    let cand = sanitize_lf(gen_txt.trim());
                    if cand.starts_with("# ") && cand.len() >= refined.len() / 2 {
                        refined = format!("{cand}\n");
                        notes.push("Texto refinado com apoio do modelo local.".to_string());
                    }
                }
            }
            if notes.is_empty() || !refined.contains(trimmed_i) {
                // Safe deterministic append of new guideline if LLM wasn't used
                if !req.use_local_model.unwrap_or(false) {
                    refined = format!(
                        "{}\n## Diretriz Adicional\n- {}\n",
                        refined.trim_end(),
                        trimmed_i
                    );
                    notes.push("Nova diretriz adicionada de forma estruturada ao final do SOUL.md.".to_string());
                }
            }
        }
    }

    if notes.is_empty() {
        notes.push("Formatação e quebras de linha LF verificadas — texto já está limpo!".to_string());
    }

    let diff = compute_persona_text_diff("SOUL.md", &req.current_soul, &refined);
    json!({
        "status": "success",
        "content": refined,
        "notes": notes,
        "diff": diff
    })
}

/// Archives (pauses) a secondary Persona into `.souls_data/personas_hub/archived/<slug>`
/// or restores an archived Persona back into `%LOCALAPPDATA%\hermes\profiles\<slug>`.
pub fn archive_or_restore_persona(
    roots: &PersonasHubRoots,
    slug: &str,
    restore: bool,
) -> Result<Value, String> {
    let clean_slug = normalize_persona_slug(slug);
    if clean_slug.is_empty() || clean_slug == "default" {
        return Err("O perfil Principal ('default') é a base do Hermes e não pode ser arquivado.".to_string());
    }

    let active_dir = roots.hermes_root_dir.join("profiles").join(&clean_slug);
    let archive_dir = roots.archived_dir().join(&clean_slug);

    if restore {
        if !archive_dir.is_dir() {
            return Err(format!("A Persona arquivada '{clean_slug}' não foi encontrada."));
        }
        if active_dir.exists() {
            return Err(format!("Já existe uma Persona ativa chamada '{clean_slug}'."));
        }
        if let Some(parent) = active_dir.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Erro ao preparar pasta profiles: {e}"))?;
        }
        move_dir_across_volumes(&archive_dir, &active_dir)?;
        Ok(json!({
            "status": "success",
            "action": "restored",
            "slug": clean_slug
        }))
    } else {
        if !active_dir.is_dir() {
            return Err(format!("A Persona '{clean_slug}' não está ativa."));
        }
        fs::create_dir_all(roots.archived_dir())
            .map_err(|e| format!("Erro ao preparar cofre de arquivamento: {e}"))?;
        if archive_dir.exists() {
            let backup_old = roots
                .archived_dir()
                .join(format!("{clean_slug}__{}", now_epoch_secs()));
            let _ = move_dir_across_volumes(&archive_dir, &backup_old);
        }
        if let Ok(soul) = fs::read_to_string(active_dir.join("SOUL.md")) {
            let _ = record_persona_snapshot(roots, &clean_slug, "SOUL.md", &soul, "pre_archive");
        }
        move_dir_across_volumes(&active_dir, &archive_dir)?;
        Ok(json!({
            "status": "success",
            "action": "archived",
            "slug": clean_slug
        }))
    }
}

fn move_dir_across_volumes(src: &Path, dst: &Path) -> Result<(), String> {
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    // Cross-volume copy (`C:\` <-> `Z:\`) + remove source once verified
    copy_dir_recursive(src, dst)?;
    fs::remove_dir_all(src).map_err(|e| format!("Falha ao limpar pasta origem após cópia: {e}"))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Falha ao criar {}: {e}", dst.display()))?;
    let rd = fs::read_dir(src).map_err(|e| format!("Falha ao ler {}: {e}", src.display()))?;
    for entry in rd.flatten() {
        let p = entry.path();
        let target = dst.join(entry.file_name());
        if p.is_dir() {
            copy_dir_recursive(&p, &target)?;
        } else if p.is_file() {
            fs::copy(&p, &target)
                .map_err(|e| format!("Falha ao copiar {}: {e}", p.display()))?;
        }
    }
    Ok(())
}

pub fn list_archived_personas(roots: &PersonasHubRoots) -> Vec<ArchivedPersonaItem> {
    let Ok(rd) = fs::read_dir(roots.archived_dir()) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let slug = entry.file_name().to_string_lossy().to_string();
        if slug.contains("__") {
            continue;
        }
        let prof = fs::read_to_string(p.join("profile.yaml")).unwrap_or_default();
        let title = yaml_get_nested(&prof, &["ui_meta", "hermes-bots", "title"])
            .unwrap_or_else(|| slug.clone());
        let desc = yaml_get_nested(&prof, &["description"]).unwrap_or_default();
        let epoch = p
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        items.push(ArchivedPersonaItem {
            slug,
            display_title: title,
            description: desc,
            archived_at_epoch: epoch,
        });
    }
    items.sort_by(|a, b| a.slug.cmp(&b.slug));
    items
}

/// Restores a specific historical snapshot (`SOUL.md`, `config.yaml`, or `profile.yaml`).
pub fn rollback_persona_snapshot(
    roots: &PersonasHubRoots,
    slug: &str,
    snapshot_id: &str,
) -> Result<Value, String> {
    let clean_slug = normalize_persona_slug(slug);
    if snapshot_id.contains("..") || snapshot_id.contains('/') || snapshot_id.contains('\\') {
        return Err("Identificador de snapshot inválido.".to_string());
    }

    let snap_path = roots.history_dir().join(&clean_slug).join(snapshot_id);
    if !snap_path.is_file() {
        return Err("Versão de histórico não encontrada.".to_string());
    }

    let stem = snapshot_id.trim_end_matches(".bak");
    let parts: Vec<&str> = stem.splitn(3, "__").collect();
    if parts.len() < 3 {
        return Err("Formato de snapshot inválido.".to_string());
    }
    let target_file = match parts[1] {
        "SOUL.md" => "SOUL.md",
        "config.yaml" => "config.yaml",
        "profile.yaml" => "profile.yaml",
        _ => return Err("Arquivo alvo de restauração não permitido.".to_string()),
    };

    let persona_dir = roots.persona_dir(&clean_slug)?;
    if !persona_dir.is_dir() {
        return Err(format!("A Persona '{clean_slug}' não está ativa."));
    }

    let snap_content = sanitize_lf(
        &fs::read_to_string(&snap_path)
            .map_err(|e| format!("Falha ao ler snapshot: {e}"))?,
    );
    let dest_path = persona_dir.join(target_file);
    if let Ok(cur) = fs::read_to_string(&dest_path) {
        let _ = record_persona_snapshot(roots, &clean_slug, target_file, &cur, "pre_rollback");
    }
    fs::write(&dest_path, snap_content.as_bytes())
        .map_err(|e| format!("Falha ao restaurar {target_file}: {e}"))?;

    Ok(json!({
        "status": "success",
        "slug": clean_slug,
        "restored_file": target_file,
        "snapshot_id": snapshot_id
    }))
}

/// Compares two Personas side-by-side (`SOUL.md` diff + structured configuration comparison).
pub fn compare_two_personas(
    roots: &PersonasHubRoots,
    left_slug: &str,
    right_slug: &str,
) -> Result<Value, String> {
    let l_clean = normalize_persona_slug(left_slug);
    let r_clean = normalize_persona_slug(right_slug);
    let l_dir = roots.persona_dir(&l_clean)?;
    let r_dir = roots.persona_dir(&r_clean)?;
    if !l_dir.is_dir() || !r_dir.is_dir() {
        return Err("Selecione duas Personas ativas válidas para comparar.".to_string());
    }

    let l_sum = scan_single_persona_dir(roots, &l_clean, &l_dir, l_clean == "default");
    let r_sum = scan_single_persona_dir(roots, &r_clean, &r_dir, r_clean == "default");

    let l_soul = fs::read_to_string(l_dir.join("SOUL.md")).unwrap_or_default();
    let r_soul = fs::read_to_string(r_dir.join("SOUL.md")).unwrap_or_default();
    let soul_diff = compute_persona_text_diff("SOUL.md", &l_soul, &r_soul);

    let config_rows = vec![
        json!({ "label": "Modelo Principal", "left": l_sum.model_default, "right": r_sum.model_default }),
        json!({ "label": "Provedor", "left": l_sum.model_provider, "right": r_sum.model_provider }),
        json!({ "label": "Profundidade de Raciocínio", "left": l_sum.reasoning_effort, "right": r_sum.reasoning_effort }),
        json!({ "label": "Limite de Passos", "left": l_sum.max_turns, "right": r_sum.max_turns }),
        json!({ "label": "Aceleração Souls Engine", "left": if l_sum.souls_mcp_connected { "Conectada" } else { "Desligada" }, "right": if r_sum.souls_mcp_connected { "Conectada" } else { "Desligada" } }),
        json!({ "label": "Postura de Privacidade", "left": if l_sum.requires_strict_privacy { "Dados Sensíveis" } else { "Uso Geral" }, "right": if r_sum.requires_strict_privacy { "Dados Sensíveis" } else { "Uso Geral" } }),
        json!({ "label": "Habilidades Ativas", "left": l_sum.active_skills_count, "right": r_sum.active_skills_count }),
        json!({ "label": "Índice de Coerência", "left": format!("{}/100", l_sum.alignment_score), "right": format!("{}/100", r_sum.alignment_score) }),
    ];

    Ok(json!({
        "status": "success",
        "left": l_sum,
        "right": r_sum,
        "config_comparison": config_rows,
        "soul_diff": soul_diff
    }))
}

// ============================================================================
// HERMETIC UNIT TESTS (TDD RED-GREEN-REFACTOR)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static PERSONA_TEST_SEQ: AtomicU64 = AtomicU64::new(1);

    struct TempPersonasEnv {
        base_dir: PathBuf,
        roots: PersonasHubRoots,
    }

    impl TempPersonasEnv {
        fn new(label: &str) -> Self {
            let seq = PERSONA_TEST_SEQ.fetch_add(1, Ordering::SeqCst);
            let pid = std::process::id();
            let base_dir = PathBuf::from(r"Z:\souls_engine\.souls_data")
                .join(format!("test_tmp_personas_hub_{label}_{pid}_{seq}"));
            let _ = fs::remove_dir_all(&base_dir);
            fs::create_dir_all(&base_dir).expect("create temp persona test dir on Z:");

            let roots = PersonasHubRoots {
                hub_data_dir: base_dir.join("hub_data"),
                hermes_root_dir: base_dir.join("hermes_root"),
            };
            fs::create_dir_all(&roots.hub_data_dir).unwrap();
            fs::create_dir_all(roots.hermes_root_dir.join("profiles").join("financas")).unwrap();

            // Seed default profile
            fs::write(
                roots.hermes_root_dir.join("config.yaml"),
                "mcp_servers:\n  souls_engine:\n    url: \"http://127.0.0.1:9123/mcp\"\nmodel:\n  default: \"deepseek/deepseek-chat\"\n  provider: \"nous\"\nagent:\n  max_turns: 500\n  reasoning_effort: high\nmemory:\n  memory_enabled: true\n",
            )
            .unwrap();
            fs::write(
                roots.hermes_root_dir.join("profile.yaml"),
                "display_name: Bruno\nui_meta:\n  hermes-bots:\n    color: '#8b5cf6'\n    title: Hermes_Bruno\n",
            )
            .unwrap();
            fs::write(
                roots.hermes_root_dir.join("SOUL.md"),
                "# SOUL.md — Identidade Global\nVocê é o orquestrador principal.\n- Antes de invocar a ferramenta delegate_task consulte compulsoriamente a ferramenta MCP\n- Antes de invocar a ferramenta `delegate_task` consulte compulsoriamente a ferramenta MCP `route_subtask` com estimativa.\n",
            )
            .unwrap();

            // Seed `financas` with a real privacy conflict + missing MCP + multiline description
            let fin_dir = roots.hermes_root_dir.join("profiles").join("financas");
            fs::write(
                fin_dir.join("config.yaml"),
                "model:\n  default: stepfun/step-5-preview:free\n  provider: nous\nagent:\n  max_turns: 500\n  reasoning_effort: medium\nfallback_providers:\n  - provider: opencode-free\n    model: foo\n  - provider: nous\n",
            )
            .unwrap();
            fs::write(
                fin_dir.join("profile.yaml"),
                "description: financas\ndescription_auto: false\nui_meta:\n  hermes-bots:\n    title: financas\n",
            )
            .unwrap();
            fs::write(
                fin_dir.join("SOUL.md"),
                "# SOUL.md — Guardião Financeiro\nCuida do patrimônio e fluxo de caixa.\n## Privacidade\n- **Privacidade Absoluta**: Extratos bancários NUNCA envie para APIs externas do Free Tier.\n",
            )
            .unwrap();

            Self { base_dir, roots }
        }
    }

    impl Drop for TempPersonasEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base_dir);
        }
    }

    #[test]
    fn test_yaml_block_surgeon_preserves_structure_and_comments() {
        let sample = "# Top comment\nmodel:\n  default: stepfun/step-5-preview:free\n  provider: nous\n  # keep comment\n  api_mode: chat_completions\nagent:\n  max_turns: 500\n  reasoning_effort: medium\nfallback_providers:\n  - provider: opencode-free\n    tier: free\n  - provider: nous\n";

        let step1 = yaml_set_nested_scalar(sample, "model", "default", "\"deepseek/deepseek-chat\"");
        let step2 = yaml_set_nested_scalar(&step1, "agent", "reasoning_effort", "high");
        let step3 = yaml_set_souls_mcp_connection(&step2, true);
        let step4 = yaml_remove_free_fallbacks(&step3);

        assert!(!step4.contains("\r\n"));
        assert!(step4.contains("# Top comment"));
        assert!(step4.contains("# keep comment"));
        assert!(step4.contains("api_mode: chat_completions"));
        assert_eq!(
            yaml_get_nested(&step4, &["model", "default"]).as_deref(),
            Some("deepseek/deepseek-chat")
        );
        assert_eq!(
            yaml_get_nested(&step4, &["agent", "reasoning_effort"]).as_deref(),
            Some("high")
        );
        assert!(yaml_has_block_path(&step4, &["mcp_servers", "souls_engine"]));
        assert!(!step4.contains("opencode-free"));
        assert!(step4.contains("- provider: nous"));

        // Toggle MCP off cleanly without harming model/agent blocks
        let step5 = yaml_set_souls_mcp_connection(&step4, false);
        assert!(!yaml_has_block_path(&step5, &["mcp_servers", "souls_engine"]));
        assert_eq!(
            yaml_get_nested(&step5, &["model", "default"]).as_deref(),
            Some("deepseek/deepseek-chat")
        );
    }

    #[test]
    fn test_privacy_conflict_detection_and_one_click_alignment() {
        let env = TempPersonasEnv::new("align");
        let ov = scan_personas_ecosystem(&env.roots);
        assert_eq!(ov.total_active_personas, 2);
        assert_eq!(ov.privacy_alerts_count, 1);

        let fin = ov.personas.iter().find(|p| p.slug == "financas").unwrap();
        assert_eq!(fin.status_badge, "privacy_alert");
        assert!(!fin.souls_mcp_connected);
        assert!(fin.findings.iter().any(|f| f.code == "PRIVACY_FREE_MODEL_CONFLICT"));
        assert!(fin.findings.iter().any(|f| f.code == "SOULS_MCP_DISCONNECTED"));
        assert!(fin.findings.iter().any(|f| f.code == "PROFILE_DESC_GENERIC"));

        // 1. Dry-run preview must NOT mutate disk
        let preview = preview_or_apply_persona_alignment(
            &env.roots,
            PersonaAlignRequest {
                slug: "financas".to_string(),
                dry_run: true,
                preferred_model: None,
                preferred_provider: None,
            },
        )
        .unwrap();
        assert_eq!(preview.get("dry_run").and_then(|v| v.as_bool()), Some(true));
        assert!(!preview
            .get("changed_files")
            .and_then(|v| v.as_array())
            .unwrap()
            .is_empty());
        let still_alert = scan_personas_ecosystem(&env.roots);
        assert_eq!(still_alert.privacy_alerts_count, 1);

        // 2. Apply alignment -> resolves privacy conflict, connects MCP, enriches description, saves backup!
        let applied = preview_or_apply_persona_alignment(
            &env.roots,
            PersonaAlignRequest {
                slug: "financas".to_string(),
                dry_run: false,
                preferred_model: None,
                preferred_provider: None,
            },
        )
        .unwrap();
        assert_eq!(applied.get("status").and_then(|v| v.as_str()), Some("success"));

        let ov_after = scan_personas_ecosystem(&env.roots);
        assert_eq!(ov_after.privacy_alerts_count, 0);
        let fin_after = ov_after.personas.iter().find(|p| p.slug == "financas").unwrap();
        assert_eq!(fin_after.alignment_score, 100);
        assert!(fin_after.souls_mcp_connected);
        assert_eq!(fin_after.model_default, "deepseek/deepseek-chat");
        assert!(!list_persona_history(&env.roots, "financas").is_empty());

        // Also verify truncated duplicate line healer on `default`
        let def_applied = preview_or_apply_persona_alignment(
            &env.roots,
            PersonaAlignRequest {
                slug: "default".to_string(),
                dry_run: false,
                preferred_model: None,
                preferred_provider: None,
            },
        )
        .unwrap();
        assert_eq!(def_applied.get("status").and_then(|v| v.as_str()), Some("success"));
        let healed_def_soul =
            fs::read_to_string(env.roots.hermes_root_dir.join("SOUL.md")).unwrap();
        assert!(healed_def_soul.contains("`route_subtask` com estimativa."));
        assert_eq!(
            healed_def_soul.matches("Antes de invocar a ferramenta").count(),
            1
        );
    }

    #[test]
    fn test_memory_sovereignty_and_archive_restore_rollback() {
        let env = TempPersonasEnv::new("sovereignty");
        let fin_dir = env.roots.hermes_root_dir.join("profiles").join("financas");
        fs::create_dir_all(fin_dir.join("memories")).unwrap();
        let mem_file = fin_dir.join("memories").join("MEMORY.md");
        let sacred_memory = "# Memória Sagrada do Hermes\nSaldo conciliado: R$ 100,00.\n";
        fs::write(&mem_file, sacred_memory).unwrap();

        // Inspect must surface memory as read_only_constitutional: true
        let detail = inspect_persona_detail(&env.roots, "financas").unwrap();
        let mems = detail
            .get("memories_read_only")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(mems.len(), 1);
        assert_eq!(
            mems[0]
                .get("read_only_constitutional")
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        // Update persona & verify MEMORY.md is 100% untouched byte-for-byte
        update_persona(
            &env.roots,
            PersonaUpdateRequest {
                slug: "financas".to_string(),
                display_title: Some("Guardião Finanças".to_string()),
                description: Some("Gestão financeira pessoal privada.".to_string()),
                accent_color: Some("#10b981".to_string()),
                model_default: Some("deepseek/deepseek-chat".to_string()),
                model_provider: Some("nous".to_string()),
                reasoning_effort: Some("high".to_string()),
                max_turns: Some(400),
                memory_enabled: Some(true),
                souls_mcp_connected: Some(true),
                sanitize_free_fallbacks: Some(true),
                soul_content: Some("# SOUL.md — Guardião Finanças v2\nMissão atualizada.\n".to_string()),
                reason: Some("unit_test".to_string()),
            },
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&mem_file).unwrap(), sacred_memory);

        // Rollback SOUL.md snapshot
        let hist = list_persona_history(&env.roots, "financas");
        let soul_snap = hist.iter().find(|h| h.target_file == "SOUL.md").unwrap();
        rollback_persona_snapshot(&env.roots, "financas", &soul_snap.snapshot_id).unwrap();
        let rolled_soul = fs::read_to_string(fin_dir.join("SOUL.md")).unwrap();
        assert!(rolled_soul.contains("Guardião Financeiro"));

        // Default cannot be archived
        assert!(archive_or_restore_persona(&env.roots, "default", false).is_err());

        // Archive `financas` and restore it with MEMORY.md still intact!
        archive_or_restore_persona(&env.roots, "financas", false).unwrap();
        assert_eq!(scan_personas_ecosystem(&env.roots).total_active_personas, 1);
        assert_eq!(scan_personas_ecosystem(&env.roots).archived_personas_count, 1);

        archive_or_restore_persona(&env.roots, "financas", true).unwrap();
        assert_eq!(scan_personas_ecosystem(&env.roots).total_active_personas, 2);
        assert_eq!(fs::read_to_string(&mem_file).unwrap(), sacred_memory);
    }
}
