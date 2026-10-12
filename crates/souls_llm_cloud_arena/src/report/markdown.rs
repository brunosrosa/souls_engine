//! # markdown.rs — Gerador de Relatório Diário em Markdown
//!
//! Formata tabelas e recomendações executivas para os agentes e documentação.

use crate::domain::{ProfileChampionReport, TaskProfileKind};

pub fn generate_markdown_report(
    date: &str,
    champions: &[ProfileChampionReport],
) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Relatório de Custo-Benefício de LLMs em Nuvem — {}\n\n", date));
    out.push_str("Este relatório consolida os preços vigentes (Nous Portal e OpenRouter) e benchmarks cognitivos sob os 5 perfis ponderados do Souls Engine v7.\n\n");

    out.push_str("## 1. Campeões da Fronteira de Pareto por Perfil de Carga\n\n");

    for champ in champions {
        let profile_name = match champ.profile {
            TaskProfileKind::Coding => "Autonomous Coding & SWE",
            TaskProfileKind::Agentic => "Autonomous Agentic & Tool Calling",
            TaskProfileKind::Reasoning => "Deep Cognitive Reasoning",
            TaskProfileKind::Fast => "Fast Heuristic Triage",
            TaskProfileKind::Design => "Design & Multimodal UI",
        };

        out.push_str(&format!("### Perfil: {}\n\n", profile_name));
        out.push_str("| Categoria | Modelo Canônico | Provedor | Score (0-100) | Custo Blended (/1M) | Throughput | ROI Index |\n");
        out.push_str("|---|---|---|---|---|---|---|\n");

        if let Some(ref leader) = champ.quality_leader {
            out.push_str(&format!(
                "| **Líder (Qualidade)** | **{}** | {} | {:.1} | ${:.2} | {:.0} t/s | {:.1} |\n",
                leader.display_name,
                leader.provider,
                leader.composite_score,
                leader.blended_cost,
                leader.throughput_tps.unwrap_or(0.0),
                leader.roi_index
            ));
        }

        if let Some(ref best) = champ.best_value {
            out.push_str(&format!(
                "| **Custo x Benefício** | **{}** | {} | {:.1} | ${:.2} | {:.0} t/s | {:.1} |\n",
                best.display_name,
                best.provider,
                best.composite_score,
                best.blended_cost,
                best.throughput_tps.unwrap_or(0.0),
                best.roi_index
            ));
        }

        if let Some(ref ultra) = champ.ultra_low_cost {
            out.push_str(&format!(
                "| **Ultra Low-Cost** | **{}** | {} | {:.1} | ${:.2} | {:.0} t/s | {:.1} |\n",
                ultra.display_name,
                ultra.provider,
                ultra.composite_score,
                ultra.blended_cost,
                ultra.throughput_tps.unwrap_or(0.0),
                ultra.roi_index
            ));
        }

        out.push('\n');
    }

    out.push_str("## 2. Diretrizes de Roteamento para Subagentes (Hermes Agent)\n\n");
    out.push_str("* **Tarefas de Engenharia Crítica / AST:** Roteie primariamente para o Líder de Coding via Nous Portal para economia máxima em cache read.\n");
    out.push_str("* **Subagentes de Alta Cadência / Triagem:** Roteie para o campeão de Custo x Benefício ou Ultra Low-Cost, minimizando despesas operacionais.\n");
    out.push_str("* **Tarefas de Raciocínio Profundo:** Roteie para os modelos especializados em Chain-of-Thought (GPQA líder).\n\n");

    out
}
