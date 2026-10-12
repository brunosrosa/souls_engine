//! # analytics/mod.rs — Motor Analítico dos 5 Perfis e Fronteira de Pareto

pub mod pareto;
pub mod profiles;
pub mod scoring;

pub use pareto::{calculate_pareto_evaluations, EvaluationCandidate};
pub use profiles::{get_profile_weights, BenchmarkWeights, CostWeights};
pub use scoring::{compute_blended_cost, compute_composite_score, compute_roi_index};
