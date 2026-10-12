//! # scoring.rs — Normalização e Cálculo de Métricas Compostas
//!
//! Calcula S_task (nota cognitiva), C_task (custo ponderado) e V_task (ROI amortecido).

use crate::analytics::profiles::get_profile_weights;
use crate::domain::{BenchmarkSnapshot, PricingTelemetrySnapshot, TaskProfileKind};

pub fn normalize_val(val: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return 50.0;
    }
    let clamped = val.clamp(min, max);
    ((clamped - min) / (max - min)) * 100.0
}

pub fn normalize_ttft(ttft_ms: f64) -> f64 {
    let min = 150.0;
    let max = 2000.0;
    let clamped = ttft_ms.clamp(min, max);
    ((max - clamped) / (max - min)) * 100.0
}

/// Calcula a nota cognitiva composta S_task para um perfil específico (0.0 a 100.0).
pub fn compute_composite_score(
    bench: &BenchmarkSnapshot,
    price: &PricingTelemetrySnapshot,
    profile: TaskProfileKind,
) -> f64 {
    let (bw, _) = get_profile_weights(profile);

    let coding_norm = normalize_val(bench.coding_elo.unwrap_or(1200.0), 1000.0, 1450.0);
    let hard_norm = normalize_val(bench.hard_prompts_elo.unwrap_or(1200.0), 1000.0, 1450.0);
    let bfcl_score = bench.bfcl_score.unwrap_or(75.0).clamp(0.0, 100.0);
    let ifeval_score = bench.ifeval_score.unwrap_or(75.0).clamp(0.0, 100.0);
    let gpqa_score = bench.gpqa_score.unwrap_or(50.0).clamp(0.0, 100.0);
    let vqa_score = bench.vqa_multimodal_score.unwrap_or(60.0).clamp(0.0, 100.0);

    let tps_norm = normalize_val(price.throughput_tps.unwrap_or(60.0), 10.0, 150.0);
    let ttft_norm = normalize_ttft(price.ttft_ms.unwrap_or(500.0));

    let score = bw.coding_elo * coding_norm
        + bw.hard_elo * hard_norm
        + bw.bfcl * bfcl_score
        + bw.ifeval * ifeval_score
        + bw.gpqa * gpqa_score
        + bw.vqa * vqa_score
        + bw.tps * tps_norm
        + bw.ttft * ttft_norm;

    score.clamp(0.0, 100.0)
}

/// Calcula o custo ponderado por 1M tokens C_task para um perfil específico.
pub fn compute_blended_cost(
    price: &PricingTelemetrySnapshot,
    profile: TaskProfileKind,
) -> f64 {
    let (_, cw) = get_profile_weights(profile);

    let cost = (cw.w_cache * price.cache_read_price_per_m)
        + (cw.w_in * price.prompt_price_per_m)
        + (cw.w_out * price.completion_price_per_m);

    cost.max(0.001)
}

/// Calcula o ROI Amortecido V_task = S_task / (1.0 + ln(1.0 + C_task)).
pub fn compute_roi_index(score: f64, cost: f64) -> f64 {
    let denom = 1.0 + (1.0 + cost).ln();
    score / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        assert_eq!(normalize_val(1000.0, 1000.0, 1450.0), 0.0);
        assert_eq!(normalize_val(1450.0, 1000.0, 1450.0), 100.0);
        assert_eq!(normalize_val(1225.0, 1000.0, 1450.0), 50.0);
    }

    #[test]
    fn test_roi_index_damped() {
        let cheap_roi = compute_roi_index(80.0, 0.20);
        let exp_roi = compute_roi_index(95.0, 3.40);
        // Ambos devem produzir índices bem balanceados
        assert!(cheap_roi > 60.0);
        assert!(exp_roi > 35.0);
    }
}
