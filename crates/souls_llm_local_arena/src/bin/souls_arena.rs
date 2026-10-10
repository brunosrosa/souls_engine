//! # souls_arena (CLI Standalone Executable)
//!
//! Bare-metal profiling and evaluation CLI for local models on Windows 11.
//! Usage:
//!   souls_arena list [--tier <0|0.5|1|2|3|4>]
//!   souls_arena sync
//!   souls_arena run [--type <1|2|3|4|5|all>] [--model <name>] [--tier <tier>] [--engine <engine>] [--output <path>]

use std::env;
use std::process::ExitCode;
use souls_llm_local_arena::{
    default_model_directories, export_markdown_dossier, print_cognitive_eval_table,
    print_hardware_profile_table, print_model_list_table, print_pressure_table,
    print_speculative_table, print_vision_table, resolve_vault_db_path,
    scan_local_models, ArenaRunner, BenchmarkType, EngineType,
    ModelDescriptor, ModelTier, VaultManager,
};

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return ExitCode::SUCCESS;
    }

    match args[1].to_lowercase().as_str() {
        "list" | "ls" => handle_list(&args[2..]),
        "sync" => handle_sync().await,
        "run" | "bench" | "test" => handle_run(&args[2..]).await,
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("[!] Comando desconhecido: '{}'. Use 'souls_arena help'.", args[1]);
            ExitCode::from(1)
        }
    }
}

fn print_usage() {
    println!("\n=========================================================================================================");
    println!("  🏟️ SOULS ARENA — CLI STANDALONE DE BENCHMARK & SILÍCIO LOCAL (v7)");
    println!("=========================================================================================================");
    println!("  Uso:");
    println!("    souls_arena list [--tier <0|0.5|1|2|3|4>]");
    println!("        Lista todos os modelos detectados no disco (LMStudio e .souls_data).");
    println!();
    println!("    souls_arena sync");
    println!("        Sincroniza os modelos descobertos no SQLite 'souls_heuristic_vault.db'.");
    println!();
    println!("    souls_arena run [--type <1|2|3|4|5|all>] [--model <nome>] [--tier <0|0.5|1|2|3|4>] [--engine <engine>] [--output <caminho>]");
    println!("        Executa a bateria de benchmark selecionada.");
    println!();
    println!("  Parâmetros do 'run':");
    println!("    --type <1..5|all>  Tipo de bateria (1: HardwareProfile, 2: CognitiveEval, 3: Vision, 4: Speculative, 5: Pressure, all)");
    println!("    --model <filtro>   Nome ou substring do modelo a testar");
    println!("    --tier <0..4>      Filtra modelos pelo ModelTier");
    println!("    --engine <motor>   Motor: llama-cuda, llama-cpu, onnx-cpu, mock (default: mock/llama-cuda)");
    println!("    --output <arquivo> Caminho para exportar o dossiê Markdown em docs/telemetry/reports/");
    println!("=========================================================================================================\n");
}

fn handle_list(args: &[String]) -> ExitCode {
    let tier_filter = parse_flag_value(args, "--tier").and_then(|t| ModelTier::parse(&t));
    let dirs = default_model_directories();
    let mut models = scan_local_models(&dirs);

    if let Some(target_tier) = tier_filter {
        models.retain(|m| m.tier == target_tier);
    }

    print_model_list_table(&models);
    ExitCode::SUCCESS
}

async fn handle_sync() -> ExitCode {
    println!("[*] Iniciando varredura e sincronização com o Heuristic Vault...");
    let runner = ArenaRunner::new();
    match runner.sync_models_to_vault().await {
        Ok(count) => {
            println!("[✓] Sucesso: {} modelos sincronizados no banco SQLite.", count);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("[!] Erro ao sincronizar vault: {}", err);
            ExitCode::from(1)
        }
    }
}

async fn handle_run(args: &[String]) -> ExitCode {
    let bench_type_str = parse_flag_value(args, "--type").unwrap_or_else(|| "1".to_string());
    let bench_type = match BenchmarkType::parse(&bench_type_str) {
        Some(bt) => bt,
        None => {
            eprintln!("[!] Tipo de benchmark inválido: '{}'. Opções: 1, 2, 3, 4, 5, all.", bench_type_str);
            return ExitCode::from(1);
        }
    };

    let model_filter = parse_flag_value(args, "--model");
    let tier_filter = parse_flag_value(args, "--tier").and_then(|t| ModelTier::parse(&t));
    let engine_type = if let Some(e) = parse_flag_value(args, "--engine") {
        EngineType::parse(&e).unwrap_or(EngineType::LlamaCppCuda)
    } else {
        match tier_filter {
            Some(ModelTier::Tier0Bootstrap) | Some(ModelTier::Tier05Epistemic) => EngineType::LlamaCppCpu,
            _ => EngineType::LlamaCppCuda,
        }
    };
    let output_file = parse_flag_value(args, "--output");

    let dirs = default_model_directories();
    let all_models = scan_local_models(&dirs);
    let mut models = all_models.clone();

    // Apply filters
    if let Some(ref pattern) = model_filter {
        let pat_lower = pattern.to_lowercase();
        models.retain(|m| m.model_id.to_lowercase().contains(&pat_lower) || m.file_path.to_string_lossy().to_lowercase().contains(&pat_lower));
    }
    if let Some(target_tier) = tier_filter {
        models.retain(|m| m.tier == target_tier);
    }

    if models.is_empty() {
        println!("[!] Nenhum modelo encontrado com os filtros especificados. (Adicionando modelo sintético para demonstração)");
        models.push(ModelDescriptor::from_file("Qwen3.5-4B-Coder-Synthetic.gguf"));
    }

    println!("\n[*] Executando {} em {} modelo(s) usando motor '{}'...",
        bench_type.as_str(), models.len(), engine_type.as_str()
    );

    let vault_path = resolve_vault_db_path();
    let vault = match VaultManager::connect(&vault_path).await {
        Ok(v) => {
            println!("[*] Conectado ao Heuristic Vault SQLite em: {:?}", vault_path);
            Some(v)
        }
        Err(e) => {
            eprintln!("[!] AVISO: Falha ao conectar ao Heuristic Vault SQLite ({:?}): {}", vault_path, e);
            None
        }
    };

    let mut runner = ArenaRunner::new();
    let mut hw_results = Vec::new();
    let mut cog_results = Vec::new();
    let mut vision_results = Vec::new();
    let mut spec_results = Vec::new();
    let mut pressure_results = Vec::new();

    for m in &models {
        println!(" -> Processando: {} [{}]", m.model_id, m.tier.as_str());

        if bench_type == BenchmarkType::Type1Hardware || bench_type == BenchmarkType::All {
            match runner.run_hardware_profile(m, engine_type).await {
                Ok(res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_hardware_profile(&res).await {
                            eprintln!("    [!] Erro ao persistir telemetria no vault para {}: {}", m.model_id, ve);
                        } else {
                            println!("    [✓] Telemetria registrada com sucesso no vault.");
                        }
                    }
                    hw_results.push(res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 1 para {}: {}", m.model_id, e),
            }
        }

        if bench_type == BenchmarkType::Type2Cognitive || bench_type == BenchmarkType::All {
            match runner.run_cognitive_eval(m, engine_type, "ast").await {
                Ok(res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_cognitive_eval(&res).await {
                            eprintln!("    [!] Erro ao registrar Type 2 (AST) no vault: {}", ve);
                        }
                    }
                    cog_results.push(res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 2 (AST) para {}: {}", m.model_id, e),
            }
            match runner.run_cognitive_eval(m, engine_type, "json").await {
                Ok(res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_cognitive_eval(&res).await {
                            eprintln!("    [!] Erro ao registrar Type 2 (JSON) no vault: {}", ve);
                        }
                    }
                    cog_results.push(res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 2 (JSON) para {}: {}", m.model_id, e),
            }
            match runner.run_cognitive_eval(m, engine_type, "cot").await {
                Ok(res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_cognitive_eval(&res).await {
                            eprintln!("    [!] Erro ao registrar Type 2 (CoT) no vault: {}", ve);
                        }
                    }
                    cog_results.push(res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 2 (CoT) para {}: {}", m.model_id, e),
            }
            match runner.run_cognitive_eval(m, engine_type, "hard").await {
                Ok(res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_cognitive_eval(&res).await {
                            eprintln!("    [!] Erro ao registrar Type 2 (Hard Eval) no vault: {}", ve);
                        } else {
                            println!("    [✓] Telemetria Type 2 (HardEval Discriminatório: {:.1}%) registrada no vault.", res.accuracy_pct);
                        }
                    }
                    cog_results.push(res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 2 (Hard Eval) para {}: {}", m.model_id, e),
            }
        }

        if bench_type == BenchmarkType::Type3Vision || bench_type == BenchmarkType::All {
            match runner.run_multimodal_vision(m, engine_type).await {
                Ok(v_res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_multimodal_vision(&v_res).await {
                            eprintln!("    [!] Erro ao registrar Type 3 no vault: {}", ve);
                        } else {
                            println!("    [✓] Telemetria Type 3 (MultimodalVision) registrada no vault.");
                        }
                    }
                    vision_results.push(v_res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 3 para {}: {}", m.model_id, e),
            }
        }

        if bench_type == BenchmarkType::Type4Speculative || bench_type == BenchmarkType::All {
            let m_lower = m.model_id.to_lowercase();
            let drafter_cand = all_models.iter().find(|cand| {
                cand.tier == ModelTier::Tier4Drafter && (
                    (m_lower.contains("qwen") && cand.model_id.to_lowercase().contains("qwen")) ||
                    (m_lower.contains("laguna") && cand.model_id.to_lowercase().contains("laguna")) ||
                    (m_lower.contains("bonsai") && cand.model_id.to_lowercase().contains("bonsai")) ||
                    (m_lower.contains("llama") && cand.model_id.to_lowercase().contains("llama"))
                )
            });
            match runner.run_speculative_draft(m, engine_type, drafter_cand).await {
                Ok(s_res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_speculative_draft(&s_res).await {
                            eprintln!("    [!] Erro ao registrar Type 4 no vault: {}", ve);
                        } else {
                            println!("    [✓] Telemetria Type 4 (SpeculativeDraft) registrada no vault.");
                        }
                    }
                    spec_results.push(s_res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 4 para {}: {}", m.model_id, e),
            }
        }

        if bench_type == BenchmarkType::Type5Pressure || bench_type == BenchmarkType::All {
            match runner.run_context_pressure(m, engine_type).await {
                Ok(p_res) => {
                    if let Some(ref v) = vault {
                        if let Err(ve) = v.record_context_pressure(&p_res).await {
                            eprintln!("    [!] Erro ao registrar Type 5 no vault: {}", ve);
                        } else {
                            println!("    [✓] Telemetria Type 5 (ContextPressure) registrada com sucesso no vault.");
                        }
                    }
                    pressure_results.push(p_res);
                }
                Err(e) => eprintln!("    [!] Erro no Type 5 para {}: {}", m.model_id, e),
            }
        }
    }

    if !hw_results.is_empty() {
        print_hardware_profile_table(&hw_results);
    }
    if !cog_results.is_empty() {
        print_cognitive_eval_table(&cog_results);
    }
    if !vision_results.is_empty() {
        print_vision_table(&vision_results);
    }
    if !spec_results.is_empty() {
        print_speculative_table(&spec_results);
    }
    if !pressure_results.is_empty() {
        print_pressure_table(&pressure_results);
    }

    if let Some(dest) = output_file {
        match export_markdown_dossier(&dest, &hw_results, &cog_results) {
            Ok(_) => println!("[✓] Dossiê Markdown exportado com sucesso para: {}", dest),
            Err(e) => eprintln!("[!] Falha ao exportar dossiê Markdown: {}", e),
        }
    }

    if let Some(ref v) = vault {
        match v.recalculate_and_sync_task_profiles().await {
            Ok(n) => println!("[✓] {} perfis multi-dimensionais e Fronteira de Pareto recalculados e persistidos no vault SSOT.", n),
            Err(e) => eprintln!("[!] Falha ao recalcular perfis e Pareto no vault: {}", e),
        }
    }

    ExitCode::SUCCESS
}

fn parse_flag_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        if let Some(val) = args[i].strip_prefix(&format!("{}=", flag)) {
            return Some(val.to_string());
        }
    }
    None
}
