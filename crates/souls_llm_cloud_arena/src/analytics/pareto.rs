//! # pareto.rs — Filtro da Fronteira de Pareto e Atribuição de Tags
//!
//! Algoritmo de dominância multiobjetivo para identificar modelos ótimos
//! e classificar Leader, BestValue, UltraLowCost, Dominated e Estimated.

use crate::domain::{DailyEvaluation, ParetoTag, TaskProfileKind};

pub struct EvaluationCandidate {
    pub canonical_id: String,
    pub composite_score: f64,
    pub blended_cost: f64,
    pub roi_index: f64,
    pub is_verified: bool,
    pub is_eligible: bool,
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

    // Dominância de Pareto ocorre exclusivamente entre candidatos verificados e tecnicamente elegíveis
    for i in 0..n {
        if !candidates[i].is_verified || !candidates[i].is_eligible {
            is_dominated[i] = true;
            continue;
        }
        for j in 0..n {
            if i == j || !candidates[j].is_verified || !candidates[j].is_eligible {
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

    // 1. Leader: candidato verificado e elegível com maior pontuação composta
    // Para o perfil Executive (Master Brain), exige piso de fronteira (>= 85.0).
    let mut best_score_idx: Option<usize> = None;
    for (i, c) in candidates.iter().enumerate() {
        if !c.is_verified || !c.is_eligible || is_dominated[i] {
            continue;
        }
        if profile == TaskProfileKind::Executive && c.composite_score < 85.0 {
            continue;
        }
        let better = match best_score_idx {
            None => true,
            Some(best) => {
                c.composite_score > candidates[best].composite_score
                    || (c.composite_score == candidates[best].composite_score
                        && c.blended_cost < candidates[best].blended_cost)
            }
        };
        if better {
            best_score_idx = Some(i);
        }
    }

    let leader_score = best_score_idx.map(|idx| candidates[idx].composite_score).unwrap_or(0.0);
    let leader_cost = best_score_idx.map(|idx| candidates[idx].blended_cost).unwrap_or(f64::INFINITY);

    // 2. BestValue: melhor ROI na faixa de Alta Capacidade (>= 82.0, dentro de 12 pts do líder, mais barato que o líder)
    let mut best_roi_idx: Option<usize> = None;
    for (i, c) in candidates.iter().enumerate() {
        if !c.is_verified || !c.is_eligible || is_dominated[i] || Some(i) == best_score_idx {
            continue;
        }
        if c.composite_score >= 82.0
            && c.composite_score >= (leader_score - 12.0)
            && c.blended_cost < leader_cost
        {
            let better = match best_roi_idx {
                None => true,
                Some(best) => {
                    c.roi_index > candidates[best].roi_index
                        || (c.roi_index == candidates[best].roi_index
                            && c.composite_score > candidates[best].composite_score)
                }
            };
            if better {
                best_roi_idx = Some(i);
            }
        }
    }

    // Fallback 1 de BestValue: não-dominado distinto com score >= 78.0 ponderando score e ROI
    if best_roi_idx.is_none() {
        for (i, c) in candidates.iter().enumerate() {
            if !c.is_verified || !c.is_eligible || is_dominated[i] || Some(i) == best_score_idx {
                continue;
            }
            if c.composite_score >= 78.0 {
                let weighted = c.composite_score * 0.6 + c.roi_index * 0.4;
                let better = match best_roi_idx {
                    None => true,
                    Some(best) => {
                        let prev = candidates[best].composite_score * 0.6 + candidates[best].roi_index * 0.4;
                        weighted > prev
                    }
                };
                if better {
                    best_roi_idx = Some(i);
                }
            }
        }
    }

    // Fallback 2 de BestValue: verificado e elegível distinto na faixa >= 78.0 (mesmo se levemente dominado por um Leader muito barato)
    if best_roi_idx.is_none() {
        for (i, c) in candidates.iter().enumerate() {
            if !c.is_verified || !c.is_eligible || Some(i) == best_score_idx {
                continue;
            }
            if c.composite_score >= 78.0 && c.blended_cost < leader_cost {
                let weighted = c.composite_score * 0.6 + c.roi_index * 0.4;
                let better = match best_roi_idx {
                    None => true,
                    Some(best) => {
                        let prev = candidates[best].composite_score * 0.6 + candidates[best].roi_index * 0.4;
                        weighted > prev
                    }
                };
                if better {
                    best_roi_idx = Some(i);
                }
            }
        }
    }

    // 3. UltraLowCost: menor custo com nota respeitável (>= 72.0) e tarifa <= $0.60/1M (distinto dos 2 primeiros)
    let mut lowest_cost_idx: Option<usize> = None;
    for (i, c) in candidates.iter().enumerate() {
        if !c.is_verified
            || !c.is_eligible
            || is_dominated[i]
            || Some(i) == best_score_idx
            || Some(i) == best_roi_idx
        {
            continue;
        }
        if c.composite_score >= 72.0 && c.blended_cost <= 0.60 {
            let better = match lowest_cost_idx {
                None => true,
                Some(best) => {
                    c.blended_cost < candidates[best].blended_cost
                        || (c.blended_cost == candidates[best].blended_cost
                            && c.roi_index > candidates[best].roi_index)
                }
            };
            if better {
                lowest_cost_idx = Some(i);
            }
        }
    }

    // Fallback de UltraLowCost: se nenhum <= $0.60 for não-dominado, aceita qualquer verificado elegível <= $0.60 com score >= 72.0
    if lowest_cost_idx.is_none() {
        for (i, c) in candidates.iter().enumerate() {
            if !c.is_verified
                || !c.is_eligible
                || Some(i) == best_score_idx
                || Some(i) == best_roi_idx
            {
                continue;
            }
            if c.composite_score >= 72.0 && c.blended_cost <= 0.60 {
                let better = match lowest_cost_idx {
                    None => true,
                    Some(best) => {
                        c.blended_cost < candidates[best].blended_cost
                            || (c.blended_cost == candidates[best].blended_cost
                                && c.roi_index > candidates[best].roi_index)
                    }
                };
                if better {
                    lowest_cost_idx = Some(i);
                }
            }
        }
    }

    // Fallback final de UltraLowCost: menor custo não-dominado distinto com score >= 72.0
    if lowest_cost_idx.is_none() {
        for (i, c) in candidates.iter().enumerate() {
            if !c.is_verified
                || !c.is_eligible
                || is_dominated[i]
                || Some(i) == best_score_idx
                || Some(i) == best_roi_idx
            {
                continue;
            }
            if c.composite_score >= 72.0 {
                let better = match lowest_cost_idx {
                    None => true,
                    Some(best) => {
                        c.blended_cost < candidates[best].blended_cost
                            || (c.blended_cost == candidates[best].blended_cost
                                && c.roi_index > candidates[best].roi_index)
                    }
                };
                if better {
                    lowest_cost_idx = Some(i);
                }
            }
        }
    }

    let mut results = Vec::with_capacity(n);

    for (i, c) in candidates.iter().enumerate() {
        let is_optimal = c.is_verified && c.is_eligible && !is_dominated[i];
        let tag = if !c.is_verified {
            ParetoTag::Estimated
        } else if Some(i) == best_score_idx {
            ParetoTag::Leader
        } else if Some(i) == best_roi_idx {
            ParetoTag::BestValue
        } else if Some(i) == lowest_cost_idx {
            ParetoTag::UltraLowCost
        } else {
            ParetoTag::Dominated
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
                is_verified: true,
                is_eligible: true,
            },
            EvaluationCandidate {
                canonical_id: "deepseek-v3".to_string(),
                composite_score: 88.0,
                blended_cost: 0.28,
                roi_index: 70.0,
                is_verified: true,
                is_eligible: true,
            },
            EvaluationCandidate {
                canonical_id: "qwen-flash".to_string(),
                composite_score: 76.0,
                blended_cost: 0.04,
                roi_index: 73.0, // ROI maior que deepseek-v3, mas fica em UltraLowCost pois score < 82
                is_verified: true,
                is_eligible: true,
            },
            EvaluationCandidate {
                canonical_id: "unverified-cheap".to_string(),
                composite_score: 78.0,
                blended_cost: 0.01,
                roi_index: 77.0,
                is_verified: false,
                is_eligible: true,
            },
            EvaluationCandidate {
                canonical_id: "inferior-model".to_string(),
                composite_score: 85.0,
                blended_cost: 3.80, // Pior em nota e mais caro que deepseek-v3 e claude
                roi_index: 33.0,
                is_verified: true,
                is_eligible: true,
            },
        ];

        let evals = calculate_pareto_evaluations(&candidates, TaskProfileKind::Coding, "2026-10-12");

        let claude = evals.iter().find(|e| e.canonical_id == "claude-3-7-sonnet").unwrap();
        assert!(claude.is_pareto_optimal);
        assert_eq!(claude.pareto_tag, ParetoTag::Leader);

        let deepseek = evals.iter().find(|e| e.canonical_id == "deepseek-v3").unwrap();
        assert!(deepseek.is_pareto_optimal);
        assert_eq!(deepseek.pareto_tag, ParetoTag::BestValue);

        let flash = evals.iter().find(|e| e.canonical_id == "qwen-flash").unwrap();
        assert!(flash.is_pareto_optimal);
        assert_eq!(flash.pareto_tag, ParetoTag::UltraLowCost);

        let unverified = evals.iter().find(|e| e.canonical_id == "unverified-cheap").unwrap();
        assert!(!unverified.is_pareto_optimal);
        assert_eq!(unverified.pareto_tag, ParetoTag::Estimated);

        let inferior = evals.iter().find(|e| e.canonical_id == "inferior-model").unwrap();
        assert!(!inferior.is_pareto_optimal);
        assert_eq!(inferior.pareto_tag, ParetoTag::Dominated);
    }
}
