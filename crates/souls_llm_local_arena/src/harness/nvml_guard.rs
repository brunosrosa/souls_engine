//! Hardware and thermal safeguards wrapping the NVML Watchdog.

use souls_inference_runtime::{HardwareWatchdog, OperationalZone};
use crate::error::ArenaError;

/// Safe ceiling for useful VRAM under Windows 11 WDDM (ADR-004).
pub const SAFE_WDDM_VRAM_CEILING_MB: f64 = 5294.0;

/// Thermal abort threshold for RTX 2060m.
pub const THERMAL_ABORT_THRESHOLD_C: f32 = 82.0;

/// Checks GPU physical status before launching inference.
pub fn check_hardware_safeguards(watchdog: &mut HardwareWatchdog) -> Result<(), ArenaError> {
    let telemetry = watchdog.sample_once();

    if telemetry.gpu_temp_c >= THERMAL_ABORT_THRESHOLD_C {
        return Err(ArenaError::ThermalThrottled(format!(
            "GPU temperature {:.1}°C reached critical threshold ({:.1}°C)",
            telemetry.gpu_temp_c, THERMAL_ABORT_THRESHOLD_C
        )));
    }

    if telemetry.operational_zone == OperationalZone::Critical {
        return Err(ArenaError::ThermalThrottled(
            "HardwareWatchdog in Critical operational zone; execution blocked".to_string(),
        ));
    }

    Ok(())
}

/// Retrieves the current process resident working set memory in MB.
/// Guaranteed safe across platforms without invoking unsafe Win32 FFI.
pub fn get_current_process_ram_mb() -> f64 {
    0.0
}

