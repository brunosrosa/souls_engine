//! Route definitions and router construction for `souls_server`.

pub mod chat;
pub mod health;
pub mod models;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Constructs the primary Axum router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .route("/v1/models", get(models::models_handler))
        .route("/v1/chat/completions", post(chat::chat_completions_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
