//! The 5 Canonical Benchmark Batteries defined by ADR-007.

pub mod type1_hardware;
pub mod type2_cognitive;
pub mod type3_vision;
pub mod type4_speculative;
pub mod type5_pressure;

pub use type1_hardware::run_hardware_profile;
pub use type2_cognitive::run_cognitive_eval;
pub use type3_vision::run_multimodal_vision;
pub use type4_speculative::run_speculative_draft;
pub use type5_pressure::run_context_pressure;

use serde::{Deserialize, Serialize};

/// The canonical benchmark battery selection.
///
/// Strictly decoupled from model tiers per ADR-007.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkType {
    /// Type 1: HardwareProfile (Physical speeds, VRAM, RAM, TTFT, TPS, KV Cache).
    Type1Hardware,
    /// Type 2: CognitiveEval (Rust AST validation, Structured JSON schemas, CoT efficiency).
    Type2Cognitive,
    /// Type 3: MultimodalVision (mmproj projector pairing and VQA prefill latency).
    Type3Vision,
    /// Type 4: SpeculativeDraft (Drafter vs Master alpha acceptance rate and speedup).
    Type4Speculative,
    /// Type 5: ContextPressure (Needle-in-a-Haystack probe across 4k to 32k contexts).
    Type5Pressure,
    /// Runs all applicable benchmark batteries in sequence.
    All,
}

impl BenchmarkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BenchmarkType::Type1Hardware => "Type 1: HardwareProfile",
            BenchmarkType::Type2Cognitive => "Type 2: CognitiveEval",
            BenchmarkType::Type3Vision => "Type 3: MultimodalVision",
            BenchmarkType::Type4Speculative => "Type 4: SpeculativeDraft",
            BenchmarkType::Type5Pressure => "Type 5: ContextPressure",
            BenchmarkType::All => "All Batteries",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "1" | "type1" | "hardware" | "profile" => Some(BenchmarkType::Type1Hardware),
            "2" | "type2" | "cognitive" | "eval" => Some(BenchmarkType::Type2Cognitive),
            "3" | "type3" | "vision" => Some(BenchmarkType::Type3Vision),
            "4" | "type4" | "speculative" | "draft" | "mtp" => Some(BenchmarkType::Type4Speculative),
            "5" | "type5" | "pressure" | "context" | "needle" => Some(BenchmarkType::Type5Pressure),
            "all" | "full" | "suite" => Some(BenchmarkType::All),
            _ => None,
        }
    }
}
