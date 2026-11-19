//! REST endpoints for the BMAD Agent Inbox & Gravity Lock Approval Plane.
//!
//! Exposes proposal inspection, Myers Diff review, Gravity Lock authorization,
//! proposal rejection, and redacted security audit trail querying.

#![forbid(unsafe_code)]

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use souls_protocol::google::ProposalStatus;

use crate::state::AppState;

/// Query filter for listing Agent Inbox proposals.
#[derive(Debug, Deserialize)]
pub struct ListProposalsQuery {
    pub status: Option<String>,
}

/// Request payload for approving a pending BMAD proposal.
#[derive(Debug, Deserialize)]
pub struct ApproveProposalRequest {
    #[serde(default)]
    pub gravity_lock_confirmed: bool,
    pub gravity_lock_token: Option<String>,
}

/// Request payload for rejecting a pending BMAD proposal.
#[derive(Debug, Deserialize)]
pub struct RejectProposalRequest {
    pub reason: String,
}

/// Query parameters for fetching the redacted audit log.
#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub limit: Option<usize>,
}

/// `GET /api/v1/inbox/proposals`
pub async fn list_proposals_handler(
    State(state): State<AppState>,
    Query(query): Query<ListProposalsQuery>,
) -> impl IntoResponse {
    let status_filter = match query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        None | Some("all") => None,
        Some("pending") => Some(ProposalStatus::Pending),
        Some("approved") => Some(ProposalStatus::Approved),
        Some("executed") => Some(ProposalStatus::Executed),
        Some("rejected") => Some(ProposalStatus::Rejected),
        Some("failed") => Some(ProposalStatus::Failed),
        Some("expired") => Some(ProposalStatus::Expired),
        Some(other) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": format!("Invalid proposal status filter '{other}'. Expected pending|approved|executed|rejected|failed|expired|all.")
                })),
            );
        }
    };

    let proposals = state.google_hub.inbox.list(status_filter);
    let pending_count = state.google_hub.inbox.pending_count();

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "total": proposals.len(),
            "pending_count": pending_count,
            "proposals": proposals
        })),
    )
}

/// `GET /api/v1/inbox/proposals/:id`
pub async fn get_proposal_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.google_hub.inbox.get(&id) {
        Some(proposal) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "proposal": proposal
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": format!("Proposal '{id}' not found")
            })),
        ),
    }
}

/// `POST /api/v1/inbox/proposals/:id/approve`
pub async fn approve_proposal_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ApproveProposalRequest>,
) -> impl IntoResponse {
    if !payload.gravity_lock_confirmed {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "error",
                "message": "Gravity Lock confirmation (`gravity_lock_confirmed: true`) is required to approve Agent Inbox proposals."
            })),
        );
    }

    match state
        .google_hub
        .execute_approved_proposal(&id, payload.gravity_lock_token.as_deref())
        .await
    {
        Ok(proposal) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "proposal": proposal
            })),
        ),
        Err((code, msg)) => (
            code,
            Json(json!({
                "status": "error",
                "message": msg
            })),
        ),
    }
}

/// `POST /api/v1/inbox/proposals/:id/reject`
pub async fn reject_proposal_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RejectProposalRequest>,
) -> impl IntoResponse {
    let reason = if payload.reason.trim().is_empty() {
        "Rejected by operator".to_string()
    } else {
        payload.reason.trim().to_string()
    };

    match state.google_hub.inbox.reject(&id, &reason) {
        Ok(proposal) => {
            state.google_hub.record_audit_event(
                "inbox_rejection",
                &proposal.account_id,
                &format!("{}.{}", proposal.service, proposal.action),
                proposal.tier,
                proposal.tainted_origin,
                "rejected",
                &format!("Rejected proposal '{}': {}", proposal.id, reason),
            );
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "proposal": proposal
                })),
            )
        }
        Err(err) => {
            let msg = err.to_string();
            let status = if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (
                status,
                Json(json!({
                    "status": "error",
                    "message": msg
                })),
            )
        }
    }
}

/// `GET /api/v1/inbox/audit`
pub async fn list_audit_handler(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let events = state.google_hub.inbox.list_audit(limit);
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "total": events.len(),
            "events": events
        })),
    )
}
