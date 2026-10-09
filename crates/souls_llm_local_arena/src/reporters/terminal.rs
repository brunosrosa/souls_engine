//! Terminal ASCII table formatters for the Arena CLI on Windows.

use crate::domain::model::ModelDescriptor;
use crate::domain::result::{
    CognitiveEvalResult, ContextPressureResult, HardwareProfileResult, MultimodalVisionResult,
    SpeculativeResult,
};

pub fn print_model_list_table(models: &[ModelDescriptor]) {
    if models.is_empty() {
        println!("\n[!] Nenhum modelo local encontrado nos diretórios padrão.");
        return;
    }

    println!("\n==========================================================================================================================================");
    println!("  SOULS ARENA — MODELOS LOCAIS DESCOBERTOS (TOTAL: {})", models.len());
    println!("==========================================================================================================================================");
    println!("  {:<10} | {:<42} | {:<9} | {:<10} | {:>8} | {:>7} | {:<8} | Path",
        "Tier", "Model ID", "Params", "Quant", "Size(MB)", "Ctx(k)", "mmproj"
    );
    println!("  -----------+--------------------------------------------+-----------+------------+----------+---------+----------+-----------------------------");

    for m in models {
        let name = if m.model_id.len() > 42 {
            format!("{}...", &m.model_id[..39])
        } else {
            m.model_id.clone()
        };
        let mmproj_str = if m.has_mmproj { "  SIM   " } else { "   -    " };
        let path_str = if m.file_path.to_string_lossy().len() > 38 {
            format!("...{}", &m.file_path.to_string_lossy()[m.file_path.to_string_lossy().len() - 35..])
        } else {
            m.file_path.to_string_lossy().to_string()
        };

        println!(
            "  {:<10} | {:<42} | {:<9} | {:<10} | {:>8.1} | {:>7} | {:<8} | {}",
            m.tier.as_str(),
            name,
            m.parameters,
            m.quant,
            m.size_mb,
            m.ctx_max / 1024,
            mmproj_str,
            path_str
        );
    }
    println!("==========================================================================================================================================\n");
}

pub fn print_hardware_profile_table(results: &[HardwareProfileResult]) {
    if results.is_empty() {
        return;
    }

    println!("\n===============================================================================================================================================================");
    println!("  SOULS ARENA — TYPE 1: HARDWARE PROFILE (PHYSICAL SPEEDS, VRAM & SILICON LATENCY)");
    println!("===============================================================================================================================================================");
    println!("  {:<10} | {:<38} | {:<11} | {:>8} | {:>8} | {:>8} | {:>9} | {:>8} | {:>8} | {:<10}",
        "Tier", "Model ID", "Engine", "TTFT(ms)", "TPOT(ms)", "TPS", "VRAM(MB)", "RAM(MB)", "E³ Score", "Status"
    );
    println!("  -----------+----------------------------------------+-------------+----------+----------+----------+-----------+----------+----------+-----------");

    for r in results {
        let name = if r.model_id.len() > 38 {
            format!("{}...", &r.model_id[..35])
        } else {
            r.model_id.clone()
        };

        println!(
            "  {:<10} | {:<38} | {:<11} | {:>8.1} | {:>8.1} | {:>8.1} | {:>9.1} | {:>8.1} | {:>8.2} | {:<10}",
            r.tier,
            name,
            r.engine,
            r.ttft_ms,
            r.tpot_ms,
            r.tps,
            r.peak_vram_mb,
            r.ram_mb,
            r.e3_score,
            r.status
        );
    }
    println!("===============================================================================================================================================================\n");
}

pub fn print_cognitive_eval_table(results: &[CognitiveEvalResult]) {
    if results.is_empty() {
        return;
    }

    println!("\n==========================================================================================================================================");
    println!("  SOULS ARENA — TYPE 2: COGNITIVE EVALUATION (RUST AST, JSON SCHEMA & CoT EFFICIENCY)");
    println!("==========================================================================================================================================");
    println!("  {:<38} | {:<22} | {:>8} | {:^9} | {:>9} | {:>10} | {:>8} | {:<12}",
        "Model ID", "Track", "Acc(%)", "AST Valid", "ThinkTok", "CoT Ratio", "E³ Score", "Status"
    );
    println!("  ---------------------------------------+------------------------+----------+-----------+-----------+------------+----------+-------------");

    for r in results {
        let name = if r.model_id.len() > 38 {
            format!("{}...", &r.model_id[..35])
        } else {
            r.model_id.clone()
        };
        let ast_mark = if r.ast_valid { "   SIM   " } else { "   NÃO   " };

        println!(
            "  {:<38} | {:<22} | {:>7.1}% | {} | {:>9} | {:>10.3} | {:>8.2} | {:<12}",
            name,
            r.track,
            r.accuracy_pct,
            ast_mark,
            r.think_tokens_avg,
            r.cot_efficiency_ratio,
            r.e3_score,
            r.status
        );
    }
    println!("==========================================================================================================================================\n");
}

pub fn print_vision_table(results: &[MultimodalVisionResult]) {
    if results.is_empty() {
        return;
    }
    println!("\n=========================================================================================================");
    println!("  SOULS ARENA — TYPE 3: MULTIMODAL VISION (mmproj PROJECTOR PAIRING)");
    println!("=========================================================================================================");
    for r in results {
        println!("  Model: {:<35} | Sidecar: {:<25} | Latency: {:>6.1}ms | Status: {}",
            r.model_id, r.sidecar_path, r.projection_latency_ms, r.status
        );
    }
    println!("=========================================================================================================\n");
}

pub fn print_speculative_table(results: &[SpeculativeResult]) {
    if results.is_empty() {
        return;
    }
    println!("\n=========================================================================================================");
    println!("  SOULS ARENA — TYPE 4: SPECULATIVE DRAFT (ALPHA RATE & REAL SPEEDUP)");
    println!("=========================================================================================================");
    for r in results {
        println!("  Target: {:<30} | Drafter: {:<20} | Alpha: {:>5.1}% | Speedup: {:>5.2}x | Verdict: {}",
            r.model_id, r.drafter_id, r.acceptance_rate_alpha * 100.0, r.speedup_ratio, r.verdict
        );
    }
    println!("=========================================================================================================\n");
}

pub fn print_pressure_table(results: &[ContextPressureResult]) {
    if results.is_empty() {
        return;
    }
    println!("\n=========================================================================================================");
    println!("  SOULS ARENA — TYPE 5: CONTEXT PRESSURE (NEEDLE-IN-A-HAYSTACK PROBE)");
    println!("=========================================================================================================");
    for r in results {
        println!("  Model: {:<35} | Tested: {:>5}k | Effective: {:>5}k | Status: {}",
            r.model_id, r.max_tested_context / 1024, r.max_effective_context / 1024, r.status
        );
    }
    println!("=========================================================================================================\n");
}
