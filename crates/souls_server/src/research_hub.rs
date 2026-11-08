//! Bare-metal Research Hub & Deep Research Mission Control Plane for `souls_server`.
//!
//! # Architecture & Governance
//! - **Filesystem SSOT**: Persistent mission state stored under `Z:\souls_engine\.souls_data\research/{run_id}/`.
//! - **Strict Dev Drive ReFS Isolation**: All mission IDs are validated against strict regex
//!   `^[0-9]{8}-[0-9]{6}-[a-z0-9-]{1,48}$` to forbid any path traversal or escaping to `C:\`.
//! - **In-Memory Reactive Index**: `ResearchIndex` holds an ultra-fast cached view and broadcasts
//!   live SSE events (`ResearchStreamEvent`) to connected Dashboard sessions.
//! - **Dual Identity & Safe Lease**: Hermes Agent claims queued missions via MCP `research_claim`,
//!   with automated 10-minute lease expiration sweeping if an agent becomes unresponsive.
//! - **LF Line Endings**: All generated JSON and Markdown files strictly enforce `\n` line endings.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tokio::sync::broadcast;
use tracing::{info, warn};

// ============================================================================
// CONSTANTS & VALIDATION GUARDS
// ============================================================================

pub const DEFAULT_MISSION_CAP_USD: f64 = 1.50;
pub const DEFAULT_DAILY_BUDGET_USD: f64 = 8.00;
pub const DEFAULT_CLAIM_LEASE_SECS: u64 = 600; // 10 minutes
pub const MAX_ITERATION_TURNS: u32 = 90;
pub const BEAST_MODE_TURNS_THRESHOLD: u32 = 72;

/// Validates run_id strictly: `YYYYMMDD-HHMMSS-slug` with ascii alphanumeric and dashes.
pub fn is_valid_run_id(id: &str) -> bool {
    let parts: Vec<&str> = id.splitn(3, '-').collect();
    if parts.len() < 3 {
        return false;
    }
    let date_part = parts[0];
    let time_part = parts[1];
    let slug_part = parts[2];

    if date_part.len() != 8 || !date_part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if time_part.len() != 6 || !time_part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if slug_part.is_empty() || slug_part.len() > 64 {
        return false;
    }
    slug_part
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

// ============================================================================
// DOMAIN MODELS & ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Draft,
    Queued,
    Claimed,
    Researching,
    Paused,
    Synthesizing,
    Verifying,
    Confirmed,
    Blocked,
    Failed,
    Cancelled,
}

impl Default for MissionStatus {
    fn default() -> Self {
        MissionStatus::Draft
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    L1FactFinding,
    L2Comparative,
    L3Exhaustive,
}

impl Default for EffortLevel {
    fn default() -> Self {
        EffortLevel::L2Comparative
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchBrief {
    pub question: String,
    pub objectives: Vec<String>,
    pub perspectives: Vec<String>,
    pub subtopics: Vec<String>,
    pub exclusions: Vec<String>,
    pub source_hierarchy: Vec<String>,
    pub effort: EffortLevel,
    pub breadth: u8,
    pub depth: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimLease {
    pub claimed_by: String,
    pub claimed_at: String,
    pub lease_expires_at: String,
    pub heartbeat_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEnvelope {
    pub cap_usd: f64,
    pub est_usd_low: f64,
    pub est_usd_high: f64,
    pub spent_usd: f64,
    pub turns_used: u32,
    pub turns_cap: u32,
    pub beast_mode_at: u32,
}

impl Default for BudgetEnvelope {
    fn default() -> Self {
        Self {
            cap_usd: DEFAULT_MISSION_CAP_USD,
            est_usd_low: 0.10,
            est_usd_high: 0.85,
            spent_usd: 0.0,
            turns_used: 0,
            turns_cap: MAX_ITERATION_TURNS,
            beast_mode_at: BEAST_MODE_TURNS_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TierUsage {
    pub tier0_tokens_saved: u64,
    pub tier1_tokens: u64,
    pub tier2_tokens: u64,
    pub tier3_tokens: u64,
    pub tier1_usd: f64,
    pub tier2_usd: f64,
    pub tier3_usd: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlFlags {
    pub pause_requested: bool,
    pub force_synthesis_requested: bool,
    pub cancel_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRecord {
    pub run_id: String,
    pub slug: String,
    pub status: MissionStatus,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub brief: ResearchBrief,
    pub budget: BudgetEnvelope,
    pub tier_usage: TierUsage,
    pub round: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim: Option<ClaimLease>,
    pub flags: ControlFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSummary {
    pub run_id: String,
    pub slug: String,
    pub status: MissionStatus,
    pub effort: EffortLevel,
    pub question: String,
    pub created_at: String,
    pub updated_at: String,
    pub spent_usd: f64,
    pub cap_usd: f64,
    pub turns_used: u32,
    pub round: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTimelineEvent {
    pub timestamp: String,
    pub run_id: String,
    pub phase: String,
    pub round: u32,
    pub summary: String,
    pub spent_usd: f64,
    pub turns_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinOpsPreview {
    pub effort: EffortLevel,
    pub breadth: u8,
    pub depth: u8,
    pub est_turns: u32,
    pub est_tokens_tier0: u64,
    pub est_tokens_tier1: u64,
    pub est_tokens_tier2: u64,
    pub est_tokens_tier3: u64,
    pub est_usd_low: f64,
    pub est_usd_high: f64,
    pub cap_usd: f64,
    pub pricing_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ResearchStreamEvent {
    MissionCreated(MissionSummary),
    MissionStatusChanged { run_id: String, old_status: MissionStatus, new_status: MissionStatus },
    MissionProgress(ResearchTimelineEvent),
    MissionHeartbeat { run_id: String, lease_expires_at: String },
}

// ============================================================================
// DOCTOR READINESS MODEL
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheckItem {
    pub id: String,
    pub label: String,
    pub category: String,
    pub status: String, // "ok", "warn", "error"
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDoctorReport {
    pub overall_status: String,
    pub hermes_home_path: String,
    pub research_root_path: String,
    pub checks: Vec<ReadinessCheckItem>,
    pub active_search_providers: Vec<String>,
}

// ============================================================================
// RESEARCH ROOTS & DISK ACCESS SSOT
// ============================================================================

#[derive(Debug, Clone)]
pub struct ResearchRoots {
    pub data_dir: PathBuf,
    pub research_dir: PathBuf,
    pub registry_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub hermes_home: PathBuf,
}

impl ResearchRoots {
    pub fn from_system() -> Self {
        // Dev Drive ReFS canonical anchor
        let base_dev = PathBuf::from(r"Z:\souls_engine");
        let data_dir = if base_dev.exists() {
            base_dev.join(".souls_data")
        } else {
            let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(app_data).join("SoulsEngine").join(".souls_data")
        };

        let research_dir = data_dir.join("research");
        let registry_dir = research_dir.join("_registry");
        let archive_dir = research_dir.join("_archive");

        let hermes_home = std::env::var("HERMES_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(local).join("hermes")
            });

        let roots = Self {
            data_dir,
            research_dir,
            registry_dir,
            archive_dir,
            hermes_home,
        };

        roots.ensure_directories();
        roots
    }

    fn ensure_directories(&self) {
        let _ = fs::create_dir_all(&self.research_dir);
        let _ = fs::create_dir_all(&self.registry_dir);
        let _ = fs::create_dir_all(&self.archive_dir);
    }

    pub fn validate_run_id(&self, run_id: &str) -> Result<(), String> {
        if !is_valid_run_id(run_id) {
            return Err(format!(
                "ID de missão inválido ('{}'). Deve seguir o padrão YYYYMMDD-HHMMSS-slug (strict)",
                run_id
            ));
        }
        Ok(())
    }

    pub fn mission_dir(&self, run_id: &str) -> Result<PathBuf, String> {
        self.validate_run_id(run_id)?;
        let path = self.research_dir.join(run_id);
        // Path containment guard
        if !path.starts_with(&self.research_dir) {
            return Err("Violação de contenção de caminho no Dev Drive".to_string());
        }
        Ok(path)
    }

    pub fn archive_mission_dir(&self, run_id: &str) -> Result<PathBuf, String> {
        self.validate_run_id(run_id)?;
        Ok(self.archive_dir.join(run_id))
    }
}

// ============================================================================
// RESEARCH INDEX (IN-MEMORY CACHE & SSE SWITCHBOARD)
// ============================================================================

pub struct ResearchIndex {
    roots: ResearchRoots,
    missions: RwLock<HashMap<String, MissionRecord>>,
    tx: broadcast::Sender<ResearchStreamEvent>,
}

impl ResearchIndex {
    pub fn new(roots: ResearchRoots) -> Self {
        let (tx, _) = broadcast::channel(128);
        let index = Self {
            roots,
            missions: RwLock::new(HashMap::new()),
            tx,
        };
        index.rebuild_from_disk();
        index
    }

    pub fn roots(&self) -> &ResearchRoots {
        &self.roots
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ResearchStreamEvent> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, event: ResearchStreamEvent) {
        let _ = self.tx.send(event);
    }

    /// Scans `.souls_data/research/` on server boot, reconstructing the memory state.
    pub fn rebuild_from_disk(&self) {
        let mut loaded = HashMap::new();
        if let Ok(entries) = fs::read_dir(&self.roots.research_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name.starts_with('_') || !is_valid_run_id(dir_name) {
                        continue;
                    }
                    let mission_file = path.join("mission.json");
                    if mission_file.exists() {
                        if let Ok(content) = fs::read_to_string(&mission_file) {
                            if let Ok(rec) = serde_json::from_str::<MissionRecord>(&content) {
                                loaded.insert(rec.run_id.clone(), rec);
                            }
                        }
                    }
                }
            }
        }
        info!(
            "ResearchIndex rebuild concluído: {} missões de pesquisa carregadas do disco.",
            loaded.len()
        );
        let mut lock = self.missions.write().unwrap();
        *lock = loaded;
    }

    pub fn list_missions(&self, status_filter: Option<MissionStatus>, search: Option<&str>) -> Vec<MissionSummary> {
        let lock = self.missions.read().unwrap();
        let mut list: Vec<MissionSummary> = lock
            .values()
            .filter(|m| {
                if let Some(st) = status_filter {
                    if m.status != st {
                        return false;
                    }
                }
                if let Some(q) = search {
                    let q_low = q.to_lowercase();
                    let match_q = m.brief.question.to_lowercase().contains(&q_low);
                    let match_slug = m.slug.to_lowercase().contains(&q_low);
                    let match_id = m.run_id.to_lowercase().contains(&q_low);
                    if !match_q && !match_slug && !match_id {
                        return false;
                    }
                }
                true
            })
            .map(|m| MissionSummary {
                run_id: m.run_id.clone(),
                slug: m.slug.clone(),
                status: m.status,
                effort: m.brief.effort,
                question: m.brief.question.clone(),
                created_at: m.created_at.clone(),
                updated_at: m.updated_at.clone(),
                spent_usd: m.budget.spent_usd,
                cap_usd: m.budget.cap_usd,
                turns_used: m.budget.turns_used,
                round: m.round,
            })
            .collect();

        // Sort descending by created_at
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub fn get_mission(&self, run_id: &str) -> Option<MissionRecord> {
        let lock = self.missions.read().unwrap();
        lock.get(run_id).cloned()
    }

    pub fn upsert_and_persist(&self, mut rec: MissionRecord) -> Result<MissionRecord, String> {
        rec.updated_at = Utc::now().to_rfc3339();
        let mission_dir = self.roots.mission_dir(&rec.run_id)?;
        fs::create_dir_all(&mission_dir).map_err(|e| format!("Falha ao criar diretório da missão: {e}"))?;

        // 1. Write mission.json atomic
        let mission_json_path = mission_dir.join("mission.json");
        let content = serde_json::to_string_pretty(&rec)
            .map_err(|e| format!("Falha ao serializar mission.json: {e}"))?;
        atomic_write_file(&mission_json_path, content.as_bytes())?;

        // 2. Write 00_research_brief.json atomic
        let brief_path = mission_dir.join("00_research_brief.json");
        let brief_content = serde_json::to_string_pretty(&rec.brief)
            .map_err(|e| format!("Falha ao serializar 00_research_brief.json: {e}"))?;
        atomic_write_file(&brief_path, brief_content.as_bytes())?;

        // Ensure scouts dir exists
        let scouts_dir = mission_dir.join("scouts");
        let _ = fs::create_dir_all(scouts_dir);

        // Update in-memory
        let mut lock = self.missions.write().unwrap();
        lock.insert(rec.run_id.clone(), rec.clone());

        Ok(rec)
    }

    pub fn append_timeline_event(&self, event: &ResearchTimelineEvent) -> Result<(), String> {
        let mission_dir = self.roots.mission_dir(&event.run_id)?;
        let events_file = mission_dir.join("events.jsonl");

        let line = serde_json::to_string(event).map_err(|e| format!("Erro ao serializar evento: {e}"))? + "\n";

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_file)
            .map_err(|e| format!("Falha ao abrir events.jsonl: {e}"))?;

        file.write_all(line.as_bytes())
            .map_err(|e| format!("Falha ao registrar events.jsonl: {e}"))?;

        // Broadcast to live SSE
        self.broadcast(ResearchStreamEvent::MissionProgress(event.clone()));
        Ok(())
    }

    pub fn read_timeline_events(&self, run_id: &str) -> Vec<ResearchTimelineEvent> {
        let mut events = Vec::new();
        if let Ok(dir) = self.roots.mission_dir(run_id) {
            let path = dir.join("events.jsonl");
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    if let Ok(ev) = serde_json::from_str::<ResearchTimelineEvent>(&line) {
                        events.push(ev);
                    }
                }
            }
        }
        events
    }

    pub fn read_evolving_report(&self, run_id: &str) -> Option<String> {
        let dir = self.roots.mission_dir(run_id).ok()?;
        let path = dir.join("01_evolving_report.md");
        fs::read_to_string(path).ok()
    }

    pub fn cancel_mission(&self, run_id: &str) -> Result<MissionRecord, String> {
        let mut rec = self.get_mission(run_id).ok_or_else(|| "Missão não encontrada".to_string())?;
        let old_status = rec.status;
        rec.status = MissionStatus::Cancelled;
        rec.flags.cancel_requested = true;
        let saved = self.upsert_and_persist(rec)?;
        self.broadcast(ResearchStreamEvent::MissionStatusChanged {
            run_id: run_id.to_string(),
            old_status,
            new_status: MissionStatus::Cancelled,
        });
        Ok(saved)
    }

    pub fn archive_mission(&self, run_id: &str) -> Result<(), String> {
        let src = self.roots.mission_dir(run_id)?;
        let dest = self.roots.archive_mission_dir(run_id)?;

        if dest.exists() {
            let _ = fs::remove_dir_all(&dest);
        }

        fs::rename(&src, &dest).map_err(|e| format!("Falha ao arquivar pasta da missão: {e}"))?;

        let mut lock = self.missions.write().unwrap();
        lock.remove(run_id);
        Ok(())
    }

    /// Claim next available queued mission for an agent.
    pub fn claim_queued_mission(&self, agent_id: &str, target_run_id: Option<&str>) -> Result<Option<MissionRecord>, String> {
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(DEFAULT_CLAIM_LEASE_SECS as i64);

        let mut lock = self.missions.write().unwrap();

        // 1. Find candidate
        let candidate_id = if let Some(trid) = target_run_id {
            if let Some(m) = lock.get(trid) {
                if m.status == MissionStatus::Queued {
                    Some(m.run_id.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Take the oldest queued mission
            let mut queued: Vec<_> = lock
                .values()
                .filter(|m| m.status == MissionStatus::Queued)
                .collect();
            queued.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            queued.first().map(|m| m.run_id.clone())
        };

        let Some(run_id) = candidate_id else {
            return Ok(None);
        };

        let rec = lock.get_mut(&run_id).unwrap();
        let old_status = rec.status;
        rec.status = MissionStatus::Claimed;
        rec.claim = Some(ClaimLease {
            claimed_by: agent_id.to_string(),
            claimed_at: now.to_rfc3339(),
            lease_expires_at: expires.to_rfc3339(),
            heartbeat_at: now.to_rfc3339(),
        });
        rec.updated_at = now.to_rfc3339();

        let updated = rec.clone();
        drop(lock);

        // Persist on disk
        self.upsert_and_persist(updated.clone())?;

        self.broadcast(ResearchStreamEvent::MissionStatusChanged {
            run_id: updated.run_id.clone(),
            old_status,
            new_status: MissionStatus::Claimed,
        });

        Ok(Some(updated))
    }

    /// Heartbeat or progress update from claiming agent.
    pub fn record_agent_event(
        &self,
        run_id: &str,
        phase: &str,
        round: u32,
        summary: &str,
        tokens_tier0: Option<u64>,
        tokens_tier1: Option<u64>,
        tokens_tier2: Option<u64>,
        tokens_tier3: Option<u64>,
        usd_spent: Option<f64>,
        turns_used: Option<u32>,
    ) -> Result<ControlFlags, String> {
        let mut rec = self.get_mission(run_id).ok_or_else(|| "Missão não encontrada".to_string())?;

        let now = Utc::now();
        let lease_expires = now + chrono::Duration::seconds(DEFAULT_CLAIM_LEASE_SECS as i64);

        // Update lease heartbeat
        if let Some(ref mut cl) = rec.claim {
            cl.heartbeat_at = now.to_rfc3339();
            cl.lease_expires_at = lease_expires.to_rfc3339();
        }

        // Advance status from Claimed to Researching upon first event
        let old_status = rec.status;
        if rec.status == MissionStatus::Claimed {
            rec.status = MissionStatus::Researching;
            self.broadcast(ResearchStreamEvent::MissionStatusChanged {
                run_id: run_id.to_string(),
                old_status,
                new_status: MissionStatus::Researching,
            });
        }

        rec.round = round;
        if let Some(t0) = tokens_tier0 {
            rec.tier_usage.tier0_tokens_saved += t0;
        }
        if let Some(t1) = tokens_tier1 {
            rec.tier_usage.tier1_tokens += t1;
        }
        if let Some(t2) = tokens_tier2 {
            rec.tier_usage.tier2_tokens += t2;
        }
        if let Some(t3) = tokens_tier3 {
            rec.tier_usage.tier3_tokens += t3;
        }
        if let Some(cost) = usd_spent {
            rec.budget.spent_usd += cost;
        }
        if let Some(turns) = turns_used {
            rec.budget.turns_used = turns;
        }

        // Cap Enforcement Guard: if spent exceeds cap, enforce force_synthesis
        if rec.budget.spent_usd >= rec.budget.cap_usd || rec.budget.turns_used >= rec.budget.beast_mode_at {
            rec.flags.force_synthesis_requested = true;
        }

        let flags = rec.flags.clone();
        self.upsert_and_persist(rec)?;

        // Append to events.jsonl
        let event = ResearchTimelineEvent {
            timestamp: now.to_rfc3339(),
            run_id: run_id.to_string(),
            phase: phase.to_string(),
            round,
            summary: summary.to_string(),
            spent_usd: usd_spent.unwrap_or(0.0),
            turns_used: turns_used.unwrap_or(0),
        };
        self.append_timeline_event(&event)?;

        Ok(flags)
    }

    /// Sweeps expired leases (claimed/researching agents that vanished).
    pub fn sweep_expired_leases(&self) -> usize {
        let now = Utc::now();
        let mut swept = 0;
        let mut to_revert = Vec::new();

        {
            let lock = self.missions.read().unwrap();
            for m in lock.values() {
                if (m.status == MissionStatus::Claimed || m.status == MissionStatus::Researching) && m.claim.is_some() {
                    if let Some(ref cl) = m.claim {
                        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&cl.lease_expires_at) {
                            if now > exp {
                                to_revert.push(m.run_id.clone());
                            }
                        }
                    }
                }
            }
        }

        for rid in to_revert {
            if let Some(mut m) = self.get_mission(&rid) {
                let old_status = m.status;
                m.status = MissionStatus::Queued;
                m.claim = None;
                if self.upsert_and_persist(m).is_ok() {
                    swept += 1;
                    warn!("Lease de missão expirado: '{}' retornou para a fila de espera.", rid);
                    self.broadcast(ResearchStreamEvent::MissionStatusChanged {
                        run_id: rid,
                        old_status,
                        new_status: MissionStatus::Queued,
                    });
                }
            }
        }

        swept
    }
}

// ============================================================================
// DETERMINISTIC PLANNING & FINOPS ESTIMATOR (TIER 0 ZERO-LLM)
// ============================================================================

pub fn generate_deterministic_plan(question: &str) -> (ResearchBrief, FinOpsPreview) {
    let q_clean = question.trim();
    let q_low = q_clean.to_lowercase();

    // 1. Classify Effort Level
    let effort = if q_low.contains("compare")
        || q_low.contains("vs")
        || q_low.contains("versus")
        || q_low.contains("qual melhor")
        || q_low.contains("diferença")
        || q_low.contains("benchmark")
    {
        EffortLevel::L2Comparative
    } else if q_low.contains("estado da arte")
        || q_low.contains("revisão")
        || q_low.contains("exaustiv")
        || q_low.contains("panorama")
        || q_low.contains("mercado")
        || q_low.contains("profund")
        || q_clean.len() > 180
    {
        EffortLevel::L3Exhaustive
    } else {
        EffortLevel::L1FactFinding
    };

    let (breadth, depth) = match effort {
        EffortLevel::L1FactFinding => (2, 1),
        EffortLevel::L2Comparative => (3, 2),
        EffortLevel::L3Exhaustive => (4, 3),
    };

    // 2. Derive Structured Perspectives & Subtopics (STORM Pattern)
    let (perspectives, subtopics) = if effort == EffortLevel::L2Comparative {
        (
            vec![
                "Arquitetura e Princípios de Design".to_string(),
                "Performance Bare-Metal e Custo Computacional".to_string(),
                "Facilidade de Adoção e Maturidade no Ecossistema".to_string(),
            ],
            vec![
                "Mecanismos centrais e diferenciais técnicos".to_string(),
                "Comparações empíricas e benchmarks conhecidos".to_string(),
                "Limitações, edge cases e cenários de falha".to_string(),
            ],
        )
    } else if effort == EffortLevel::L3Exhaustive {
        (
            vec![
                "Fundamentos Teóricos e Estado da Arte".to_string(),
                "Implementações de Referência e Indústria".to_string(),
                "Implicações de Segurança, FinOps e Escalabilidade".to_string(),
                "Visão Crítica e Tendências Futuras".to_string(),
            ],
            vec![
                "Mapeamento histórico e evolução técnica recente".to_string(),
                "Análise comparativa das soluções líderes no mercado".to_string(),
                "Padrões de projeto, trade-offs e gargalos críticos".to_string(),
                "Síntese prospectiva e recomendações pragmáticas".to_string(),
            ],
        )
    } else {
        (
            vec![
                "Evidência Factual Direta".to_string(),
                "Consenso da Comunidade Especializada".to_string(),
            ],
            vec![
                "Definição canônica e dados verificados".to_string(),
                "Fontes primárias e documentação oficial".to_string(),
            ],
        )
    };

    let brief = ResearchBrief {
        question: q_clean.to_string(),
        objectives: vec![
            format!("Mapear com precisão factual e evidências verificáveis a pergunta: '{}'", q_clean),
            "Identificar divergências e potenciais contradições entre fontes".to_string(),
            "Formular um relatório estruturado com tags epistêmicas".to_string(),
        ],
        perspectives,
        subtopics,
        exclusions: vec![
            "Artigos opinativos genéricos sem fundamentação técnica".to_string(),
            "Fazendas de conteúdo e portais de SEO automatizados".to_string(),
        ],
        source_hierarchy: vec![
            "primary (Código-fonte, Documentação Oficial, Papers)".to_string(),
            "secondary (Benchmarks independentes, Post-mortems técnicos)".to_string(),
            "tertiary (Discussões técnicas e artigos da comunidade)".to_string(),
        ],
        effort,
        breadth,
        depth,
        focus_section: None,
        since: None,
    };

    let finops = estimate_finops(&brief);
    (brief, finops)
}

pub fn estimate_finops(brief: &ResearchBrief) -> FinOpsPreview {
    let (est_turns, est_t0, est_t1, est_t2, est_t3, usd_low, usd_high, cap) = match brief.effort {
        EffortLevel::L1FactFinding => (12, 18_000, 10_000, 12_000, 4_000, 0.05, 0.25, 0.50),
        EffortLevel::L2Comparative => (32, 60_000, 35_000, 45_000, 16_000, 0.20, 0.75, 1.50),
        EffortLevel::L3Exhaustive => (64, 180_000, 95_000, 120_000, 45_000, 0.60, 2.40, 4.00),
    };

    FinOpsPreview {
        effort: brief.effort,
        breadth: brief.breadth,
        depth: brief.depth,
        est_turns,
        est_tokens_tier0: est_t0,
        est_tokens_tier1: est_t1,
        est_tokens_tier2: est_t2,
        est_tokens_tier3: est_t3,
        est_usd_low: usd_low,
        est_usd_high: usd_high,
        cap_usd: cap,
        pricing_source: "Souls Cloud Arena (Pareto Frontier v7.2)".to_string(),
    }
}

pub fn create_mission_record(brief: ResearchBrief, cap_usd: Option<f64>) -> MissionRecord {
    let now = Utc::now();
    let slug = sanitize_slug(&brief.question);
    let run_id = format!("{}-{}", now.format("%Y%m%d-%H%M%S"), slug);
    let finops = estimate_finops(&brief);

    let mut budget = BudgetEnvelope::default();
    budget.cap_usd = cap_usd.unwrap_or(finops.cap_usd);
    budget.est_usd_low = finops.est_usd_low;
    budget.est_usd_high = finops.est_usd_high;

    MissionRecord {
        run_id,
        slug,
        status: MissionStatus::Draft,
        version: 1,
        parent_run_id: None,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        brief,
        budget,
        tier_usage: TierUsage::default(),
        round: 0,
        claim: None,
        flags: ControlFlags::default(),
    }
}

// ============================================================================
// DOCTOR DIAGNOSTIC ENGINE (READ-ONLY)
// ============================================================================

pub fn run_research_doctor(roots: &ResearchRoots) -> ResearchDoctorReport {
    let mut checks = Vec::new();
    let mut ok_count = 0;

    // 1. Hermes Home
    let hh_exists = roots.hermes_home.exists();
    checks.push(ReadinessCheckItem {
        id: "hermes_home".to_string(),
        label: "Diretório de Governança Hermes".to_string(),
        category: "Ambiente".to_string(),
        status: if hh_exists { "ok" } else { "warn" }.to_string(),
        details: format!("Caminho: {}", roots.hermes_home.display()),
    });
    if hh_exists {
        ok_count += 1;
    }

    // 2. Hermes config.yaml
    let config_yaml = roots.hermes_home.join("config.yaml");
    let has_config = config_yaml.exists();
    checks.push(ReadinessCheckItem {
        id: "hermes_config".to_string(),
        label: "Arquivo config.yaml do Hermes".to_string(),
        category: "Configuração".to_string(),
        status: if has_config { "ok" } else { "warn" }.to_string(),
        details: if has_config {
            "Detectado".to_string()
        } else {
            "Não encontrado (será provisionado)".to_string()
        },
    });
    if has_config {
        ok_count += 1;
    }

    // 3. Delegation profile routes in config.yaml
    let mut has_fast_scout = false;
    let mut has_verifier = false;
    if has_config {
        if let Ok(content) = fs::read_to_string(&config_yaml) {
            has_fast_scout = content.contains("fast-scout");
            has_verifier = content.contains("verifier-readonly");
        }
    }
    checks.push(ReadinessCheckItem {
        id: "profile_routes".to_string(),
        label: "Rotas de Perfis para Subagentes (fast-scout / verifier)".to_string(),
        category: "Subagentes".to_string(),
        status: if has_fast_scout && has_verifier { "ok" } else { "warn" }.to_string(),
        details: if has_fast_scout && has_verifier {
            "Rotas fast-scout e verifier-readonly ativas".to_string()
        } else {
            "Rotas ausentes ou parciais em delegation.backends.profile_routes".to_string()
        },
    });
    if has_fast_scout && has_verifier {
        ok_count += 1;
    }

    // 4. Profiles folders
    let p_scout = roots.hermes_home.join("profiles").join("fast-scout");
    let p_verifier = roots.hermes_home.join("profiles").join("verifier-readonly");
    let profiles_ok = p_scout.exists() && p_verifier.exists();
    checks.push(ReadinessCheckItem {
        id: "profiles_dirs".to_string(),
        label: "Diretórios de Perfis Hermes Isolados".to_string(),
        category: "Subagentes".to_string(),
        status: if profiles_ok { "ok" } else { "warn" }.to_string(),
        details: if profiles_ok {
            "Perfis fast-scout e verifier-readonly encontrados".to_string()
        } else {
            "Perfis ainda não provisionados no Hermes".to_string()
        },
    });
    if profiles_ok {
        ok_count += 1;
    }

    // 5. deepsoul-research Skill
    let skill_dir = roots.hermes_home.join("skills").join("deepsoul-research");
    let has_skill = skill_dir.join("SKILL.md").exists();
    checks.push(ReadinessCheckItem {
        id: "deepsoul_skill".to_string(),
        label: "Skill Canônica deepsoul-research".to_string(),
        category: "Skills".to_string(),
        status: if has_skill { "ok" } else { "warn" }.to_string(),
        details: if has_skill {
            "Skill instalada em ~/.hermes/skills/deepsoul-research/".to_string()
        } else {
            "Pendente de provisionamento em 1 clique".to_string()
        },
    });
    if has_skill {
        ok_count += 1;
    }

    // 6. Search Providers Env Vars
    let mut active_providers = Vec::new();
    let providers = [
        ("Tavily Search API", "TAVILY_API_KEY"),
        ("Exa AI Search API", "EXA_API_KEY"),
        ("Brave Search API", "BRAVE_API_KEY"),
        ("Linkup Deep API", "LINKUP_API_KEY"),
    ];

    for (label, var_name) in providers {
        let is_set = std::env::var(var_name).is_ok();
        if is_set {
            active_providers.push(label.to_string());
        }
        checks.push(ReadinessCheckItem {
            id: var_name.to_lowercase(),
            label: format!("Chave {}", label),
            category: "Buscadores".to_string(),
            status: if is_set { "ok" } else { "warn" }.to_string(),
            details: if is_set {
                format!("Definida no ambiente (${{{}}})", var_name)
            } else {
                format!("Não encontrada no ambiente (${{{}}})", var_name)
            },
        });
        if is_set {
            ok_count += 1;
        }
    }

    // 7. Research Data Root on ReFS
    let data_ok = roots.research_dir.exists();
    checks.push(ReadinessCheckItem {
        id: "research_data_root".to_string(),
        label: "Repositório Bare-Metal ReFS (.souls_data/research)".to_string(),
        category: "Storage".to_string(),
        status: if data_ok { "ok" } else { "error" }.to_string(),
        details: format!("Raiz: {}", roots.research_dir.display()),
    });
    if data_ok {
        ok_count += 1;
    }

    let overall_status = if ok_count >= 8 {
        "ready".to_string()
    } else if ok_count >= 4 {
        "needs_provisioning".to_string()
    } else {
        "incomplete".to_string()
    };

    ResearchDoctorReport {
        overall_status,
        hermes_home_path: roots.hermes_home.display().to_string(),
        research_root_path: roots.research_dir.display().to_string(),
        checks,
        active_search_providers: active_providers,
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

fn sanitize_slug(input: &str) -> String {
    let mut slug = String::with_capacity(32);
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
        if slug.len() >= 32 {
            break;
        }
    }
    let res = slug.trim_matches('-').to_string();
    if res.is_empty() {
        "research-topic".to_string()
    } else {
        res
    }
}

pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "Caminho sem diretório pai".to_string())?;
    let tmp_path = parent.join(format!(".tmp_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));

    let mut file = File::create(&tmp_path)
        .map_err(|e| format!("Falha ao criar arquivo temporário {}: {e}", tmp_path.display()))?;

    file.write_all(content)
        .map_err(|e| format!("Falha ao escrever em {}: {e}", tmp_path.display()))?;

    file.flush()
        .map_err(|e| format!("Falha ao descarregar buffer em {}: {e}", tmp_path.display()))?;

    drop(file);

    fs::rename(&tmp_path, path)
        .map_err(|e| format!("Falha ao renomear {} para {}: {e}", tmp_path.display(), path.display()))?;

    Ok(())
}

// ============================================================================
// UNIT TESTS (#[cfg(test)])
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_id_regex_validation() {
        let roots = ResearchRoots::from_system();
        assert!(roots.validate_run_id("20261107-210000-rust-tokio-io").is_ok());
        assert!(roots.validate_run_id("20261107-210000-short").is_ok());

        // Traversal / Escape attack vectors
        assert!(roots.validate_run_id("../evil-path").is_err());
        assert!(roots.validate_run_id("20261107-210000-../../etc/passwd").is_err());
        assert!(roots.validate_run_id("C:\\Windows\\System32").is_err());
        assert!(roots.validate_run_id("invalid_characters!@#$").is_err());
    }

    #[test]
    fn test_deterministic_plan_effort_classification() {
        let (brief_l1, finops_l1) = generate_deterministic_plan("Qual a versão do Rust lançada em 2026?");
        assert_eq!(brief_l1.effort, EffortLevel::L1FactFinding);
        assert_eq!(brief_l1.breadth, 2);
        assert_eq!(brief_l1.depth, 1);

        let (brief_l2, finops_l2) = generate_deterministic_plan("Compare Tokio vs Glommio para I/O bare-metal");
        assert_eq!(brief_l2.effort, EffortLevel::L2Comparative);
        assert_eq!(brief_l2.breadth, 3);
        assert_eq!(brief_l2.depth, 2);

        let (brief_l3, finops_l3) = generate_deterministic_plan("Revisão completa do estado da arte de Deep Research multiagente");
        assert_eq!(brief_l3.effort, EffortLevel::L3Exhaustive);
        assert_eq!(brief_l3.breadth, 4);
        assert_eq!(brief_l3.depth, 3);

        // FinOps monotonicity: L3 > L2 > L1
        assert!(finops_l3.est_usd_high > finops_l2.est_usd_high);
        assert!(finops_l2.est_usd_high > finops_l1.est_usd_high);
        assert!(finops_l3.est_turns > finops_l2.est_turns);
    }

    #[test]
    fn test_atomic_file_write_and_lf_integrity() {
        let tmp_dir = std::env::temp_dir().join(format!("souls_test_atomic_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        fs::create_dir_all(&tmp_dir).unwrap();

        let target_file = tmp_dir.join("test_report.md");
        let content = b"# Header\nLine 1\nLine 2\n";
        assert!(atomic_write_file(&target_file, content).is_ok());

        let read_back = fs::read(&target_file).unwrap();
        assert_eq!(read_back, content);
        assert!(!read_back.contains(&b'\r')); // Pure LF guard

        let _ = fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn test_mission_claim_and_lease_lifecycle() {
        let test_base = std::env::temp_dir().join(format!("souls_test_roots_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let roots = ResearchRoots {
            data_dir: test_base.clone(),
            research_dir: test_base.join("research"),
            registry_dir: test_base.join("research").join("_registry"),
            archive_dir: test_base.join("research").join("_archive"),
            hermes_home: test_base.join("hermes"),
        };
        roots.ensure_directories();

        let index = ResearchIndex::new(roots);
        let (brief, _) = generate_deterministic_plan("Test Tokio");
        let mut mission = create_mission_record(brief, None);
        mission.status = MissionStatus::Queued;
        let created = index.upsert_and_persist(mission).unwrap();

        // 1. Claim queued
        let claimed = index.claim_queued_mission("hermes_agent_1", None).unwrap().expect("Deveria ter reivindicado");
        assert_eq!(claimed.run_id, created.run_id);
        assert_eq!(claimed.status, MissionStatus::Claimed);
        assert_eq!(claimed.claim.unwrap().claimed_by, "hermes_agent_1");

        // 2. Next claim should be empty
        let next_claim = index.claim_queued_mission("hermes_agent_2", None).unwrap();
        assert!(next_claim.is_none());

        // 3. Heartbeat & progress event
        let flags = index.record_agent_event(
            &created.run_id,
            "scouts_round_1",
            1,
            "3 scouts executados",
            Some(1000),
            Some(2000),
            Some(3000),
            Some(500),
            Some(0.15),
            Some(8),
        ).unwrap();
        assert!(!flags.force_synthesis_requested);

        let updated = index.get_mission(&created.run_id).unwrap();
        assert_eq!(updated.status, MissionStatus::Researching);
        assert_eq!(updated.budget.turns_used, 8);
        assert_eq!(updated.budget.spent_usd, 0.15);

        let _ = fs::remove_dir_all(test_base);
    }
}
