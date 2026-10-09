//! Markdown dossier exporter for arena benchmark runs.

use std::path::Path;
use crate::domain::result::{CognitiveEvalResult, HardwareProfileResult};
use crate::error::ArenaError;

/// Generates a comprehensive markdown report dossier from benchmark results.
pub fn export_markdown_dossier<P: AsRef<Path>>(
    destination: P,
    hardware_results: &[HardwareProfileResult],
    cognitive_results: &[CognitiveEvalResult],
) -> Result<(), ArenaError> {
    let p = destination.as_ref();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut md = String::new();
    let now = chrono_free_now();

    md.push_str("# 🏟️ SOULS ARENA — Relatório Executivo de Benchmarks de Silício Local\n\n");
    md.push_str(&format!("> **Data de Geração:** {}  \n", now));
    md.push_str("> **Hardware Host:** Intel Core i9-9880H (AVX2) | 32GB DDR4 | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)  \n");
    md.push_str("> **Governança:** ADR-004 (VRAM Ceiling 5.2GB) & ADR-007 (Desacoplamento Ontológico dos Benchmark Types)\n\n");

    if !hardware_results.is_empty() {
        md.push_str("## ⚡ Type 1: Hardware Profile (Desempenho Físico e Latência)\n\n");
        md.push_str("| Model ID | Tier | Engine | TTFT (ms) | TPOT (ms) | Sustained TPS | Peak VRAM (MB) | Process RAM (MB) | E³ Score | Status |\n");
        md.push_str("|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|\n");
        for r in hardware_results {
            md.push_str(&format!(
                "| `{}` | {} | `{}` | {:.1} | {:.1} | **{:.1}** | {:.1} | {:.1} | {:.2} | `{}` |\n",
                r.model_id, r.tier, r.engine, r.ttft_ms, r.tpot_ms, r.tps, r.peak_vram_mb, r.ram_mb, r.e3_score, r.status
            ));
        }
        md.push_str("\n");
    }

    if !cognitive_results.is_empty() {
        md.push_str("## 🧠 Type 2: Avaliação Cognitiva E³ (Rust AST, JSON e CoT)\n\n");
        md.push_str("| Model ID | Trilha | Acurácia (%) | AST Válido | Think Tokens | CoT Ratio | E³ Score | Veredito |\n");
        md.push_str("|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|\n");
        for r in cognitive_results {
            let ast_str = if r.ast_valid { "✅ Sim" } else { "❌ Não" };
            md.push_str(&format!(
                "| `{}` | {} | {:.1}% | {} | {} | {:.3} | {:.2} | `{}` |\n",
                r.model_id, r.track, r.accuracy_pct, ast_str, r.think_tokens_avg, r.cot_efficiency_ratio, r.e3_score, r.status
            ));
        }
        md.push_str("\n");
    }

    md.push_str("---\n*Gerado automaticamente pela CLI `souls_arena` (Souls Engine v7).*\n");

    std::fs::write(p, md)?;
    Ok(())
}

fn chrono_free_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("Unix Epoch {}", now)
}
