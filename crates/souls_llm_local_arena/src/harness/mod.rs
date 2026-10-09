//! Harness subroutines for execution containment and hardware surveillance.

pub mod nvml_guard;
pub mod worker;

pub use nvml_guard::{
    check_hardware_safeguards, get_current_process_ram_mb, SAFE_WDDM_VRAM_CEILING_MB,
    THERMAL_ABORT_THRESHOLD_C,
};
pub use worker::dispatch_safe_worker;
