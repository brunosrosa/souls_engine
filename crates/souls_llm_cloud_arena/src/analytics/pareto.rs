//! # pareto.rs — Filtro da Fronteira de Pareto e Atribuição de Tags
//!
//! Algoritmo de dominância multiobjetivo para identificar modelos ótimos
//! e classificar Leader, BestValue, UltraLowCost e Dominated.

use crate::domain::{DailyEvaluation, ParetoTag, TaskProfileKind};

pub struct EvaluationCandidate {
    pub canonical_id: String,
    pub composite_score: f64,
    pub blended_cost: f64,
    pub roi_index: f64,
}

pub fn calculate_pareto_evaluations(
    candidates: &[EvaluationCandidate],
    profile: TaskProfileKind,
    date: &str,
) -> Vec<DailyEvaluation> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let n = candidates.len();
    let mut is_dominated = vec![false; n];

    // Algoritmo clássico de dominância de Pareto
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            // j domina i se S(j) >= S(i) e C(j) <= C(i) com pelo menos uma desigualdade estrita
            let score_better_or_equal = candidates[j].composite_score >= candidates[i].composite_score;
            let cost_better_or_equal = candidates[j].blended_cost <= candidates[i].blended_cost;
            let strictly_better = (candidates[j].composite_score > candidates[i].composite_score)
                || (candidates[j].blended_cost < candidates[i].blended_cost);

            if score_better_or_equal && cost_better_or_equal && strictly_better {
                is_dominated[i] = true;
                break;
            }
        }
    }

    // Identifica os campeões da fronteira
    let mut best_score_idx: Option<usize> = None;
    let mut best_roi_idx: Option<usize> = None;
    let mut lowest_cost_idx: Option<usize> = None;

    for (i, c) in candidates.iter().enumerate() {
        if !is_dominated[i] {
            // Leader: maior pontuação
            if best_score_idx.is_none()
                || c.composite_score > candidates[best_score_idx.unwrap()].composite_score
            {
                best_score_idx = Some(i);
            }

            // BestValue: maior ROI amortecido
            if best_roi_idx.is_none()
                || c.roi_index > candidates[best_roi_idx.unwrap()].roi_index
            {
                best_roi_idx = Some(i);
            }

            // UltraLowCost: menor custo com nota mínima respeitável (>= 70)
            if c.composite_score >= 70.0 {
                if lowest_cost_idx.is_none()
                    || c.blended_cost < candidates[lowest_cost_idx.unwrap()].blended_cost
                {
                    lowest_cost_idx = Some(i);
                }
            }
        }
    }

    let mut results = Vec::with_capacity(n);

    for (i, c) in candidates.iter().enumerate() {
        let is_optimal = !is_dominated[i];
        let tag = if !is_optimal {
            ParetoTag::Dominated
        } else if Some(i) == best_score_idx {
            ParetoTag::Leader
        } else if Some(i) == best_roi_idx {
            ParetoTag::BestValue
        } else if Some(i) == lowest_cost_idx {
            ParetoTag::UltraLowCost
        } else {
            ParetoTag::BestValue
        };

        results.push(DailyEvaluation {
            canonical_id: c.canonical_id.clone(),
            evaluation_date: date.to_string(),
            profile_task: profile,
            composite_score: c.composite_score,
            task_blended_cost: c.blended_cost,
            roi_index: c.roi_index,
            is_pareto_optimal: is_optimal,
            pareto_tag: tag,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pareto_frontier() {
        let candidates = vec![
            EvaluationCandidate {
                canonical_id: "claude-3-7-sonnet".to_string(),
                composite_score: 95.0,
                blended_cost: 3.40,
                roi_index: 38.0,
            },
            EvaluationCandidate {
                canonical_id: "deepseek-v3".to_string(),
                composite_score: 88.0,
                blended_cost: 0.28,
                roi_index: 70.0,
            },
            EvaluationCandidate {
                canonical_id: "inferior-model".to_string(),
                composite_score: 85.0,
                blended_cost: 3.80, // Pior em nota e mais caro que deepseek-v3 e claude
                roi_index: 33.0,
            },
        ];

        let evals = calculate_pareto_evaluations(&candidates, TaskProfileKind::Coding, "2026-10-12");

        let claude = evals.iter().find(|e| e.canonical_id == "claude-3-7-sonnet").unwrap();
        assert!(claude.is_pareto_optimal);
        assert_eq!(claude.pareto_tag, ParetoTag::Leader);

        let deepseek = evals.iter().find(|e| e.canonical_id == "deepseek-v3").unwrap();
        assert!(deepseek.is_pareto_optimal);
        assert_eq!(deepseek.pareto_tag, ParetoTag::BestValue);

        let inferior = evals.iter().find(|e| e.canonical_id == "inferior-model").unwrap();
        assert!(!inferior.is_pareto_optimal);
        assert_eq!(inferior.pareto_tag, ParetoTag::Dominated);
    }
}
