//! Handler for `GET /health` endpoint.
//!
//! Provides bare-metal observability:
//! - Hardware Watchdog telemetry (Free VRAM, GPU temperature, operational zone, thermodynamic barrier).
//! - FrankenSQLite STRICT WAL integrity status.
//! - Active connection count via RAII guard.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Hardware health report structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareHealthReport {
    pub vram_free_mb: u32,
    pub vram_total_mb: u32,
    pub vram_used_mb: u32,
    pub wddm_reserve_mb: u32,
    pub vram_usable_mb: u32,
    pub gpu_temp_c: f32,
    pub operational_zone: String,
    pub barrier_phi: f32,
}

/// Consolidated `/health` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub active_connections: usize,
    pub catnap_active: bool,
    pub hardware: HardwareHealthReport,
    pub sqlite_status: String,
    pub uptime_secs: u64,
}

fn resolve_wddm_reserve_mb(config_path: &std::path::Path) -> u32 {
    if let Ok(raw) = std::fs::read_to_string(config_path) {
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.starts_with("wddm_reserve_mb") {
                continue;
            }
            if let Some((_, val)) = trimmed.split_once('=') {
                if let Ok(parsed) = val.trim().parse::<u32>() {
                    return parsed.clamp(128, 4096);
                }
            }
        }
    }
    souls_inference_runtime::nvml::WDDM_RESERVED_MB
}

/// GET /health handler.
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let _guard = state.acquire_passive_connection();

    // 1. Query Hardware Watchdog snapshot
    let (vram_free_mb, vram_total_mb, gpu_temp_c, zone_str, barrier_phi) = {
        let wd = state.runtime.watchdog();
        let wd_guard = wd.lock().unwrap();
        let snap = wd_guard.snapshot();
        let zone_str = match snap.operational_zone {
            souls_inference_runtime::OperationalZone::Nominal => "Nominal",
            souls_inference_runtime::OperationalZone::Preventive => "Preventive",
            souls_inference_runtime::OperationalZone::Critical => "Critical",
        };
        (
            snap.vram_free_mb,
            snap.vram_total_mb,
            snap.gpu_temp_c,
            zone_str.to_string(),
            snap.barrier_phi,
        )
    };
    let wddm_reserve_mb = resolve_wddm_reserve_mb(&state.evolutive_harness.config_path);
    let vram_used_mb = vram_total_mb.saturating_sub(vram_free_mb);
    let vram_usable_mb = vram_total_mb.saturating_sub(wddm_reserve_mb);

    // 2. Query FrankenSQLite integrity
    let pool = state.memory.pool();
    let sqlite_status = match sqlx::query("SELECT 1;").execute(&*pool).await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {e}"),
    };

    let response = HealthResponse {
        status: if sqlite_status == "ok" {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        active_connections: state.active_connections(),
        catnap_active: state.evolutive_harness.catnap.is_slice_running(),
        hardware: HardwareHealthReport {
            vram_free_mb,
            vram_total_mb,
            vram_used_mb,
            wddm_reserve_mb,
            vram_usable_mb,
            gpu_temp_c,
            operational_zone: zone_str,
            barrier_phi,
        },
        sqlite_status,
        uptime_secs: state.uptime_secs(),
    };

    (StatusCode::OK, Json(response))
}
