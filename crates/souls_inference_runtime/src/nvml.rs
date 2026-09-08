//! Hardware Watchdog and GPU Telemetry via NVML for RTX 2060m (6GB GDDR6).
//!
//! Enforces ADR-004 thermodynamic governance:
//! - EWMA thermal peak prediction (\alpha = 0.35)
//! - 3 Operational Zones: Nominal, Preventive, Critical
//! - Thermodynamic barrier function \Phi(T_GPU, V_free, k)
//! - Graceful fail-soft when NVML is not available (e.g. CI / CPU-only hosts)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Canonical VRAM capacity of the target NVIDIA GeForce RTX 2060m.
pub const RTX_2060M_VRAM_TOTAL_MB: u32 = 6_144;

/// WDDM 3.x reserved memory allowance in MB.
pub const WDDM_RESERVED_MB: u32 = 850;

/// Maximum allowable usable VRAM threshold before risking PCIe spillover.
pub const MAX_USABLE_VRAM_MB: u32 = 5_294;

/// Thermal threshold for Nominal zone (°C).
pub const THERMAL_NOMINAL_MAX_C: f32 = 75.0;

/// Critical thermal threshold triggering immediate GPU shutdown (°C).
pub const THERMAL_CRITICAL_C: f32 = 82.0;

/// Free VRAM threshold for Nominal zone (MB).
pub const VRAM_FREE_NOMINAL_MIN_MB: u32 = 1_400;

/// Critical free VRAM threshold triggering immediate GPU shutdown (MB).
pub const VRAM_FREE_CRITICAL_MB: u32 = 800;

/// Default EWMA smoothing factor (\alpha) as calibrated in ADR-004.
pub const EWMA_ALPHA: f32 = 0.35;

/// Empirical penalty weights for the preventive barrier function.
pub const BARRIER_LAMBDA: f32 = 2.5;
pub const BARRIER_MU: f32 = 1.8;

/// Three operational zones governing GPU model dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalZone {
    /// Full clearance for Tier 1 local GPU inference.
    Nominal,
    /// Memory freeze / token ceiling reduction; reroute heavy tasks.
    Preventive,
    /// Immediate GPU halt; abort local inference and fallback to CPU/Cloud.
    Critical,
}

/// Instantaneous GPU telemetry snapshot with EWMA predictions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuTelemetrySnapshot {
    /// Instantaneous GPU temperature in Celsius.
    pub gpu_temp_c: f32,
    /// Projected GPU temperature in next 3 seconds based on EWMA rate.
    pub projected_temp_c: f32,
    /// Instantaneous allocated VRAM in Megabytes.
    pub vram_allocated_mb: u32,
    /// Instantaneous free VRAM in Megabytes.
    pub vram_free_mb: u32,
    /// Total VRAM reported by driver in Megabytes.
    pub vram_total_mb: u32,
    /// Fan speed percentage (0..100).
    pub fan_speed_pct: u32,
    /// Power draw in Watts.
    pub power_usage_w: f32,
    /// Temperature derivative EWMA (\Delta T / \Delta t in °C/s).
    pub temp_ewma_rate: f32,
    /// VRAM allocation smoothed via EWMA in MB.
    pub vram_ewma_mb: f32,
    /// Active operational zone determined by physical conditions.
    pub operational_zone: OperationalZone,
    /// Thermodynamic barrier value \Phi for Tier 1.
    pub barrier_phi: f32,
}

impl Default for GpuTelemetrySnapshot {
    fn default() -> Self {
        Self {
            gpu_temp_c: 45.0,
            projected_temp_c: 45.0,
            vram_allocated_mb: 850,
            vram_free_mb: 5_294,
            vram_total_mb: RTX_2060M_VRAM_TOTAL_MB,
            fan_speed_pct: 30,
            power_usage_w: 15.0,
            temp_ewma_rate: 0.0,
            vram_ewma_mb: 850.0,
            operational_zone: OperationalZone::Nominal,
            barrier_phi: 0.0,
        }
    }
}

/// Computes the thermodynamic barrier \Phi(T_GPU, V_free, k) according to ADR-004.
///
/// Returns:
/// - 0.0 if k is not a GPU tier or conditions are Nominal.
/// - \lambda * ((1400 - V_free) / 600) + \mu * ((T_GPU - 75) / 7) in Preventive zone.
/// - f32::INFINITY in Critical zone.
pub fn compute_thermodynamic_barrier(gpu_temp_c: f32, vram_free_mb: u32, tier: &str) -> f32 {
    let is_gpu_tier = tier.eq_ignore_ascii_case("tier 1")
        || tier.eq_ignore_ascii_case("tier1")
        || tier.eq_ignore_ascii_case("tier 2")
        || tier.eq_ignore_ascii_case("tier2")
        || tier.eq_ignore_ascii_case("gpu");

    if !is_gpu_tier {
        return 0.0;
    }

    // Critical boundary
    if vram_free_mb <= VRAM_FREE_CRITICAL_MB || gpu_temp_c >= THERMAL_CRITICAL_C {
        return f32::INFINITY;
    }

    // Nominal boundary
    if vram_free_mb > VRAM_FREE_NOMINAL_MIN_MB && gpu_temp_c < THERMAL_NOMINAL_MAX_C {
        return 0.0;
    }

    // Preventive boundary
    let vram_penalty = (1_400.0 - vram_free_mb as f32).max(0.0) / 600.0;
    let temp_penalty = (gpu_temp_c - 75.0).max(0.0) / 7.0;

    BARRIER_LAMBDA * vram_penalty + BARRIER_MU * temp_penalty
}

/// Evaluates the operational zone given temperature, projected temperature, and free VRAM.
pub fn evaluate_operational_zone(
    gpu_temp_c: f32,
    projected_temp_c: f32,
    vram_free_mb: u32,
) -> OperationalZone {
    // Critical: V_free <= 800MB or T_GPU >= 82°C
    if vram_free_mb <= VRAM_FREE_CRITICAL_MB || gpu_temp_c >= THERMAL_CRITICAL_C {
        OperationalZone::Critical
    // Preventive: 800 < V_free <= 1400MB or 75°C <= T_GPU < 82°C or projected to hit >= 82°C in 3s
    } else if vram_free_mb <= VRAM_FREE_NOMINAL_MIN_MB
        || gpu_temp_c >= THERMAL_NOMINAL_MAX_C
        || projected_temp_c >= THERMAL_CRITICAL_C
    {
        OperationalZone::Preventive
    } else {
        OperationalZone::Nominal
    }
}

/// Hardware Watchdog managing NVML telemetry and EWMA smoothing.
pub struct HardwareWatchdog {
    last_temp_c: f32,
    last_sample_time: Instant,
    temp_ewma_rate: f32,
    vram_ewma_mb: f32,
    current_snapshot: GpuTelemetrySnapshot,
    nvml_available: bool,
    packed_state: Arc<AtomicU64>,
}

impl HardwareWatchdog {
    /// Creates a new watchdog instance, safely probing NVML via Windows API.
    pub fn new() -> Self {
        let (nvml_available, initial_snapshot) = Self::probe_nvml_safe();
        let packed = Arc::new(AtomicU64::new(Self::pack_state(
            initial_snapshot.vram_allocated_mb,
            initial_snapshot.gpu_temp_c,
            initial_snapshot.operational_zone,
        )));

        Self {
            last_temp_c: initial_snapshot.gpu_temp_c,
            last_sample_time: Instant::now(),
            temp_ewma_rate: 0.0,
            vram_ewma_mb: initial_snapshot.vram_allocated_mb as f32,
            current_snapshot: initial_snapshot,
            nvml_available,
            packed_state: packed,
        }
    }

    /// Creates a watchdog with an explicit initial state (ideal for testing and simulation).
    pub fn new_with_initial_state(initial_temp_c: f32, initial_vram_allocated_mb: u32) -> Self {
        let total = RTX_2060M_VRAM_TOTAL_MB;
        let free = total.saturating_sub(initial_vram_allocated_mb);
        let zone = evaluate_operational_zone(initial_temp_c, initial_temp_c, free);
        let barrier = compute_thermodynamic_barrier(initial_temp_c, free, "Tier 1");

        let snapshot = GpuTelemetrySnapshot {
            gpu_temp_c: initial_temp_c,
            projected_temp_c: initial_temp_c,
            vram_allocated_mb: initial_vram_allocated_mb,
            vram_free_mb: free,
            vram_total_mb: total,
            fan_speed_pct: 45,
            power_usage_w: 25.0,
            temp_ewma_rate: 0.0,
            vram_ewma_mb: initial_vram_allocated_mb as f32,
            operational_zone: zone,
            barrier_phi: barrier,
        };

        let packed = Arc::new(AtomicU64::new(Self::pack_state(
            initial_vram_allocated_mb,
            initial_temp_c,
            zone,
        )));

        Self {
            last_temp_c: initial_temp_c,
            last_sample_time: Instant::now(),
            temp_ewma_rate: 0.0,
            vram_ewma_mb: initial_vram_allocated_mb as f32,
            current_snapshot: snapshot,
            nvml_available: false,
            packed_state: packed,
        }
    }

    /// Safely attempts to probe NVML without crashing if DLL or device is absent.
    fn probe_nvml_safe() -> (bool, GpuTelemetrySnapshot) {
        // SAFETY: NVML dynamically queries nvml.dll via nvml-wrapper C FFI.
        // We catch errors and fail-soft into synthetic safe metrics.
        match nvml_wrapper::Nvml::init() {
            Ok(nvml) => match nvml.device_by_index(0) {
                Ok(device) => {
                    let temp = device
                        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                        .map(|t| t as f32)
                        .unwrap_or(45.0);

                    let mem = device.memory_info().ok();
                    let vram_total_mb = mem
                        .as_ref()
                        .map(|m| (m.total / (1024 * 1024)) as u32)
                        .unwrap_or(RTX_2060M_VRAM_TOTAL_MB);
                    let vram_free_mb = mem
                        .as_ref()
                        .map(|m| (m.free / (1024 * 1024)) as u32)
                        .unwrap_or(5_294);
                    let vram_allocated_mb = mem
                        .as_ref()
                        .map(|m| (m.used / (1024 * 1024)) as u32)
                        .unwrap_or(850);

                    let fan = device.fan_speed(0).unwrap_or(30);
                    let power = device
                        .power_usage()
                        .map(|p| p as f32 / 1000.0)
                        .unwrap_or(15.0);

                    let zone = evaluate_operational_zone(temp, temp, vram_free_mb);
                    let barrier = compute_thermodynamic_barrier(temp, vram_free_mb, "Tier 1");

                    info!("NVML Hardware Watchdog initialized successfully on dGPU.");
                    (
                        true,
                        GpuTelemetrySnapshot {
                            gpu_temp_c: temp,
                            projected_temp_c: temp,
                            vram_allocated_mb,
                            vram_free_mb,
                            vram_total_mb,
                            fan_speed_pct: fan,
                            power_usage_w: power,
                            temp_ewma_rate: 0.0,
                            vram_ewma_mb: vram_allocated_mb as f32,
                            operational_zone: zone,
                            barrier_phi: barrier,
                        },
                    )
                }
                Err(err) => {
                    warn!("NVML device query failed: {}. Entering fail-soft mode.", err);
                    (false, GpuTelemetrySnapshot::default())
                }
            },
            Err(err) => {
                warn!(
                    "NVML driver init failed: {}. Host running in CPU/Dev fail-soft mode.",
                    err
                );
                (false, GpuTelemetrySnapshot::default())
            }
        }
    }

    /// Manually updates telemetry state with explicit parameters (used for sampling and testing).
    pub fn record_sample(
        &mut self,
        gpu_temp_c: f32,
        vram_allocated_mb: u32,
        vram_total_mb: u32,
        dt_secs: f32,
    ) -> GpuTelemetrySnapshot {
        let dt = if dt_secs > 0.001 { dt_secs } else { 1.0 };
        let delta_t = gpu_temp_c - self.last_temp_c;
        let instant_rate = delta_t / dt;

        // EWMA on thermal rate: EWMA_t = \alpha * (\Delta T / \Delta t) + (1 - \alpha) * EWMA_{t-1}
        self.temp_ewma_rate =
            EWMA_ALPHA * instant_rate + (1.0 - EWMA_ALPHA) * self.temp_ewma_rate;

        // EWMA on VRAM: VRAM_EWMA_t = \alpha * V_t + (1 - \alpha) * VRAM_EWMA_{t-1}
        self.vram_ewma_mb =
            EWMA_ALPHA * (vram_allocated_mb as f32) + (1.0 - EWMA_ALPHA) * self.vram_ewma_mb;

        // Projected temperature in next 3 seconds
        let projected_temp_c = gpu_temp_c + (3.0 * self.temp_ewma_rate);

        let vram_free_mb = vram_total_mb.saturating_sub(vram_allocated_mb);
        let zone = evaluate_operational_zone(gpu_temp_c, projected_temp_c, vram_free_mb);
        let barrier = compute_thermodynamic_barrier(gpu_temp_c, vram_free_mb, "Tier 1");

        self.last_temp_c = gpu_temp_c;
        self.last_sample_time = Instant::now();

        let snapshot = GpuTelemetrySnapshot {
            gpu_temp_c,
            projected_temp_c,
            vram_allocated_mb,
            vram_free_mb,
            vram_total_mb,
            fan_speed_pct: if gpu_temp_c > 70.0 { 85 } else { 45 },
            power_usage_w: 45.0,
            temp_ewma_rate: self.temp_ewma_rate,
            vram_ewma_mb: self.vram_ewma_mb,
            operational_zone: zone,
            barrier_phi: barrier,
        };

        self.current_snapshot = snapshot.clone();

        let packed = Self::pack_state(vram_allocated_mb, gpu_temp_c, zone);
        self.packed_state.store(packed, Ordering::Release);

        snapshot
    }

    /// Performs one sampling tick using live NVML if available, or maintains healthy state.
    pub fn sample_once(&mut self) -> GpuTelemetrySnapshot {
        if self.nvml_available {
            // SAFETY: Safe dynamic FFI query on already validated NVML runtime.
            if let Ok(nvml) = nvml_wrapper::Nvml::init() {
                if let Ok(device) = nvml.device_by_index(0) {
                    let temp = device
                        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                        .map(|t| t as f32)
                        .unwrap_or(self.last_temp_c);

                    let mem = device.memory_info().ok();
                    let vram_total_mb = mem
                        .as_ref()
                        .map(|m| (m.total / (1024 * 1024)) as u32)
                        .unwrap_or(RTX_2060M_VRAM_TOTAL_MB);
                    let vram_allocated_mb = mem
                        .as_ref()
                        .map(|m| (m.used / (1024 * 1024)) as u32)
                        .unwrap_or(self.current_snapshot.vram_allocated_mb);

                    let dt = self.last_sample_time.elapsed().as_secs_f32();
                    return self.record_sample(temp, vram_allocated_mb, vram_total_mb, dt);
                }
            }
        }

        // Fail-soft mode: maintain last known or default snapshot
        let dt = self.last_sample_time.elapsed().as_secs_f32();
        self.record_sample(
            self.current_snapshot.gpu_temp_c,
            self.current_snapshot.vram_allocated_mb,
            self.current_snapshot.vram_total_mb,
            dt,
        )
    }

    /// Returns the most recent telemetry snapshot.
    pub fn snapshot(&self) -> GpuTelemetrySnapshot {
        self.current_snapshot.clone()
    }

    /// Checks if Tier 1 local GPU inference is safe to run.
    pub fn is_gpu_safe(&self) -> bool {
        self.current_snapshot.operational_zone != OperationalZone::Critical
            && !self.current_snapshot.barrier_phi.is_infinite()
    }

    /// Compacts critical metrics into a single atomic u64 for O(1) lock-free querying.
    #[inline]
    fn pack_state(vram_mb: u32, gpu_temp_c: f32, zone: OperationalZone) -> u64 {
        let vram_part = (vram_mb as u64) & 0xFFFFF; // 20 bits
        let temp_part = (((gpu_temp_c * 2.0).clamp(0.0, 1023.0)) as u64) & 0x3FF; // 10 bits
        let zone_part = match zone {
            OperationalZone::Nominal => 0u64,
            OperationalZone::Preventive => 1u64,
            OperationalZone::Critical => 2u64,
        };

        vram_part | (temp_part << 20) | (zone_part << 30)
    }

    /// Retrieves the lock-free shared atomic state handle.
    pub fn shared_state(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.packed_state)
    }
}

impl Default for HardwareWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermodynamic_barrier_nominal() {
        // Free VRAM > 1400MB and Temp < 75°C -> Barrier is exactly 0.0
        let phi = compute_thermodynamic_barrier(68.0, 2_500, "Tier 1");
        assert_eq!(phi, 0.0);
    }

    #[test]
    fn test_thermodynamic_barrier_preventive() {
        // Free VRAM = 1000MB, Temp = 78°C -> In preventive zone
        let phi = compute_thermodynamic_barrier(78.0, 1_000, "Tier 1");
        assert!(phi > 0.0);
        assert!(!phi.is_infinite());

        // Exact calculation:
        // lambda = 2.5, vram_penalty = (1400 - 1000) / 600 = 400/600 = 0.6666...
        // mu = 1.8, temp_penalty = (78 - 75) / 7 = 3/7 = 0.42857...
        // expected = 2.5 * (400/600) + 1.8 * (3/7) = 1.6666 + 0.7714 = 2.438
        let expected = 2.5 * (400.0 / 600.0) + 1.8 * (3.0 / 7.0);
        assert!((phi - expected).abs() < 0.01);
    }

    #[test]
    fn test_thermodynamic_barrier_critical() {
        // Free VRAM <= 800MB -> Barrier is infinity
        let phi_vram = compute_thermodynamic_barrier(70.0, 800, "Tier 1");
        assert!(phi_vram.is_infinite());

        // Temp >= 82°C -> Barrier is infinity
        let phi_temp = compute_thermodynamic_barrier(82.5, 2_000, "Tier 1");
        assert!(phi_temp.is_infinite());
    }

    #[test]
    fn test_thermodynamic_barrier_non_gpu_tier() {
        // CPU tiers have zero barrier regardless of GPU state
        let phi = compute_thermodynamic_barrier(85.0, 400, "Tier 0");
        assert_eq!(phi, 0.0);

        let phi_cloud = compute_thermodynamic_barrier(88.0, 200, "Tier 3");
        assert_eq!(phi_cloud, 0.0);
    }

    #[test]
    fn test_ewma_vram_and_temperature_prediction() {
        let mut watchdog = HardwareWatchdog::new_with_initial_state(70.0, 2_000);

        // Simulate consecutive 1-second ticks of rising temperature and VRAM usage
        let s1 = watchdog.record_sample(70.0, 2_000, 6_144, 1.0);
        assert_eq!(s1.operational_zone, OperationalZone::Nominal);

        let s2 = watchdog.record_sample(73.0, 3_000, 6_144, 1.0);
        assert!(s2.temp_ewma_rate > 0.0);
        assert!(s2.vram_ewma_mb > 2_000.0);

        let s3 = watchdog.record_sample(76.0, 4_500, 6_144, 1.0);
        // Free VRAM = 6144 - 4500 = 1644, but temp is 76°C >= 75°C -> Preventive
        assert_eq!(s3.operational_zone, OperationalZone::Preventive);

        // Heavy sudden surge: temp reaches 82.5 and free VRAM drops to 644MB <= 800MB -> Critical
        let s4 = watchdog.record_sample(82.5, 5_500, 6_144, 1.0);
        // Free VRAM = 644MB <= 800MB -> Critical
        assert_eq!(s4.operational_zone, OperationalZone::Critical);
        assert!(s4.barrier_phi.is_infinite());
        assert!(!watchdog.is_gpu_safe());
    }

    #[test]
    fn test_fail_soft_when_nvml_unavailable() {
        let mut watchdog = HardwareWatchdog::new();
        // Force fail-soft
        watchdog.nvml_available = false;

        let snapshot = watchdog.sample_once();
        assert!(snapshot.vram_total_mb >= 5_000);
        assert!(snapshot.vram_free_mb > 0);
    }
}
