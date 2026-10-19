//! REST and SSE endpoints for `souls_llm_inventory`.
//!
//! Exposes hardware diagnostics, model recommendations, Hugging Face search,
//! sequential download queue control, and live SSE download progress.

use axum::extract::{Path as AxumPath, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::state::AppState;
use souls_llm_inventory::{
    delete_model_file, get_curated_catalog, get_recommended_triad, probe_hardware,
    resolve_canonical_models_dir, scan_local_models,
};

/// Hardware diagnostic endpoint.
pub async fn inventory_hardware_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    Json(json!({
        "status": "success",
        "hardware": diag
    }))
}

/// Curated recommendations and recommended 3-tier triad.
pub async fn inventory_recommendations_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    let root = resolve_canonical_models_dir();

    let installed_models = scan_local_models(&root, &diag).unwrap_or_default();
    let installed_filenames: Vec<String> = installed_models
        .into_iter()
        .map(|m| m.file_name)
        .collect();

    let triad = get_recommended_triad(&diag, &installed_filenames);
    let catalog = get_curated_catalog(&diag, &installed_filenames);

    Json(json!({
        "status": "success",
        "hardware": diag,
        "triad": triad,
        "catalog": catalog,
        "installed_count": installed_filenames.len()
    }))
}

/// Locally installed physical models with zero-copy metadata and hardware fit.
pub async fn inventory_models_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();
    let root = resolve_canonical_models_dir();

    let models = scan_local_models(&root, &diag).unwrap_or_default();

    Json(json!({
        "status": "success",
        "root_dir": root.display().to_string(),
        "total_models": models.len(),
        "models": models
    }))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

/// On-demand search query against the Hugging Face Hub.
pub async fn inventory_search_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let search_term = query.q.or(query.query).unwrap_or_default();

    if search_term.trim().is_empty() {
        return Json(json!({
            "status": "success",
            "query": "",
            "results": []
        }));
    }

    match state.hf_client.search_models(&search_term, query.limit.unwrap_or(20)).await {
        Ok(results) => Json(json!({
            "status": "success",
            "query": search_term,
            "results": results
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
            "results": []
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct RepoFilesQuery {
    pub repo_id: String,
}

/// Live inspection of GGUF quantization files in a specific Hugging Face repository.
pub async fn inventory_repo_files_handler(
    State(state): State<AppState>,
    Query(query): Query<RepoFilesQuery>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let diag = probe_hardware();

    match state.hf_client.get_repo_gguf_files(&query.repo_id).await {
        Ok(files) => {
            let files_with_fit: Vec<Value> = files
                .into_iter()
                .map(|f| {
                    let fit = souls_llm_inventory::evaluate_model_fit(
                        &diag,
                        f.size_bytes,
                        8192,
                        3_000_000_000,
                    );
                    json!({
                        "file_name": f.file_name,
                        "size_bytes": f.size_bytes,
                        "size_gb": f.size_gb,
                        "download_url": f.download_url,
                        "sha256": f.sha256,
                        "quant_tag": f.quant_tag,
                        "hardware_fit": fit
                    })
                })
                .collect();

            Json(json!({
                "status": "success",
                "repo_id": query.repo_id,
                "files": files_with_fit
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
            "files": []
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct EnqueueDownloadPayload {
    pub repo_id: String,
    pub file_name: String,
    pub download_url: Option<String>,
    pub expected_sha256: Option<String>,
}

/// Enqueues a model for background downloading.
pub async fn inventory_download_handler(
    State(state): State<AppState>,
    Json(payload): Json<EnqueueDownloadPayload>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    let dest_dir = resolve_canonical_models_dir();

    let download_url = payload.download_url.unwrap_or_else(|| {
        format!(
            "https://huggingface.co/{}/resolve/main/{}",
            payload.repo_id, payload.file_name
        )
    });

    match state
        .inventory_downloader
        .enqueue_download(
            &payload.repo_id,
            &payload.file_name,
            &download_url,
            payload.expected_sha256,
            &dest_dir,
        )
        .await
    {
        Ok(item) => Json(json!({
            "status": "enqueued",
            "item": item
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

/// Lists all active, queued, paused, and completed downloads.
pub async fn inventory_list_downloads_handler(State(state): State<AppState>) -> Json<Value> {
    let _guard = state.acquire_connection();
    let downloads = state.inventory_downloader.list_downloads().await;
    Json(json!({
        "status": "success",
        "downloads": downloads
    }))
}

/// Pauses an active or queued download.
pub async fn inventory_pause_download_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match state.inventory_downloader.pause_download(&id).await {
        Ok(_) => Json(json!({ "status": "paused", "id": id })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Resumes a paused or failed download.
pub async fn inventory_resume_download_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match state.inventory_downloader.resume_download(&id).await {
        Ok(_) => Json(json!({ "status": "resumed", "id": id })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Cancels a download and purges intermediate .part files.
pub async fn inventory_cancel_download_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match state.inventory_downloader.cancel_download(&id).await {
        Ok(_) => Json(json!({ "status": "cancelled", "id": id })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Safely removes a model from disk.
pub async fn inventory_delete_model_handler(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Json<Value> {
    let _guard = state.acquire_connection();
    match delete_model_file(&filename) {
        Ok(_) => Json(json!({ "status": "deleted", "filename": filename })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Real-time Server-Sent Events stream of download progress.
pub async fn inventory_download_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.inventory_downloader.subscribe_progress();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(item) => {
            let data = serde_json::to_string(&item).unwrap_or_default();
            Some(Ok(Event::default().event("download_progress").data(data)))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(10)))
}
