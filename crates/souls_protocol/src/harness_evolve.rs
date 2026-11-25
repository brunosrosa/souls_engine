//! Evolutive Harness, Cordis Reversible Microkernel, Cognitive Catnap,
//! Tombstone JIT, and WeEvolve ELO/EMA Data Transfer Objects (`v0.9.4` / `ADR-034`).
//!
//! Pure contracts shared across `souls_core`, `souls_memory`, and `souls_server`
//! with zero I/O side effects and `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Eight canonical editable anatomy slots (`c1`–`c8`) inspired by Google Harness².
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessComponentSlot {
    /// Constitutional read-only layer (`SOUL.md` / `AGENTS.md`), pinned by SHA-256 (fail-closed).
    C1SystemPrompt,
    /// Bipartite guardrails: immutable safety taboos vs auto-optimizable operational anti-loop rules.
    C2Guardrails,
    /// Evidence-based working/long-term memory hygiene & JIT Tombstones.
    C3Memory,
    /// Modular on-demand playbooks (`SKILL.md` — primary Ralph Loop target via Quarantine).
    C4Skills,
    /// Consolidated ACI tool selection cues and switchboard hints.
    C5ToolGuidance,
    /// Deterministic validators/parsers sandboxed with `SCRUB_ENV`.
    C6Scripts,
    /// Sub-agent routing cues and delegation boundary profiles.
    C7Subagents,
    /// Lifecycle middleware mounted via reversible Cordis LIFO fibers.
    C8PluginsHooks,
}

impl HarnessComponentSlot {
    /// Returns the short canonical slot code (`"c1"`..`"c8"`).
    #[must_use]
    pub const fn as_code(&self) -> &'static str {
        match self {
            Self::C1SystemPrompt => "c1",
            Self::C2Guardrails => "c2",
            Self::C3Memory => "c3",
            Self::C4Skills => "c4",
            Self::C5ToolGuidance => "c5",
            Self::C6Scripts => "c6",
            Self::C7Subagents => "c7",
            Self::C8PluginsHooks => "c8",
        }
    }

    /// Returns a clear, jargon-free Portuguese label for UI display.
    #[must_use]
    pub const fn label_pt(&self) -> &'static str {
        match self {
            Self::C1SystemPrompt => "Identidade & Regras Fundamentais",
            Self::C2Guardrails => "Proteções Operacionais Anti-Erro",
            Self::C3Memory => "Higiene de Memória Sem Contradições",
            Self::C4Skills => "Habilidades Modulares (Skills)",
            Self::C5ToolGuidance => "Orientações de Uso de Ferramentas",
            Self::C6Scripts => "Scripts Determinísticos Isolados",
            Self::C7Subagents => "Perfis & Delegação de Subagentes",
            Self::C8PluginsHooks => "Interceptadores & Ajustes Reversíveis",
        }
    }

    /// Returns `true` when this slot is constitutionally pinned (fail-closed against automated mutation).
    #[must_use]
    pub const fn is_constitutionally_pinned(&self) -> bool {
        matches!(self, Self::C1SystemPrompt)
    }

    /// Parses a slot from `"c1"`..`"c8"` or snake_case names.
    pub fn from_str_lossy(raw: &str) -> Option<Self> {
        let norm = raw.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        match norm.as_str() {
            "c1" | "c1_system_prompt" | "system_prompt" => Some(Self::C1SystemPrompt),
            "c2" | "c2_guardrails" | "guardrails" => Some(Self::C2Guardrails),
            "c3" | "c3_memory" | "memory" => Some(Self::C3Memory),
            "c4" | "c4_skills" | "skills" => Some(Self::C4Skills),
            "c5" | "c5_tool_guidance" | "tool_guidance" => Some(Self::C5ToolGuidance),
            "c6" | "c6_scripts" | "scripts" => Some(Self::C6Scripts),
            "c7" | "c7_subagents" | "subagents" => Some(Self::C7Subagents),
            "c8" | "c8_plugins_hooks" | "plugins_hooks" | "hooks" => Some(Self::C8PluginsHooks),
            _ => None,
        }
    }

    /// Returns all 8 canonical slots in order `c1`..`c8`.
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::C1SystemPrompt,
            Self::C2Guardrails,
            Self::C3Memory,
            Self::C4Skills,
            Self::C5ToolGuidance,
            Self::C6Scripts,
            Self::C7Subagents,
            Self::C8PluginsHooks,
        ]
    }
}

fn default_pin_mode() -> String {
    "fail_closed".to_string()
}

fn default_catnap_trigger_mode() -> String {
    "auto".to_string()
}

fn default_catnap_actor() -> String {
    "catnap_scheduler".to_string()
}

/// Auditable cross-channel authorization provenance attached to sensitive mutations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationProvenanceDto {
    pub channel: String,
    pub timestamp: i64,
    pub scope: String,
    #[serde(default)]
    pub actor: Option<String>,
}

impl AuthorizationProvenanceDto {
    /// Returns `true` when channel, positive timestamp, and scope are explicitly provided.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.channel.trim().is_empty() && self.timestamp > 0 && !self.scope.trim().is_empty()
    }
}

/// Cryptographic SHA-256 pin verification record for a constitutional (`c1`) file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalPinStatusDto {
    pub file_path_posix: String,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub verified: bool,
    pub fail_closed_triggered: bool,
    #[serde(default = "default_pin_mode")]
    pub pin_mode: String,
    #[serde(default)]
    pub drift_detected: bool,
    #[serde(default)]
    pub resolved_path: Option<String>,
    pub checked_at: i64,
}

/// Live reversible tactical fiber mounted in the Cordis microkernel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CordisFiberDto {
    pub fiber_id: String,
    pub slot: HarnessComponentSlot,
    pub title_pt: String,
    pub benefit_summary_pt: String,
    pub ephemeral_cue_text: String,
    pub target_task_type: Option<String>,
    pub coeffect_dependencies: Vec<String>,
    pub active: bool,
    pub paused_by_coeffect: bool,
    pub undo_operations_count: usize,
    pub elo_rating: f64,
    pub ema_score: f64,
    pub drift_streak: u32,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

/// Report returned when reverting a Cordis fiber in strict LIFO order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CordisRollbackReportDto {
    pub fiber_id: String,
    pub reverted_operations: Vec<String>,
    pub elapsed_us: u64,
    pub restored_cleanly: bool,
}

/// Configuration for the Cognitive Catnap Protocol (idle & thermal FinOps gates).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatnapConfigDto {
    pub enabled: bool,
    pub min_idle_secs: u64,
    pub max_gpu_temp_c: f32,
    pub max_cpu_load_pct: f32,
    pub max_daily_budget_usd: f64,
    pub allow_free_cloud_improver: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start_hour: u8,
    pub quiet_hours_end_hour: u8,
}

impl Default for CatnapConfigDto {
    fn default() -> Self {
        Self {
            enabled: true,
            min_idle_secs: 180,
            max_gpu_temp_c: 62.0,
            max_cpu_load_pct: 15.0,
            max_daily_budget_usd: 0.00,
            allow_free_cloud_improver: true,
            quiet_hours_enabled: true,
            quiet_hours_start_hour: 22,
            quiet_hours_end_hour: 8,
        }
    }
}

/// Runtime snapshot of the Cognitive Catnap scheduler & preemption guard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatnapStateDto {
    pub state: String,
    pub idle_elapsed_secs: u64,
    pub last_preemption_latency_ms: f64,
    pub slices_completed_today: u64,
    pub spent_today_usd: f64,
    pub last_cursor_checkpoint: String,
    pub pending_failed_trajectories: usize,
    pub config: CatnapConfigDto,
}

/// Audit record of a single atomic micro-slice executed by the Cognitive Catnap Protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatnapSliceRecordDto {
    pub slice_id: String,
    pub slice_kind: String,
    pub runmark_fingerprint: String,
    pub cursor_checkpoint: String,
    pub status: String,
    pub duration_ms: f64,
    pub cost_usd: f64,
    pub summary_pt: String,
    #[serde(default = "default_catnap_trigger_mode")]
    pub trigger_mode: String,
    #[serde(default = "default_catnap_actor")]
    pub actor: String,
    #[serde(default)]
    pub authorization_provenance: Option<AuthorizationProvenanceDto>,
    pub created_at: i64,
}

/// Auditable logical tombstone superseding a contradicting memory without violating STABLE immutability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryTombstoneDto {
    pub tombstone_id: String,
    pub superseded_memory_id: String,
    pub superseded_by_memory_id: String,
    pub category: String,
    pub superseded_content_preview: String,
    pub new_content_preview: String,
    pub reason_pt: String,
    pub status: String,
    pub created_at: i64,
    pub restored_at: Option<i64>,
}

/// Bradley-Terry ELO & EMA closed-loop performance record for a harness artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeEvolveRatingDto {
    pub target_id: String,
    pub target_kind: String,
    pub elo_rating: f64,
    pub ema_score: f64,
    pub total_matches: i64,
    pub last_signal: String,
    pub updated_at: i64,
}

/// Crystallized skill candidate produced by The Ralph Loop and staged in Skills Hub Quarantine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RalphSkillCandidateDto {
    pub candidate_id: String,
    pub slug: String,
    pub title_pt: String,
    pub source_session_id: String,
    pub problem_pattern_pt: String,
    pub solution_summary_pt: String,
    pub e3_score: f64,
    pub elo_rating: f64,
    pub quarantine_path: String,
    pub diff_preview: String,
    pub status: String,
    pub created_at: i64,
}

/// Persistent episodic domain harness record (`EXPERIENCE.md` + `INDEX.md` inventory).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainExperienceEntryDto {
    pub episode_id: String,
    pub domain_tag: String,
    pub cue_pattern: String,
    pub always_on_chars: usize,
    pub skills_count: usize,
    pub subagents_count: usize,
    pub scripts_count: usize,
    pub hooks_count: usize,
    pub runmark_fingerprint: String,
    pub experience_summary_pt: String,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_provenance_validation() {
        let valid = AuthorizationProvenanceDto {
            channel: "antigravity_ide".to_string(),
            timestamp: 1795577667,
            scope: "resign_c1_agents".to_string(),
            actor: Some("bruno".to_string()),
        };
        assert!(valid.is_valid());

        let empty_channel = AuthorizationProvenanceDto {
            channel: "   ".to_string(),
            ..valid.clone()
        };
        assert!(!empty_channel.is_valid());

        let bad_ts = AuthorizationProvenanceDto {
            timestamp: 0,
            ..valid.clone()
        };
        assert!(!bad_ts.is_valid());
    }

    #[test]
    fn test_harness_dtos_backwards_compatible_serde_defaults() {
        let legacy_pin = r#"{
            "file_path_posix": "hermes/memories/MEMORY.md",
            "expected_sha256": "abc",
            "actual_sha256": "def",
            "verified": false,
            "fail_closed_triggered": false,
            "checked_at": 1795500000
        }"#;
        let pin: ConstitutionalPinStatusDto = serde_json::from_str(legacy_pin).unwrap();
        assert_eq!(pin.pin_mode, "fail_closed");
        assert!(!pin.drift_detected);
        assert!(pin.resolved_path.is_none());

        let legacy_slice = r#"{
            "slice_id": "s1",
            "slice_kind": "shadow_probe_ppc",
            "runmark_fingerprint": "fp",
            "cursor_checkpoint": "c1",
            "status": "completed",
            "duration_ms": 1.2,
            "cost_usd": 0.0,
            "summary_pt": "ok",
            "created_at": 1795500000
        }"#;
        let slice: CatnapSliceRecordDto = serde_json::from_str(legacy_slice).unwrap();
        assert_eq!(slice.trigger_mode, "auto");
        assert_eq!(slice.actor, "catnap_scheduler");
        assert!(slice.authorization_provenance.is_none());
    }
}
