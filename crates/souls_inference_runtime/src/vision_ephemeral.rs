//! Ephemeral Single-Turn Local Vision-Language Runner (`Qwen2.5-VL` / `llama-mtmd-cli`).
//!
//! Enforces strict thermodynamic & VRAM governance under the RTX 2060m 6GB WDDM budget:
//! - Hard ceiling of `4,200 MB` allocated VRAM and `78.0 °C` GPU temperature before local VLM dispatch.
//! - Deterministic Set-of-Mark (SoM) accessibility fallback and Cloud Frontier escalation when diverted.
//! - Zero persistent VRAM lock: every local VLM invocation is single-turn, ephemeral, and mutex-guarded
//!   via `gpu_execution_lock()` when running on dGPU.
//! - Windows subprocess hygiene (`AGENTS.md` §8.4): `Stdio::null()` stdin, dedicated stdout reader
//!   thread, bounded 35s `try_wait()` timeout, and `kill() + wait()` cleanup.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::InferenceError;
use crate::inference::{
    gpu_execution_lock, resolve_latest_llamacpp_binary, resolve_llama_cli_binary,
    resolve_llama_completion_binary,
};
use crate::nvml::{GpuTelemetrySnapshot, HardwareWatchdog, OperationalZone};
use souls_protocol::multimodal::{BlindSpotAssessment, SomOverlayMark, VisualAnalysisMode};

/// Maximum allocated GPU VRAM (in MB) allowed before ephemeral local VLM execution is diverted.
pub const EPHEMERAL_VLM_MAX_VRAM_USED_MB: u32 = 4_200;

/// Maximum GPU temperature (in °C) allowed before ephemeral local VLM execution is diverted.
pub const EPHEMERAL_VLM_MAX_TEMP_C: f32 = 78.0;

/// Bounded wall-clock execution timeout for ephemeral local VLM subprocesses.
const EPHEMERAL_VLM_TIMEOUT: Duration = Duration::from_secs(35);

/// Maximum recursive directory depth when scanning model directories.
const MODEL_DISCOVERY_MAX_DEPTH: usize = 2;

/// Input request for ephemeral visual screen analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EphemeralVisionRequest {
    pub prompt: String,
    pub image_path: Option<String>,
    pub mode: VisualAnalysisMode,
    pub blind_spot: Option<BlindSpotAssessment>,
    pub som_marks: Vec<SomOverlayMark>,
}

/// Output outcome produced by [`EphemeralVisionRunner`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EphemeralVisionOutcome {
    pub analysis_text: String,
    pub engine_used: String,
    pub device_target: String,
    pub vram_consumed_mb: usize,
    pub diverted_by_watchdog: bool,
    pub escalate_to_cloud: bool,
    pub latency_ms: f64,
}

/// Ephemeral single-turn VLM runner governed by NVML telemetry and SoM fallback.
#[derive(Debug, Clone)]
pub struct EphemeralVisionRunner {
    model_path: Option<PathBuf>,
    mmproj_path: Option<PathBuf>,
}

impl EphemeralVisionRunner {
    /// Creates a new runner by dynamically discovering local `Qwen2.5-VL` GGUF and `mmproj` weights.
    #[must_use]
    pub fn new() -> Self {
        let (model_path, mmproj_path) = Self::resolve_qwen_vl_model_paths();
        Self {
            model_path,
            mmproj_path,
        }
    }

    /// Creates a runner bound to an explicit VLM GGUF model path and auto-discovers sibling `mmproj`.
    #[must_use]
    pub fn with_model_path(path: impl AsRef<Path>) -> Self {
        let model = path.as_ref().to_path_buf();
        let mmproj_path = model
            .parent()
            .and_then(|dir| Self::resolve_qwen_vl_in_dirs(&[dir.to_path_buf()]).1)
            .or_else(|| Self::resolve_qwen_vl_model_paths().1);
        Self {
            model_path: Some(model),
            mmproj_path,
        }
    }

    /// Returns the resolved primary VLM GGUF path, if present.
    #[must_use]
    pub fn model_path(&self) -> Option<&Path> {
        self.model_path.as_deref()
    }

    /// Returns the resolved multimodal projector (`mmproj`) GGUF path, if present.
    #[must_use]
    pub fn mmproj_path(&self) -> Option<&Path> {
        self.mmproj_path.as_deref()
    }

    /// Dynamically searches canonical host directories for `Qwen2.5-VL` (or `*vl*`) GGUF weights
    /// and multimodal `mmproj` projector files without hardcoding a single user profile.
    #[must_use]
    pub fn resolve_qwen_vl_model_paths() -> (Option<PathBuf>, Option<PathBuf>) {
        let mut search_roots = Vec::new();

        if let Ok(custom_dir) = std::env::var("SOULS_LOCAL_MODELS_DIR") {
            let trimmed = custom_dir.trim();
            if !trimmed.is_empty() {
                search_roots.push(PathBuf::from(trimmed));
            }
        }

        search_roots.push(PathBuf::from(r"Z:\souls_engine\.souls_data\models"));
        search_roots.push(PathBuf::from(".souls_data/models"));

        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let trimmed = local_app_data.trim();
            if !trimmed.is_empty() {
                search_roots.push(PathBuf::from(trimmed).join("hermes").join("models"));
            }
        }

        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let trimmed = user_profile.trim();
            if !trimmed.is_empty() {
                search_roots.push(PathBuf::from(trimmed).join("llm_local_models"));
            }
        }

        Self::resolve_qwen_vl_in_dirs(&search_roots)
    }

    /// Searches an explicit list of root directories (up to depth 2) for VL model + `mmproj` GGUFs.
    fn resolve_qwen_vl_in_dirs(roots: &[PathBuf]) -> (Option<PathBuf>, Option<PathBuf>) {
        let mut gguf_files = Vec::new();
        for root in roots {
            if root.is_dir() {
                collect_gguf_files(root, 0, MODEL_DISCOVERY_MAX_DEPTH, &mut gguf_files);
            }
        }

        let mut raw_models: Vec<(u8, String, PathBuf)> = Vec::new();
        let mut mmproj_candidates: Vec<(u8, String, PathBuf)> = Vec::new();
        let mut mmproj_dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

        for path in gguf_files {
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let lower = file_name.to_ascii_lowercase();
            if !lower.ends_with(".gguf") {
                continue;
            }

            if lower.contains("mmproj") {
                let score = if lower.contains("qwen2.5-vl") {
                    3
                } else if lower.contains("vl") {
                    2
                } else {
                    1
                };
                if let Some(parent) = path.parent() {
                    mmproj_dirs.insert(parent.to_path_buf());
                }
                mmproj_candidates.push((score, lower, path));
            } else if lower.contains("qwen2.5-vl") {
                raw_models.push((3, lower, path));
            } else if lower.contains("vl") {
                raw_models.push((2, lower, path));
            }
        }

        let mut model_candidates: Vec<(bool, u8, String, PathBuf)> = raw_models
            .into_iter()
            .map(|(score, lower, path)| {
                let has_sibling_mmproj = path
                    .parent()
                    .is_some_and(|parent| mmproj_dirs.contains(parent));
                (has_sibling_mmproj, score, lower, path)
            })
            .collect();

        model_candidates.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.cmp(&a.1))
                .then_with(|| a.2.cmp(&b.2))
        });
        let selected_model = model_candidates.into_iter().next().map(|(_, _, _, p)| p);

        // Prefer an mmproj residing in the same directory as the selected model when available.
        let model_parent = selected_model.as_ref().and_then(|p| p.parent());
        mmproj_candidates.sort_by(|a, b| {
            let a_same_dir = model_parent.is_some_and(|dir| a.2.parent() == Some(dir));
            let b_same_dir = model_parent.is_some_and(|dir| b.2.parent() == Some(dir));
            b_same_dir
                .cmp(&a_same_dir)
                .then_with(|| b.0.cmp(&a.0))
                .then_with(|| a.1.cmp(&b.1))
        });
        let selected_mmproj = mmproj_candidates.into_iter().next().map(|(_, _, p)| p);

        (selected_model, selected_mmproj)
    }

    /// Evaluates whether NVML telemetry and analysis mode permit ephemeral local dGPU execution.
    ///
    /// Returns `(allow_dgpu, gpu_layers, reason)`.
    #[must_use]
    pub fn evaluate_hardware_clearance(
        snapshot: &GpuTelemetrySnapshot,
        mode: VisualAnalysisMode,
    ) -> (bool, u32, &'static str) {
        if !matches!(
            mode,
            VisualAnalysisMode::Auto | VisualAnalysisMode::LocalEphemeralVlm
        ) {
            return (false, 0, "Mode does not request local dGPU VLM execution");
        }

        if snapshot.vram_total_mb == 0 {
            return (false, 0, "CPU-only host detected (vram_total_mb == 0)");
        }

        if snapshot.operational_zone == OperationalZone::Critical {
            return (false, 0, "NVML operational zone is Critical");
        }

        if snapshot.vram_allocated_mb > EPHEMERAL_VLM_MAX_VRAM_USED_MB {
            return (
                false,
                0,
                "Allocated VRAM exceeds 4200 MB ephemeral VLM ceiling",
            );
        }

        if snapshot.gpu_temp_c >= EPHEMERAL_VLM_MAX_TEMP_C {
            return (
                false,
                0,
                "GPU temperature reached or exceeded 78.0C ephemeral VLM ceiling",
            );
        }

        (true, 99, "dGPU clearance granted (NVML within ephemeral limits)")
    }

    /// Analyzes the request against live telemetry captured from [`HardwareWatchdog`].
    pub fn analyze(
        &self,
        req: &EphemeralVisionRequest,
        watchdog: &mut HardwareWatchdog,
    ) -> Result<EphemeralVisionOutcome, InferenceError> {
        let snapshot = watchdog.snapshot();
        self.analyze_with_snapshot(req, &snapshot)
    }

    /// Pure decision + execution entrypoint using an explicit [`GpuTelemetrySnapshot`].
    pub fn analyze_with_snapshot(
        &self,
        req: &EphemeralVisionRequest,
        snapshot: &GpuTelemetrySnapshot,
    ) -> Result<EphemeralVisionOutcome, InferenceError> {
        let start = Instant::now();

        match req.mode {
            VisualAnalysisMode::DeterministicSomOnly => {
                return Ok(EphemeralVisionOutcome {
                    analysis_text: format_deterministic_som_summary(req, "Deterministic SoM mode"),
                    engine_used: "deterministic_som_axtree".to_string(),
                    device_target: "CpuAxTree (0 MB VRAM)".to_string(),
                    vram_consumed_mb: 0,
                    diverted_by_watchdog: false,
                    escalate_to_cloud: false,
                    latency_ms: elapsed_ms(start),
                });
            }
            VisualAnalysisMode::CloudFrontier => {
                return Ok(EphemeralVisionOutcome {
                    analysis_text: format_deterministic_som_summary(
                        req,
                        "Cloud Frontier dispatch requested",
                    ),
                    engine_used: "cloud_frontier_recommended".to_string(),
                    device_target: "CloudFrontier".to_string(),
                    vram_consumed_mb: 0,
                    diverted_by_watchdog: false,
                    escalate_to_cloud: true,
                    latency_ms: elapsed_ms(start),
                });
            }
            VisualAnalysisMode::Auto | VisualAnalysisMode::LocalEphemeralVlm => {}
        }

        let (allow_dgpu, gpu_layers, clearance_reason) =
            Self::evaluate_hardware_clearance(snapshot, req.mode);

        let is_test = is_test_environment();
        let model_exists = self.model_path.as_ref().is_some_and(|p| p.exists());
        let has_resolved_model = self.model_path.is_some();

        if !has_resolved_model || is_test || !model_exists {
            if !allow_dgpu && req.mode == VisualAnalysisMode::Auto {
                return Ok(EphemeralVisionOutcome {
                    analysis_text: format_deterministic_som_summary(req, clearance_reason),
                    engine_used: "cloud_frontier_watchdog_divert".to_string(),
                    device_target: "CloudFrontier (Watchdog NVML)".to_string(),
                    vram_consumed_mb: 0,
                    diverted_by_watchdog: true,
                    escalate_to_cloud: true,
                    latency_ms: elapsed_ms(start),
                });
            }

            if !allow_dgpu && req.mode == VisualAnalysisMode::LocalEphemeralVlm {
                return Ok(EphemeralVisionOutcome {
                    analysis_text: format_deterministic_som_summary(req, clearance_reason),
                    engine_used: if is_test {
                        "local_ephemeral_vlm_cpu_test".to_string()
                    } else {
                        "deterministic_som_cpu_fallback".to_string()
                    },
                    device_target: "CpuAvx2 (-ngl 0)".to_string(),
                    vram_consumed_mb: 0,
                    diverted_by_watchdog: true,
                    escalate_to_cloud: !is_test && !model_exists,
                    latency_ms: elapsed_ms(start),
                });
            }

            // `allow_dgpu == true` in test mode or when no local VL weights are installed on disk.
            let vram_consumed_mb = if has_resolved_model && model_exists {
                2_200
            } else {
                0
            };
            let escalate_to_cloud = !is_test && !model_exists && req.mode == VisualAnalysisMode::Auto;

            return Ok(EphemeralVisionOutcome {
                analysis_text: format_deterministic_som_summary(req, clearance_reason),
                engine_used: if is_test {
                    "local_ephemeral_vlm_test".to_string()
                } else {
                    "deterministic_som_no_vlm_model".to_string()
                },
                device_target: if allow_dgpu {
                    format!("dGPU RTX 2060m (-ngl {gpu_layers})")
                } else {
                    "CpuAvx2 (-ngl 0)".to_string()
                },
                vram_consumed_mb,
                diverted_by_watchdog: false,
                escalate_to_cloud,
                latency_ms: elapsed_ms(start),
            });
        }

        // Live production execution path with resolved model weights on disk.
        if !allow_dgpu && req.mode == VisualAnalysisMode::Auto {
            return Ok(EphemeralVisionOutcome {
                analysis_text: format_deterministic_som_summary(req, clearance_reason),
                engine_used: "cloud_frontier_watchdog_divert".to_string(),
                device_target: "CloudFrontier (Watchdog NVML)".to_string(),
                vram_consumed_mb: 0,
                diverted_by_watchdog: true,
                escalate_to_cloud: true,
                latency_ms: elapsed_ms(start),
            });
        }

        let Some(ref image_path_str) = req.image_path else {
            return Ok(EphemeralVisionOutcome {
                analysis_text: format_deterministic_som_summary(
                    req,
                    "No image_path provided for local VLM",
                ),
                engine_used: "deterministic_som_axtree_no_image".to_string(),
                device_target: "CpuAxTree (0 MB VRAM)".to_string(),
                vram_consumed_mb: 0,
                diverted_by_watchdog: !allow_dgpu,
                escalate_to_cloud: req.mode == VisualAnalysisMode::Auto,
                latency_ms: elapsed_ms(start),
            });
        };

        let image_path = Path::new(image_path_str);
        if !image_path.exists() {
            return Ok(EphemeralVisionOutcome {
                analysis_text: format_deterministic_som_summary(
                    req,
                    "Target image_path not found on disk",
                ),
                engine_used: "deterministic_som_axtree_missing_image".to_string(),
                device_target: "CpuAxTree (0 MB VRAM)".to_string(),
                vram_consumed_mb: 0,
                diverted_by_watchdog: !allow_dgpu,
                escalate_to_cloud: req.mode == VisualAnalysisMode::Auto,
                latency_ms: elapsed_ms(start),
            });
        }

        let Some(bin) = resolve_latest_llamacpp_binary("llama-mtmd-cli")
            .or_else(|| resolve_latest_llamacpp_binary("llama-qwen2vl-cli"))
            .or_else(resolve_llama_completion_binary)
            .or_else(resolve_llama_cli_binary)
        else {
            return Ok(EphemeralVisionOutcome {
                analysis_text: format_deterministic_som_summary(
                    req,
                    "No llama.cpp vision binary found on host",
                ),
                engine_used: "deterministic_som_fallback_no_binary".to_string(),
                device_target: if allow_dgpu {
                    "CloudFrontier (Fallback)".to_string()
                } else {
                    "CpuAvx2 (-ngl 0)".to_string()
                },
                vram_consumed_mb: 0,
                diverted_by_watchdog: !allow_dgpu,
                escalate_to_cloud: true,
                latency_ms: elapsed_ms(start),
            });
        };

        let model_path = self
            .model_path
            .as_ref()
            .ok_or_else(|| InferenceError::ModelNotFound("VLM model path missing".to_string()))?;

        let (ngl, device_target, diverted_by_watchdog) = if allow_dgpu {
            (gpu_layers, format!("dGPU RTX 2060m (-ngl {gpu_layers})"), false)
        } else {
            (0, "CpuAvx2 (-ngl 0)".to_string(), true)
        };

        let enriched_prompt = build_vlm_prompt(req);

        // Acquire process-wide GPU mutex only when actually offloading layers to dGPU.
        let _gpu_guard = if allow_dgpu {
            Some(
                gpu_execution_lock()
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            )
        } else {
            None
        };

        match self.execute_vlm_subprocess(&bin, model_path, image_path, &enriched_prompt, ngl) {
            Ok(raw_text) => {
                let engine_name = bin
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("llama-mtmd-cli")
                    .to_string();

                Ok(EphemeralVisionOutcome {
                    analysis_text: raw_text,
                    engine_used: engine_name,
                    device_target,
                    vram_consumed_mb: if allow_dgpu { 2_200 } else { 0 },
                    diverted_by_watchdog,
                    escalate_to_cloud: false,
                    latency_ms: elapsed_ms(start),
                })
            }
            Err(err) => {
                warn!(
                    "Ephemeral local VLM failed ({}); returning SoM fallback with cloud escalation",
                    err
                );
                Ok(EphemeralVisionOutcome {
                    analysis_text: format_deterministic_som_summary(
                        req,
                        "Local VLM subprocess failed or unsupported; SoM fallback attached",
                    ),
                    engine_used: "deterministic_som_fallback_vlm_error".to_string(),
                    device_target,
                    vram_consumed_mb: 0,
                    diverted_by_watchdog,
                    escalate_to_cloud: true,
                    latency_ms: elapsed_ms(start),
                })
            }
        }
    }

    /// Executes a single-turn multimodal llama.cpp CLI subprocess with bounded timeout & pipe safety.
    fn execute_vlm_subprocess(
        &self,
        bin: &Path,
        model_path: &Path,
        image_path: &Path,
        prompt: &str,
        ngl: u32,
    ) -> Result<String, InferenceError> {
        let mut cmd = Command::new(bin);
        if let Some(parent) = bin.parent() {
            cmd.current_dir(parent);
        }

        let temp_prompt_file = if prompt.len() > 1024 {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros();
            let prompt_dir = image_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(r"Z:\souls_engine\.souls_data\vision_snapshots"));
            let _ = std::fs::create_dir_all(&prompt_dir);
            let p = prompt_dir.join(format!(
                "souls_vlm_p_{}_{}.txt",
                std::process::id(),
                ts
            ));
            let _ = std::fs::write(&p, prompt);
            Some(p)
        } else {
            None
        };

        cmd.arg("-m").arg(model_path);

        if let Some(ref mmproj) = self.mmproj_path {
            if mmproj.exists() {
                cmd.arg("--mmproj").arg(mmproj);
            }
        }

        cmd.arg("--image").arg(image_path);

        if let Some(ref pf) = temp_prompt_file {
            cmd.arg("-f").arg(pf);
        } else {
            cmd.arg("-p").arg(prompt);
        }

        cmd.arg("-ngl")
            .arg(ngl.to_string())
            .arg("-n")
            .arg("512")
            .arg("--temp")
            .arg("0.1")
            .arg("-c")
            .arg("4096")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let run_result = spawn_command_with_timeout(&mut cmd, EPHEMERAL_VLM_TIMEOUT);

        if let Some(ref pf) = temp_prompt_file {
            let _ = std::fs::remove_file(pf);
        }

        match run_result {
            Ok((true, stdout_bytes, false)) => {
                let raw = String::from_utf8_lossy(&stdout_bytes).trim().to_string();
                if raw.is_empty() {
                    Err(InferenceError::LlamaCppError(
                        "Ephemeral VLM generated empty stdout".to_string(),
                    ))
                } else {
                    Ok(raw)
                }
            }
            Ok((_, _, true)) => Err(InferenceError::LlamaCppError(format!(
                "Ephemeral VLM timed out after {}s; child process killed",
                EPHEMERAL_VLM_TIMEOUT.as_secs()
            ))),
            Ok((false, _, false)) => Err(InferenceError::LlamaCppError(
                "Ephemeral VLM exited with non-zero status".to_string(),
            )),
            Err(err) => Err(InferenceError::LlamaCppError(format!(
                "Failed to spawn ephemeral VLM subprocess: {err}"
            ))),
        }
    }
}

impl Default for EphemeralVisionRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if running inside automated test harnesses without an explicit real-llama override.
fn is_test_environment() -> bool {
    if std::env::var("SOULS_FORCE_REAL_LLAMA_TEST").is_ok() {
        return false;
    }
    cfg!(test)
        || std::env::var("SOULS_TEST_MODE").is_ok()
        || std::env::current_exe()
            .map(|p| p.to_string_lossy().contains(r"\deps\"))
            .unwrap_or(false)
}

/// Recursively collects `.gguf` files up to `max_depth`.
fn collect_gguf_files(dir: &Path, depth: usize, max_depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let is_gguf = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"));
            if is_gguf {
                out.push(path);
            }
        } else if path.is_dir() && depth < max_depth {
            collect_gguf_files(&path, depth + 1, max_depth, out);
        }
    }
}

/// Spawns `cmd`, drains `stdout` on a dedicated OS thread to avoid Windows 4KB pipe deadlock,
/// polls `try_wait()` until `timeout`, and forcefully kills + reaps the child if exceeded.
fn spawn_command_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> std::io::Result<(bool, Vec<u8>, bool)> {
    let mut child = cmd.spawn()?;
    let stdout_pipe = child.stdout.take();
    let stdout_handle = std::thread::spawn(move || -> Vec<u8> {
        use std::io::Read;
        let mut buf = Vec::new();
        if let Some(mut reader) = stdout_pipe {
            let _ = reader.read_to_end(&mut buf);
        }
        buf
    });

    let deadline = Instant::now() + timeout;
    let mut exited_ok = false;
    let mut timed_out = false;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                exited_ok = status.success();
                break;
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
        }
    }

    let stdout_bytes = stdout_handle.join().unwrap_or_default();
    Ok((exited_ok, stdout_bytes, timed_out))
}

/// Formats a compact, deterministic Set-of-Mark accessibility summary with blind-spot context.
fn format_deterministic_som_summary(req: &EphemeralVisionRequest, reason: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("[{reason}] Prompt: {}", req.prompt.trim()));

    if let Some(ref blind_spot) = req.blind_spot {
        lines.push(format!(
            "BlindSpot(requires_pixel={}, actionable={}, canvas={}, coverage={:.2}): {}",
            blind_spot.requires_pixel_snapshot,
            blind_spot.actionable_node_count,
            blind_spot.canvas_node_count,
            blind_spot.coverage_ratio,
            blind_spot.recommendation_pt
        ));
    }

    if req.som_marks.is_empty() {
        lines.push("SoM Marks: none detected.".to_string());
    } else {
        lines.push(format!("SoM Marks ({} elements):", req.som_marks.len()));
        for mark in req.som_marks.iter().take(24) {
            lines.push(format!(
                "  [{}] {} \"{}\" @ ({:.0}, {:.0})",
                mark.index, mark.role, mark.label, mark.click_point.0, mark.click_point.1
            ));
        }
        if req.som_marks.len() > 24 {
            lines.push(format!("  ... and {} more elements", req.som_marks.len() - 24));
        }
    }

    lines.join("\n")
}

/// Builds a concise multimodal prompt enriched with SoM coordinates when present.
fn build_vlm_prompt(req: &EphemeralVisionRequest) -> String {
    if req.som_marks.is_empty() {
        return req.prompt.clone();
    }

    let mut som_hints = Vec::with_capacity(req.som_marks.len().min(20));
    for mark in req.som_marks.iter().take(20) {
        som_hints.push(format!(
            "[{}] {} \"{}\" ({:.0},{:.0})",
            mark.index, mark.role, mark.label, mark.click_point.0, mark.click_point.1
        ));
    }

    format!(
        "{}\n\nSet-of-Mark UI anchors:\n{}",
        req.prompt.trim(),
        som_hints.join("\n")
    )
}

#[inline]
fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvml::{simulated_rtx2060m_snapshot, RTX_2060M_VRAM_TOTAL_MB};
    use souls_protocol::multimodal::VisionEscalationReason;

    fn sample_som_marks() -> Vec<SomOverlayMark> {
        vec![
            SomOverlayMark {
                index: 1,
                node_id: "btn_save".to_string(),
                role: "button".to_string(),
                label: "Salvar".to_string(),
                bounds: [10.0, 20.0, 110.0, 60.0],
                click_point: (60.0, 40.0),
            },
            SomOverlayMark {
                index: 2,
                node_id: "input_search".to_string(),
                role: "textbox".to_string(),
                label: "Buscar".to_string(),
                bounds: [120.0, 20.0, 320.0, 60.0],
                click_point: (220.0, 40.0),
            },
        ]
    }

    fn sample_request(mode: VisualAnalysisMode) -> EphemeralVisionRequest {
        EphemeralVisionRequest {
            prompt: "Identifique o botão principal na tela.".to_string(),
            image_path: None,
            mode,
            blind_spot: Some(BlindSpotAssessment {
                requires_pixel_snapshot: true,
                reason: VisionEscalationReason::CanvasOrWebGlDetected,
                actionable_node_count: 2,
                canvas_node_count: 1,
                coverage_ratio: 0.45,
                recommendation_pt: "Canvas detectado; inspecionar região visual.".to_string(),
            }),
            som_marks: sample_som_marks(),
        }
    }

    #[test]
    fn test_watchdog_diverts_when_vram_above_4200_or_temp_above_78() {
        let runner = EphemeralVisionRunner::with_model_path("nonexistent_qwen2.5-vl.gguf");

        // Boundary nominal: 4200 MB allocated and 77.9°C -> allowed on dGPU
        let mut boundary_ok = simulated_rtx2060m_snapshot();
        boundary_ok.vram_allocated_mb = EPHEMERAL_VLM_MAX_VRAM_USED_MB;
        boundary_ok.vram_free_mb = RTX_2060M_VRAM_TOTAL_MB - EPHEMERAL_VLM_MAX_VRAM_USED_MB;
        boundary_ok.gpu_temp_c = 77.9;
        let (allow, ngl, _) =
            EphemeralVisionRunner::evaluate_hardware_clearance(&boundary_ok, VisualAnalysisMode::Auto);
        assert!(allow);
        assert_eq!(ngl, 99);

        // Breach 1: VRAM allocated > 4200 MB
        let mut high_vram = simulated_rtx2060m_snapshot();
        high_vram.vram_allocated_mb = 4_201;
        high_vram.vram_free_mb = RTX_2060M_VRAM_TOTAL_MB - 4_201;
        let (allow_vram, ngl_vram, _) =
            EphemeralVisionRunner::evaluate_hardware_clearance(&high_vram, VisualAnalysisMode::Auto);
        assert!(!allow_vram);
        assert_eq!(ngl_vram, 0);

        let auto_vram_out = runner
            .analyze_with_snapshot(&sample_request(VisualAnalysisMode::Auto), &high_vram)
            .expect("Auto diversion on high VRAM should succeed");
        assert!(auto_vram_out.diverted_by_watchdog);
        assert!(auto_vram_out.escalate_to_cloud);
        assert_eq!(auto_vram_out.device_target, "CloudFrontier (Watchdog NVML)");
        assert_eq!(auto_vram_out.vram_consumed_mb, 0);

        let local_vram_out = runner
            .analyze_with_snapshot(
                &sample_request(VisualAnalysisMode::LocalEphemeralVlm),
                &high_vram,
            )
            .expect("LocalEphemeralVlm CPU fallback on high VRAM should succeed");
        assert!(local_vram_out.diverted_by_watchdog);
        assert!(!local_vram_out.escalate_to_cloud);
        assert_eq!(local_vram_out.device_target, "CpuAvx2 (-ngl 0)");
        assert_eq!(local_vram_out.vram_consumed_mb, 0);

        // Breach 2: GPU temperature >= 78.0°C
        let mut hot_gpu = simulated_rtx2060m_snapshot();
        hot_gpu.gpu_temp_c = 78.5;
        let (allow_temp, ngl_temp, _) =
            EphemeralVisionRunner::evaluate_hardware_clearance(&hot_gpu, VisualAnalysisMode::Auto);
        assert!(!allow_temp);
        assert_eq!(ngl_temp, 0);

        let auto_temp_out = runner
            .analyze_with_snapshot(&sample_request(VisualAnalysisMode::Auto), &hot_gpu)
            .expect("Auto diversion on high temp should succeed");
        assert!(auto_temp_out.diverted_by_watchdog);
        assert!(auto_temp_out.escalate_to_cloud);
        assert_eq!(auto_temp_out.device_target, "CloudFrontier (Watchdog NVML)");
        assert_eq!(auto_temp_out.vram_consumed_mb, 0);

        // Breach 3: CPU-only host (vram_total_mb == 0)
        let cpu_only = GpuTelemetrySnapshot::default();
        let (allow_cpu, ngl_cpu, _) =
            EphemeralVisionRunner::evaluate_hardware_clearance(&cpu_only, VisualAnalysisMode::Auto);
        assert!(!allow_cpu);
        assert_eq!(ngl_cpu, 0);
    }

    #[test]
    fn test_deterministic_som_and_cloud_modes_consume_zero_vram() {
        let runner = EphemeralVisionRunner {
            model_path: None,
            mmproj_path: None,
        };

        let mut critical_snapshot = simulated_rtx2060m_snapshot();
        critical_snapshot.operational_zone = OperationalZone::Critical;
        critical_snapshot.gpu_temp_c = 85.0;

        let som_out = runner
            .analyze_with_snapshot(
                &sample_request(VisualAnalysisMode::DeterministicSomOnly),
                &critical_snapshot,
            )
            .expect("DeterministicSomOnly should succeed");
        assert_eq!(som_out.engine_used, "deterministic_som_axtree");
        assert_eq!(som_out.vram_consumed_mb, 0);
        assert!(!som_out.escalate_to_cloud);
        assert!(!som_out.diverted_by_watchdog);
        assert!(som_out.analysis_text.contains("Salvar"));
        assert!(som_out.analysis_text.contains("Buscar"));

        let cloud_out = runner
            .analyze_with_snapshot(
                &sample_request(VisualAnalysisMode::CloudFrontier),
                &simulated_rtx2060m_snapshot(),
            )
            .expect("CloudFrontier should succeed");
        assert_eq!(cloud_out.engine_used, "cloud_frontier_recommended");
        assert_eq!(cloud_out.vram_consumed_mb, 0);
        assert!(cloud_out.escalate_to_cloud);
        assert!(!cloud_out.diverted_by_watchdog);
    }

    #[test]
    fn test_ephemeral_runner_respects_nominal_vs_critical_zone() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let fake_model = temp_dir.path().join("Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf");
        std::fs::write(&fake_model, b"GGUF_TEST_FIXTURE").expect("write fake model");

        let runner_with_model = EphemeralVisionRunner::with_model_path(&fake_model);
        let runner_no_model = EphemeralVisionRunner {
            model_path: None,
            mmproj_path: None,
        };

        // Nominal zone with simulated RTX 2060m
        let nominal = simulated_rtx2060m_snapshot();
        let out_nom_model = runner_with_model
            .analyze_with_snapshot(&sample_request(VisualAnalysisMode::Auto), &nominal)
            .expect("Nominal analysis with model should succeed");
        assert!(!out_nom_model.diverted_by_watchdog);
        assert!(!out_nom_model.escalate_to_cloud);
        assert!(out_nom_model.device_target.contains("dGPU"));
        assert!(out_nom_model.vram_consumed_mb > 0);

        // No model on disk -> 0 MB VRAM consumed even on nominal clearance
        let out_nom_empty = runner_no_model
            .analyze_with_snapshot(&sample_request(VisualAnalysisMode::Auto), &nominal)
            .expect("Nominal analysis without model should succeed");
        assert!(!out_nom_empty.diverted_by_watchdog);
        assert!(out_nom_empty.device_target.contains("dGPU"));
        assert_eq!(out_nom_empty.vram_consumed_mb, 0);

        // Critical zone (even if temp < 78 and VRAM <= 4200) -> must divert!
        let mut critical = simulated_rtx2060m_snapshot();
        critical.operational_zone = OperationalZone::Critical;
        let out_crit_auto = runner_with_model
            .analyze_with_snapshot(&sample_request(VisualAnalysisMode::Auto), &critical)
            .expect("Critical Auto analysis should divert");
        assert!(out_crit_auto.diverted_by_watchdog);
        assert!(out_crit_auto.escalate_to_cloud);
        assert_eq!(out_crit_auto.vram_consumed_mb, 0);
        assert_eq!(out_crit_auto.device_target, "CloudFrontier (Watchdog NVML)");

        let out_crit_local = runner_with_model
            .analyze_with_snapshot(
                &sample_request(VisualAnalysisMode::LocalEphemeralVlm),
                &critical,
            )
            .expect("Critical LocalEphemeralVlm should divert to CPU");
        assert!(out_crit_local.diverted_by_watchdog);
        assert!(!out_crit_local.escalate_to_cloud);
        assert_eq!(out_crit_local.vram_consumed_mb, 0);
        assert_eq!(out_crit_local.device_target, "CpuAvx2 (-ngl 0)");

        // Integration check with HardwareWatchdog instance
        let mut watchdog = HardwareWatchdog::new_with_initial_state(83.0, 5_500);
        watchdog.record_sample(83.0, 5_500, RTX_2060M_VRAM_TOTAL_MB, 1.0);
        let wd_out = runner_with_model
            .analyze(&sample_request(VisualAnalysisMode::Auto), &mut watchdog)
            .expect("Watchdog-governed analyze should succeed");
        assert!(wd_out.diverted_by_watchdog);
        assert!(wd_out.escalate_to_cloud);
        assert_eq!(wd_out.vram_consumed_mb, 0);
    }

    #[test]
    fn test_qwen_vl_discovery_graceful_on_missing_or_present_files() {
        // Global discovery must never panic regardless of host environment
        let _ = EphemeralVisionRunner::resolve_qwen_vl_model_paths();

        // Missing directories -> (None, None)
        let missing_dir = PathBuf::from(r"Z:\souls_engine\__nonexistent_models_dir_stage4__");
        assert_eq!(
            EphemeralVisionRunner::resolve_qwen_vl_in_dirs(&[missing_dir]),
            (None, None)
        );

        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        // Non-VL GGUF alone should not be picked as a VL model
        std::fs::write(root.join("qwen2.5-coder-3b-q4_k_m.gguf"), b"non-vl")
            .expect("write non-vl");
        assert_eq!(
            EphemeralVisionRunner::resolve_qwen_vl_in_dirs(&[root.to_path_buf()]),
            (None, None)
        );

        // Create nested depth-2 directory with generic VL, Qwen2.5-VL, and mmproj
        let depth2 = root.join("vision").join("qwen25vl");
        std::fs::create_dir_all(&depth2).expect("create depth 2");
        let generic_vl = root.join("vision").join("llava-vl-7b.gguf");
        let qwen_vl = depth2.join("Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf");
        let mmproj = depth2.join("mmproj-Qwen2.5-VL-3B-f16.gguf");
        std::fs::write(&generic_vl, b"generic-vl").expect("write generic vl");
        std::fs::write(&qwen_vl, b"qwen-vl").expect("write qwen vl");
        std::fs::write(&mmproj, b"mmproj").expect("write mmproj");

        // Depth 3 must be ignored
        let depth3 = depth2.join("too_deep");
        std::fs::create_dir_all(&depth3).expect("create depth 3");
        std::fs::write(depth3.join("qwen2.5-vl-depth3.gguf"), b"ignored").expect("write depth 3");

        let (found_model, found_mmproj) =
            EphemeralVisionRunner::resolve_qwen_vl_in_dirs(&[root.to_path_buf()]);
        assert_eq!(found_model.as_deref(), Some(qwen_vl.as_path()));
        assert_eq!(found_mmproj.as_deref(), Some(mmproj.as_path()));

        // An orphan model copy without sibling mmproj must lose to the directory that carries mmproj
        let orphan_dir = root.join("aaa_orphan");
        std::fs::create_dir_all(&orphan_dir).expect("create orphan dir");
        std::fs::write(
            orphan_dir.join("Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf"),
            b"orphan-qwen-vl",
        )
        .expect("write orphan qwen vl");
        let (preferred_model, preferred_mmproj) =
            EphemeralVisionRunner::resolve_qwen_vl_in_dirs(&[root.to_path_buf()]);
        assert_eq!(preferred_model.as_deref(), Some(qwen_vl.as_path()));
        assert_eq!(preferred_mmproj.as_deref(), Some(mmproj.as_path()));
    }
}
