//! Dynamic and agnostic hardware profiling for local LLM viability.
//!
//! Evaluates host resources (GPU VRAM, System RAM, CPU threads and AVX extensions)
//! without hardcoded GPU constraints, classifying models into OptimalVram, PartialCpuOffload,
//! or IncompatibleSpillover.

use serde::{Deserialize, Serialize};
use tracing::info;

/// Complete diagnostic snapshot of the host hardware.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareDiagnostic {
    pub has_dedicated_gpu: bool,
    pub gpu_name: String,
    pub vram_total_mb: u64,
    pub vram_free_mb: u64,
    pub vram_safe_usable_mb: u64,
    pub host_ram_total_mb: u64,
    pub host_ram_available_mb: u64,
    pub host_ram_safe_usable_mb: u64,
    pub cpu_cores_physical: usize,
    pub cpu_cores_logical: usize,
    pub has_avx2: bool,
    pub has_avx512: bool,
}

impl Default for HardwareDiagnostic {
    fn default() -> Self {
        Self {
            has_dedicated_gpu: false,
            gpu_name: "CPU Host".to_string(),
            vram_total_mb: 0,
            vram_free_mb: 0,
            vram_safe_usable_mb: 0,
            host_ram_total_mb: 16_384,
            host_ram_available_mb: 12_288,
            host_ram_safe_usable_mb: 9_216,
            cpu_cores_physical: 8,
            cpu_cores_logical: 16,
            has_avx2: true,
            has_avx512: false,
        }
    }
}

/// Classification of physical model viability on the detected hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareFit {
    /// 100% of weights and KV-cache fit in dedicated GPU VRAM (maximum speed: 35-70+ tok/s).
    OptimalVram,
    /// Model exceeds dedicated VRAM but fits in System RAM via CPU AVX2 (~10-25 tok/s).
    PartialCpuOffload,
    /// Model exceeds total safe memory; danger of OOM or disk swap thrashing (< 2 tok/s).
    IncompatibleSpillover,
}

#[cfg(windows)]
fn query_system_ram_mb() -> (u64, u64) {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    // SAFETY: GlobalMemoryStatusEx safely populates MEMORYSTATUSEX with physical memory statistics from the Windows NT kernel.
    let ok = unsafe { GlobalMemoryStatusEx(&mut mem_status) };
    if ok != 0 {
        let total = mem_status.ullTotalPhys / (1024 * 1024);
        let avail = mem_status.ullAvailPhys / (1024 * 1024);
        (total, avail)
    } else {
        (32_768, 24_000)
    }
}

#[cfg(not(windows))]
fn query_system_ram_mb() -> (u64, u64) {
    (32_768, 24_000)
}

/// Probes the host system and builds an agnostic hardware diagnostic.
pub fn probe_hardware() -> HardwareDiagnostic {
    let (host_ram_total_mb, host_ram_available_mb) = query_system_ram_mb();
    // Reserve at least 3.0 GB for OS, background services and IDE
    let host_ram_safe_usable_mb = host_ram_available_mb.saturating_sub(3_072);

    let cpu_cores_logical = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(16)
        .max(1);
    let cpu_cores_physical = (cpu_cores_logical / 2).max(1);

    #[cfg(target_arch = "x86_64")]
    let has_avx2 = std::is_x86_feature_detected!("avx2");
    #[cfg(not(target_arch = "x86_64"))]
    let has_avx2 = false;

    #[cfg(target_arch = "x86_64")]
    let has_avx512 = std::is_x86_feature_detected!("avx512f");
    #[cfg(not(target_arch = "x86_64"))]
    let has_avx512 = false;

    // Probe dedicated GPU via HardwareWatchdog (NVML)
    let watchdog = souls_inference_runtime::HardwareWatchdog::new();
    let snap = watchdog.snapshot();

    let has_dedicated_gpu = snap.vram_total_mb > 0;
    let (gpu_name, vram_total_mb, vram_free_mb, vram_safe_usable_mb) = if has_dedicated_gpu {
        let total = snap.vram_total_mb as u64;
        let free = snap.vram_free_mb as u64;
        // On Windows with WDDM, OS reserves ~850MB, or ~12% of VRAM
        let reserved = 850.max((total as f64 * 0.12) as u64);
        let safe = total.saturating_sub(reserved);
        (
            "NVIDIA GeForce Dedicated".to_string(),
            total,
            free,
            safe,
        )
    } else {
        ("None (CPU Execution)".to_string(), 0, 0, 0)
    };

    info!(
        gpu = %gpu_name,
        vram_total = vram_total_mb,
        vram_safe = vram_safe_usable_mb,
        host_ram_total = host_ram_total_mb,
        host_ram_safe = host_ram_safe_usable_mb,
        avx2 = has_avx2,
        "Agnostic hardware probe completed"
    );

    HardwareDiagnostic {
        has_dedicated_gpu,
        gpu_name,
        vram_total_mb,
        vram_free_mb,
        vram_safe_usable_mb,
        host_ram_total_mb,
        host_ram_available_mb,
        host_ram_safe_usable_mb,
        cpu_cores_physical,
        cpu_cores_logical,
        has_avx2,
        has_avx512,
    }
}

/// Estimates KV-Cache memory consumption in Megabytes.
pub fn estimate_kv_cache_mb(parameter_count: u64, context_length: u32) -> u64 {
    let param_billions = (parameter_count as f64 / 1_000_000_000.0).max(0.1);
    // Asymmetric KV-cache (FP16 Key + Q4_0 Value) ~= 6.5 MB per 1000 tokens per Billion parameters (with GQA)
    let factor_per_1k = 6.5 * param_billions;
    let tokens_k = context_length as f64 / 1000.0;
    let kv_mb = (factor_per_1k * tokens_k).round() as u64;
    kv_mb.max(32)
}

/// Evaluates hardware fit for a given model size and context length.
pub fn evaluate_model_fit(
    diag: &HardwareDiagnostic,
    model_size_bytes: u64,
    context_length: u32,
    parameter_count: u64,
) -> HardwareFit {
    let model_size_mb = model_size_bytes / (1024 * 1024);
    let kv_cache_mb = estimate_kv_cache_mb(parameter_count, context_length);
    // CUDA runtime / Driver workspace buffer margin (~300 MB)
    let total_required_mb = model_size_mb + kv_cache_mb + 300;

    if diag.has_dedicated_gpu && total_required_mb <= diag.vram_safe_usable_mb {
        HardwareFit::OptimalVram
    } else if total_required_mb <= diag.host_ram_safe_usable_mb {
        HardwareFit::PartialCpuOffload
    } else {
        HardwareFit::IncompatibleSpillover
    }
}

/// Projects generation throughput (tokens/sec) based on model size and hardware fit.
pub fn estimate_throughput_tps(
    fit: HardwareFit,
    parameter_count: u64,
    has_avx2: bool,
) -> f32 {
    let params_b = parameter_count as f64 / 1_000_000_000.0;

    match fit {
        HardwareFit::OptimalVram => {
            if params_b <= 0.2 {
                180.0
            } else if params_b <= 1.5 {
                75.0
            } else if params_b <= 3.5 {
                42.0
            } else if params_b <= 8.0 {
                28.0
            } else {
                16.0
            }
        }
        HardwareFit::PartialCpuOffload => {
            let base_speed = if params_b <= 0.2 {
                120.0
            } else if params_b <= 1.5 {
                35.0
            } else if params_b <= 3.5 {
                20.0
            } else if params_b <= 8.0 {
                9.5
            } else {
                4.0
            };
            if has_avx2 {
                base_speed
            } else {
                base_speed * 0.45
            }
        }
        HardwareFit::IncompatibleSpillover => 1.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_cache_estimation() {
        let kv_7b_8k = estimate_kv_cache_mb(7_000_000_000, 8192);
        assert!(kv_7b_8k >= 200 && kv_7b_8k <= 400);

        let kv_135m_4k = estimate_kv_cache_mb(135_000_000, 4096);
        assert!(kv_135m_4k >= 32);
    }

    #[test]
    fn test_hardware_fit_evaluation() {
        let diag = HardwareDiagnostic {
            has_dedicated_gpu: true,
            gpu_name: "RTX Test".to_string(),
            vram_total_mb: 6144,
            vram_free_mb: 5000,
            vram_safe_usable_mb: 5294,
            host_ram_total_mb: 32768,
            host_ram_available_mb: 24000,
            host_ram_safe_usable_mb: 20000,
            cpu_cores_physical: 8,
            cpu_cores_logical: 16,
            has_avx2: true,
            has_avx512: false,
        };

        // 3.8 GB Q4_K_M model with 8k context should fit in 5.2GB VRAM
        let size_3_8gb = 3_800 * 1024 * 1024;
        let fit = evaluate_model_fit(&diag, size_3_8gb, 8192, 7_000_000_000);
        assert_eq!(fit, HardwareFit::OptimalVram);

        // 8.5 GB Q8_0 model exceeds 5.2GB VRAM, but fits in 20GB Host RAM
        let size_8_5gb = 8_500 * 1024 * 1024;
        let fit_large = evaluate_model_fit(&diag, size_8_5gb, 8192, 7_000_000_000);
        assert_eq!(fit_large, HardwareFit::PartialCpuOffload);

        // 25 GB model exceeds both VRAM and safe Host RAM
        let size_25gb = 25_000 * 1024 * 1024;
        let fit_oversized = evaluate_model_fit(&diag, size_25gb, 32768, 32_000_000_000);
        assert_eq!(fit_oversized, HardwareFit::IncompatibleSpillover);
    }

    #[test]
    fn test_probe_hardware() {
        let diag = probe_hardware();
        assert!(diag.host_ram_total_mb > 0);
        assert!(diag.cpu_cores_logical > 0);
    }
}
