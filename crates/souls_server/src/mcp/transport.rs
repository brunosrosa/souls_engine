//! # Axum MCP Dual-Mount Transport (`souls_server::mcp::transport`)
//!
//! Exposes both modern Streamable HTTP (`/mcp` endpoint) and legacy HTTP+SSE (`/sse` & `/messages`).

use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Json, Response};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, info, warn};

use crate::mcp::dispatcher::dispatch_jsonrpc;
use crate::mcp::session::MCP_KEEP_ALIVE_INTERVAL;
use crate::state::AppState;

/// POST /mcp — Modern Streamable HTTP JSON-RPC handler (MCP 2025-03-26+).
pub async fn mcp_post_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    let _guard = state.acquire_connection();

    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    debug!("Streamable HTTP POST /mcp received (session: {session_id:?})");

    if let Some(resp) = dispatch_jsonrpc(&state, payload, session_id.as_deref()).await {
        (StatusCode::OK, Json(resp)).into_response()
    } else {
        (StatusCode::ACCEPTED, Json(json!({}))).into_response()
    }
}

/// GET /mcp — Modern Streamable HTTP SSE stream handler.
pub async fn mcp_get_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let _guard = state.acquire_connection();

    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let (tx, rx) = mpsc::channel::<Event>(128);

    let session = if let Some(ref id) = session_id {
        if let Some(existing) = state.mcp_sessions.get_session(id).await {
            existing
        } else {
            state.mcp_sessions.create_session(tx).await
        }
    } else {
        state.mcp_sessions.create_session(tx).await
    };

    info!("Streamable HTTP GET /mcp connected for session: {}", session.session_id);

    let stream = ReceiverStream::new(rx).map(Ok::<Event, Infallible>);

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(MCP_KEEP_ALIVE_INTERVAL)
                .text("keepalive"),
        )
        .into_response()
}

/// GET /sse — Legacy MCP HTTP+SSE stream initiation (MCP 2024-11-05).
pub async fn sse_legacy_handler(
    State(state): State<AppState>,
) -> Response {
    let _guard = state.acquire_connection();

    let (tx, rx) = mpsc::channel::<Event>(128);
    let session = state.mcp_sessions.create_session(tx.clone()).await;
    let sess_id = session.session_id.clone();

    info!("Legacy MCP GET /sse connected; assigned session: {sess_id}");

    // Emit initial endpoint handshake event as mandated by MCP 2024-11-05
    let endpoint_url = format!("/messages?sessionId={sess_id}");
    let initial_event = Event::default().event("endpoint").data(endpoint_url);

    let _ = tx.send(initial_event).await;

    let stream = ReceiverStream::new(rx).map(Ok::<Event, Infallible>);

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(MCP_KEEP_ALIVE_INTERVAL)
                .text("keepalive"),
        )
        .into_response()
}

/// POST /messages — Legacy MCP HTTP+SSE message receiver (MCP 2024-11-05).
pub async fn messages_legacy_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<Value>,
) -> Response {
    let _guard = state.acquire_connection();

    let session_id = match params.get("sessionId") {
        Some(s) => s.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing sessionId query parameter" })),
            )
                .into_response();
        }
    };

    debug!("Legacy POST /messages received for session: {session_id}");

    // Verify session exists
    let session = match state.mcp_sessions.get_session(&session_id).await {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Session not found: {session_id}") })),
            )
                .into_response();
        }
    };

    // Dispatch JSON-RPC
    let response_opt = dispatch_jsonrpc(&state, payload, Some(&session_id)).await;

    if let Some(resp) = response_opt {
        let resp_str = serde_json::to_string(&resp).unwrap_or_default();
        let event = Event::default().event("message").data(resp_str);

        if let Err(e) = session.tx.send(event).await {
            warn!("Failed to deliver message to SSE stream for {session_id}: {e}");
        }
    }

    (StatusCode::ACCEPTED, "Accepted").into_response()
}
