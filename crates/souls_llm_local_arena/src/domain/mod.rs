//! Domain entities and contracts for the Arena.

pub mod engine;
pub mod model;
pub mod result;

pub use engine::{ArenaInferEngine, EngineType, InferenceRunParams, RawInferenceOutput};
pub use model::{extract_parameters, extract_quant, ModelDescriptor, ModelTier};
pub use result::{
    CognitiveEvalResult, ContextPressureResult, HardwareProfileResult, MultimodalVisionResult,
    SpeculativeResult,
};
