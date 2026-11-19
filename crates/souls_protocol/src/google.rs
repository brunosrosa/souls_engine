//! Google Workspace, Life Hub, BMAD Governance, and Notebooks DTOs.
//!
//! Pure data transfer objects shared across the Google integration boundaries,
//! with zero I/O, zero async runtime dependencies, and strict secret redaction
//! (plaintext OAuth tokens are never exposed in profile DTOs).

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// 1. Governance, Accounts & BMAD Policy Enums
// ============================================================================

/// Classification of a registered Google account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoogleAccountKind {
    PersonalGmail,
    WorkspaceEnterprise,
    SecondarySandbox,
}

/// Permission policy governing automated tool access for an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountPermissionPolicy {
    ReadOnly,
    DraftsAndTasksOnly,
    BalancedHotlHitl,
    WhitelistWrite,
}

/// Blast radius classification for BMAD proposal gating and audit logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlastRadiusTier {
    Tier0ReadOnly,
    Tier1SafeReversible,
    Tier2ExternalDestructive,
}

/// Lifecycle status of a BMAD governance proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Failed,
    Expired,
}

// ============================================================================
// 2. Account Profiles, Diagnostics & BMAD Governance DTOs
// ============================================================================

/// Public-safe profile summary for a configured Google account.
///
/// Never carries plaintext access or refresh tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoogleAccountProfileDto {
    pub account_id: String,
    pub label: String,
    pub email: String,
    pub kind: GoogleAccountKind,
    pub policy: AccountPermissionPolicy,
    pub is_primary: bool,
    pub has_oauth: bool,
    pub has_refresh_token: bool,
    pub oauth_testing_mode_warning: bool,
    pub has_nlm_bridge: bool,
    pub secondary_web_opt_in: bool,
    pub write_whitelist_ids: Vec<String>,
    pub updated_at: i64,
}

/// Diagnostic probe status for a companion Google CLI binary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoogleCliProbeDto {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub authenticated: bool,
    pub notes: String,
}

/// Consolidated Google integration health and readiness report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoogleDoctorReportDto {
    pub overall_status: String,
    pub clis: Vec<GoogleCliProbeDto>,
    pub accounts_count: usize,
    pub oauth_production_guidance: String,
    pub keep_enterprise_note: String,
    pub routes_v2_ready: bool,
    pub warnings: Vec<String>,
}

/// BMAD change proposal awaiting or recording human-in-the-loop authorization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BmadProposalDto {
    pub id: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub account_id: String,
    pub service: String,
    pub action: String,
    pub title: String,
    pub summary: String,
    pub tier: BlastRadiusTier,
    pub tainted_origin: bool,
    pub target_resource: String,
    pub before_snapshot: Option<String>,
    pub after_snapshot: Option<String>,
    pub myers_diff: Option<String>,
    pub payload_json: Value,
    pub status: ProposalStatus,
    pub gravity_lock_required: bool,
    pub gravity_lock_token: Option<String>,
    pub rejection_reason: Option<String>,
    pub execution_receipt: Option<Value>,
}

/// Redacted audit log entry for Google tool invocations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEventDto {
    pub timestamp: i64,
    pub session_id: String,
    pub account_id: String,
    pub tool_name: String,
    pub tier: BlastRadiusTier,
    pub tainted: bool,
    pub outcome: String,
    pub redacted_summary: String,
}

// ============================================================================
// 3. Life Hub DTOs (Gmail, Calendar, Drive, Docs, Sheets, Slides, Tasks, etc.)
// ============================================================================

/// Triaged Gmail message summary with priority scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GmailMessageSummaryDto {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub subject: String,
    pub snippet: String,
    pub date: String,
    pub labels: Vec<String>,
    pub priority_score: f32,
    pub category: String,
    pub requires_reply: bool,
}

/// Batch Gmail inbox triage output with untrusted-content framing status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GmailTriageResultDto {
    pub account_email: String,
    pub total_scanned: usize,
    pub priority_messages: Vec<GmailMessageSummaryDto>,
    pub newsletter_count: usize,
    pub untrusted_framed: bool,
}

/// Google Calendar event representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEventDto {
    pub id: String,
    pub summary: String,
    pub start_rfc3339: String,
    pub end_rfc3339: String,
    pub location: Option<String>,
    pub attendees_count: usize,
    pub status: String,
}

/// Available deep-focus time block computed from calendar availability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FocusSlotDto {
    pub start_rfc3339: String,
    pub end_rfc3339: String,
    pub duration_minutes: u32,
    pub label: String,
}

/// Google Drive file metadata record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriveFileDto {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub modified_time: Option<String>,
    pub web_view_link: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Incremental Google Docs scratchpad snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocsScratchpadDto {
    pub doc_id: String,
    pub title: String,
    pub lines: Vec<String>,
    pub updated_at: i64,
    pub revision: u32,
}

/// Quantitative analysis of a Google Sheets range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetRangeAnalysisDto {
    pub spreadsheet_id: String,
    pub range: String,
    pub row_count: usize,
    pub numeric_columns: Vec<String>,
    pub totals_by_column: Vec<(String, f64)>,
    pub category_breakdown: Vec<(String, f64)>,
    pub anomalies: Vec<String>,
    pub projected_monthly_total: Option<f64>,
}

/// Individual slide specification within a planned presentation deck.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideItemDto {
    pub index: usize,
    pub title: String,
    pub bullets: Vec<String>,
    pub speaker_notes: Option<String>,
}

/// Structured Google Slides presentation plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideDeckPlanDto {
    pub title: String,
    pub topic: String,
    pub slides: Vec<SlideItemDto>,
    pub presentation_id: Option<String>,
}

/// Google Tasks item representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoogleTaskDto {
    pub id: String,
    pub tasklist_id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due_rfc3339: Option<String>,
    pub status: String,
    pub updated_at: Option<String>,
}

/// Google Contacts card summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactCardDto {
    pub resource_name: String,
    pub display_name: String,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub organization: Option<String>,
    pub birthday: Option<String>,
}

/// Proactive departure and traffic alert for a geolocated calendar event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommuteRouteAlertDto {
    pub event_id: String,
    pub event_title: String,
    pub event_start_rfc3339: String,
    pub destination: String,
    pub duration_seconds: u64,
    pub static_duration_seconds: u64,
    pub traffic_delay_seconds: i64,
    pub distance_meters: u64,
    pub suggested_departure_rfc3339: String,
    pub advisory: String,
}

/// YouTube video transcript and metadata summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YouTubeVideoSummaryDto {
    pub video_id: String,
    pub url: String,
    pub title: Option<String>,
    pub channel: Option<String>,
    pub language: String,
    pub transcript_text: String,
    pub segment_count: usize,
    pub cached: bool,
}

/// Unified daily briefing aggregating calendar, commute, email, and tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyBriefingDto {
    pub generated_at: i64,
    pub account_email: Option<String>,
    pub oauth_connected: bool,
    pub upcoming_events: Vec<CalendarEventDto>,
    pub focus_slots: Vec<FocusSlotDto>,
    pub commute_alerts: Vec<CommuteRouteAlertDto>,
    pub priority_emails: Vec<GmailMessageSummaryDto>,
    pub pending_tasks: Vec<GoogleTaskDto>,
    pub pending_inbox_count: usize,
    pub notes: Vec<String>,
}

// ============================================================================
// 4. Notebooks & Gems DTOs
// ============================================================================

/// Summary metadata for a NotebookLM or compatible notebook container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookSummaryDto {
    pub notebook_id: String,
    pub title: String,
    pub source_count: usize,
    pub modified_at: Option<String>,
    pub has_stale_sources: bool,
    pub source_backend: String,
}

/// Source entry attached to a notebook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookSourceDto {
    pub source_id: String,
    pub title: String,
    pub source_type: String,
    pub url: Option<String>,
    pub is_stale_drive: Option<bool>,
    pub status: String,
}

/// Generated NotebookLM Studio artifact (audio overview, briefing, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudioArtifactDto {
    pub artifact_id: String,
    pub notebook_id: String,
    pub artifact_type: String,
    pub title: String,
    pub status: String,
    pub local_audio_url: Option<String>,
    pub created_at: Option<String>,
}

/// Custom Gem persona configuration profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GemProfileDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_instruction: String,
    pub model: String,
    pub backend: String,
    pub remote_gem_id: Option<String>,
}

// ============================================================================
// 5. Unit Tests (JSON Round-Trip Verification)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    fn assert_json_round_trip<T>(value: &T)
    where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let serialized = serde_json::to_string(value).expect("serialize to JSON");
        let deserialized: T = serde_json::from_str(&serialized).expect("deserialize from JSON");
        assert_eq!(&deserialized, value);
    }

    #[test]
    fn test_enums_snake_case_and_round_trip() {
        assert_eq!(
            serde_json::to_string(&GoogleAccountKind::PersonalGmail).unwrap(),
            "\"personal_gmail\""
        );
        assert_eq!(
            serde_json::to_string(&GoogleAccountKind::WorkspaceEnterprise).unwrap(),
            "\"workspace_enterprise\""
        );
        assert_eq!(
            serde_json::to_string(&GoogleAccountKind::SecondarySandbox).unwrap(),
            "\"secondary_sandbox\""
        );

        assert_eq!(
            serde_json::to_string(&AccountPermissionPolicy::ReadOnly).unwrap(),
            "\"read_only\""
        );
        assert_eq!(
            serde_json::to_string(&AccountPermissionPolicy::DraftsAndTasksOnly).unwrap(),
            "\"drafts_and_tasks_only\""
        );
        assert_eq!(
            serde_json::to_string(&AccountPermissionPolicy::BalancedHotlHitl).unwrap(),
            "\"balanced_hotl_hitl\""
        );
        assert_eq!(
            serde_json::to_string(&AccountPermissionPolicy::WhitelistWrite).unwrap(),
            "\"whitelist_write\""
        );

        assert_eq!(
            serde_json::to_string(&BlastRadiusTier::Tier0ReadOnly).unwrap(),
            "\"tier0_read_only\""
        );
        assert_eq!(
            serde_json::to_string(&BlastRadiusTier::Tier1SafeReversible).unwrap(),
            "\"tier1_safe_reversible\""
        );
        assert_eq!(
            serde_json::to_string(&BlastRadiusTier::Tier2ExternalDestructive).unwrap(),
            "\"tier2_external_destructive\""
        );

        assert_eq!(
            serde_json::to_string(&ProposalStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&ProposalStatus::Approved).unwrap(),
            "\"approved\""
        );
        assert_eq!(
            serde_json::to_string(&ProposalStatus::Executed).unwrap(),
            "\"executed\""
        );
        assert_eq!(
            serde_json::to_string(&ProposalStatus::Rejected).unwrap(),
            "\"rejected\""
        );
        assert_eq!(
            serde_json::to_string(&ProposalStatus::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&ProposalStatus::Expired).unwrap(),
            "\"expired\""
        );

        for kind in [
            GoogleAccountKind::PersonalGmail,
            GoogleAccountKind::WorkspaceEnterprise,
            GoogleAccountKind::SecondarySandbox,
        ] {
            assert_json_round_trip(&kind);
        }

        for policy in [
            AccountPermissionPolicy::ReadOnly,
            AccountPermissionPolicy::DraftsAndTasksOnly,
            AccountPermissionPolicy::BalancedHotlHitl,
            AccountPermissionPolicy::WhitelistWrite,
        ] {
            assert_json_round_trip(&policy);
        }

        for tier in [
            BlastRadiusTier::Tier0ReadOnly,
            BlastRadiusTier::Tier1SafeReversible,
            BlastRadiusTier::Tier2ExternalDestructive,
        ] {
            assert_json_round_trip(&tier);
        }

        for status in [
            ProposalStatus::Pending,
            ProposalStatus::Approved,
            ProposalStatus::Executed,
            ProposalStatus::Rejected,
            ProposalStatus::Failed,
            ProposalStatus::Expired,
        ] {
            assert_json_round_trip(&status);
        }
    }

    #[test]
    fn test_governance_and_doctor_dtos_round_trip() {
        let profile = GoogleAccountProfileDto {
            account_id: "acct_primary".to_string(),
            label: "Primary Personal".to_string(),
            email: "user@gmail.com".to_string(),
            kind: GoogleAccountKind::PersonalGmail,
            policy: AccountPermissionPolicy::BalancedHotlHitl,
            is_primary: true,
            has_oauth: true,
            has_refresh_token: true,
            oauth_testing_mode_warning: false,
            has_nlm_bridge: true,
            secondary_web_opt_in: false,
            write_whitelist_ids: vec!["sheet_123".to_string()],
            updated_at: 1_763_517_972,
        };
        assert_json_round_trip(&profile);

        let cli_probe = GoogleCliProbeDto {
            name: "gws".to_string(),
            installed: true,
            path: Some("C:\\tools\\gws.exe".to_string()),
            version: Some("0.4.2".to_string()),
            authenticated: true,
            notes: "Ready".to_string(),
        };
        assert_json_round_trip(&cli_probe);

        let doctor = GoogleDoctorReportDto {
            overall_status: "healthy".to_string(),
            clis: vec![cli_probe],
            accounts_count: 1,
            oauth_production_guidance: "Publish OAuth consent screen".to_string(),
            keep_enterprise_note: "Google Keep requires Workspace Enterprise".to_string(),
            routes_v2_ready: true,
            warnings: vec!["testing_mode_expiry".to_string()],
        };
        assert_json_round_trip(&doctor);

        let proposal = BmadProposalDto {
            id: "prop_001".to_string(),
            created_at: 1_763_517_900,
            expires_at: 1_763_518_500,
            account_id: "acct_primary".to_string(),
            service: "gmail".to_string(),
            action: "send_message".to_string(),
            title: "Send reply to Alice".to_string(),
            summary: "Reply confirming Friday meeting".to_string(),
            tier: BlastRadiusTier::Tier2ExternalDestructive,
            tainted_origin: false,
            target_resource: "thread_abc".to_string(),
            before_snapshot: Some("Draft v1".to_string()),
            after_snapshot: Some("Sent message".to_string()),
            myers_diff: Some("- Draft v1\n+ Sent message".to_string()),
            payload_json: json!({ "to": "alice@example.com", "subject": "Re: Friday" }),
            status: ProposalStatus::Pending,
            gravity_lock_required: true,
            gravity_lock_token: Some("glock_xyz".to_string()),
            rejection_reason: None,
            execution_receipt: Some(json!({ "message_id": "msg_99" })),
        };
        assert_json_round_trip(&proposal);

        let audit = AuditEventDto {
            timestamp: 1_763_517_980,
            session_id: "sess_1".to_string(),
            account_id: "acct_primary".to_string(),
            tool_name: "google_gmail_triage".to_string(),
            tier: BlastRadiusTier::Tier0ReadOnly,
            tainted: true,
            outcome: "allowed".to_string(),
            redacted_summary: "Scanned 12 unread messages".to_string(),
        };
        assert_json_round_trip(&audit);
    }

    #[test]
    fn test_life_hub_dtos_round_trip() {
        let email = GmailMessageSummaryDto {
            id: "msg_1".to_string(),
            thread_id: "thr_1".to_string(),
            from: "boss@company.com".to_string(),
            subject: "Q4 Architecture Review".to_string(),
            snippet: "Please review the attached RFC...".to_string(),
            date: "2026-11-19T01:30:00Z".to_string(),
            labels: vec!["INBOX".to_string(), "IMPORTANT".to_string()],
            priority_score: 0.92,
            category: "work".to_string(),
            requires_reply: true,
        };
        assert_json_round_trip(&email);

        let triage = GmailTriageResultDto {
            account_email: "user@gmail.com".to_string(),
            total_scanned: 40,
            priority_messages: vec![email.clone()],
            newsletter_count: 14,
            untrusted_framed: true,
        };
        assert_json_round_trip(&triage);

        let event = CalendarEventDto {
            id: "evt_1".to_string(),
            summary: "Architecture Sync".to_string(),
            start_rfc3339: "2026-11-19T14:00:00-03:00".to_string(),
            end_rfc3339: "2026-11-19T15:00:00-03:00".to_string(),
            location: Some("Av. Paulista, 1000".to_string()),
            attendees_count: 4,
            status: "confirmed".to_string(),
        };
        assert_json_round_trip(&event);

        let focus_slot = FocusSlotDto {
            start_rfc3339: "2026-11-19T09:00:00-03:00".to_string(),
            end_rfc3339: "2026-11-19T11:30:00-03:00".to_string(),
            duration_minutes: 150,
            label: "Morning Deep Work".to_string(),
        };
        assert_json_round_trip(&focus_slot);

        let drive_file = DriveFileDto {
            id: "drv_1".to_string(),
            name: "Budget_2026.xlsx".to_string(),
            mime_type: "application/vnd.google-apps.spreadsheet".to_string(),
            modified_time: Some("2026-11-18T20:00:00Z".to_string()),
            web_view_link: Some("https://drive.google.com/file/d/drv_1/view".to_string()),
            size_bytes: Some(48_128),
        };
        assert_json_round_trip(&drive_file);

        let scratchpad = DocsScratchpadDto {
            doc_id: "doc_1".to_string(),
            title: "Daily Scratchpad".to_string(),
            lines: vec!["- Ship Onda 1".to_string(), "- Validate tests".to_string()],
            updated_at: 1_763_517_990,
            revision: 7,
        };
        assert_json_round_trip(&scratchpad);

        let sheet_analysis = SheetRangeAnalysisDto {
            spreadsheet_id: "sheet_1".to_string(),
            range: "Expenses!A1:D30".to_string(),
            row_count: 29,
            numeric_columns: vec!["Amount".to_string()],
            totals_by_column: vec![("Amount".to_string(), 4250.75)],
            category_breakdown: vec![("Cloud".to_string(), 1200.0), ("Groceries".to_string(), 3050.75)],
            anomalies: vec!["Row 14 exceeds 3x median".to_string()],
            projected_monthly_total: Some(5100.90),
        };
        assert_json_round_trip(&sheet_analysis);

        let slide = SlideItemDto {
            index: 0,
            title: "Executive Overview".to_string(),
            bullets: vec!["Zero-copy architecture".to_string(), "BMAD safety gates".to_string()],
            speaker_notes: Some("Keep this slide under 60 seconds.".to_string()),
        };
        assert_json_round_trip(&slide);

        let deck = SlideDeckPlanDto {
            title: "Souls Engine Q4".to_string(),
            topic: "Bare-metal AI infrastructure".to_string(),
            slides: vec![slide],
            presentation_id: Some("pres_1".to_string()),
        };
        assert_json_round_trip(&deck);

        let task = GoogleTaskDto {
            id: "task_1".to_string(),
            tasklist_id: "list_default".to_string(),
            title: "Review PR #42".to_string(),
            notes: Some("Check LF line endings".to_string()),
            due_rfc3339: Some("2026-11-19T18:00:00Z".to_string()),
            status: "needsAction".to_string(),
            updated_at: Some("2026-11-19T01:00:00Z".to_string()),
        };
        assert_json_round_trip(&task);

        let contact = ContactCardDto {
            resource_name: "people/c12345".to_string(),
            display_name: "Alice Silva".to_string(),
            emails: vec!["alice@example.com".to_string()],
            phones: vec!["+55 11 99999-0000".to_string()],
            organization: Some("Souls Labs".to_string()),
            birthday: Some("1992-05-14".to_string()),
        };
        assert_json_round_trip(&contact);

        let commute = CommuteRouteAlertDto {
            event_id: "evt_1".to_string(),
            event_title: "Architecture Sync".to_string(),
            event_start_rfc3339: "2026-11-19T14:00:00-03:00".to_string(),
            destination: "Av. Paulista, 1000".to_string(),
            duration_seconds: 2700,
            static_duration_seconds: 1800,
            traffic_delay_seconds: 900,
            distance_meters: 12_400,
            suggested_departure_rfc3339: "2026-11-19T13:10:00-03:00".to_string(),
            advisory: "Heavy traffic on Marginal; leave 15m early.".to_string(),
        };
        assert_json_round_trip(&commute);

        let youtube = YouTubeVideoSummaryDto {
            video_id: "dQw4w9WgXcQ".to_string(),
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            title: Some("Systems Engineering Talk".to_string()),
            channel: Some("BareMetalConf".to_string()),
            language: "en".to_string(),
            transcript_text: "Welcome to the systems talk...".to_string(),
            segment_count: 42,
            cached: true,
        };
        assert_json_round_trip(&youtube);

        let briefing = DailyBriefingDto {
            generated_at: 1_763_518_000,
            account_email: Some("user@gmail.com".to_string()),
            oauth_connected: true,
            upcoming_events: vec![event],
            focus_slots: vec![focus_slot],
            commute_alerts: vec![commute],
            priority_emails: vec![email],
            pending_tasks: vec![task],
            pending_inbox_count: 3,
            notes: vec!["All systems nominal".to_string()],
        };
        assert_json_round_trip(&briefing);
    }

    #[test]
    fn test_notebooks_and_gems_dtos_round_trip() {
        let notebook = NotebookSummaryDto {
            notebook_id: "nb_1".to_string(),
            title: "Kernel Research".to_string(),
            source_count: 8,
            modified_at: Some("2026-11-18T22:10:00Z".to_string()),
            has_stale_sources: false,
            source_backend: "notebooklm_mcp".to_string(),
        };
        assert_json_round_trip(&notebook);

        let source = NotebookSourceDto {
            source_id: "src_1".to_string(),
            title: "Windows NT Internals".to_string(),
            source_type: "google_drive".to_string(),
            url: Some("https://drive.google.com/file/d/src_1/view".to_string()),
            is_stale_drive: Some(false),
            status: "ready".to_string(),
        };
        assert_json_round_trip(&source);

        let artifact = StudioArtifactDto {
            artifact_id: "art_1".to_string(),
            notebook_id: "nb_1".to_string(),
            artifact_type: "audio_overview".to_string(),
            title: "Deep Dive Podcast".to_string(),
            status: "completed".to_string(),
            local_audio_url: Some("/api/google/notebooks/nb_1/audio/art_1.mp3".to_string()),
            created_at: Some("2026-11-18T22:30:00Z".to_string()),
        };
        assert_json_round_trip(&artifact);

        let gem = GemProfileDto {
            id: "gem_architect".to_string(),
            name: "Protocol Architect".to_string(),
            description: "Reviews pure DTO contracts and invariants".to_string(),
            system_instruction: "Enforce zero-IO protocol boundaries.".to_string(),
            model: "gemini-2.5-pro".to_string(),
            backend: "gemini_api".to_string(),
            remote_gem_id: None,
        };
        assert_json_round_trip(&gem);
    }
}
