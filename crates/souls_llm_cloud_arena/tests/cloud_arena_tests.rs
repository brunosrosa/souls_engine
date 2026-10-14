//! # cloud_arena_tests.rs — Testes de Integração e Domínio da Cloud Arena

use souls_llm_cloud_arena::{
    analytics::{
        calculate_pareto_evaluations, compute_blended_cost, compute_composite_score,
        compute_roi_index, EvaluationCandidate,
    },
    canonical::CanonicalResolver,
    domain::{
        BenchmarkSnapshot, CanonicalModel, ParetoTag, PricingTelemetrySnapshot,
        TaskProfileKind,
    },
    report::generate_markdown_report,
    storage::{run_migrations, CloudArenaRepository},
    CloudArenaConfig, CloudArenaCoordinator,
};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn test_sqlite_strict_migrations_and_repository() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Falha ao abrir SQLite em memória");

    run_migrations(&pool)
        .await
        .expect("Falha ao executar migrations STRICT");

    let repo = CloudArenaRepository::new(pool);

    let model = CanonicalModel {
        id: "claude-3-7-sonnet".to_string(),
        display_name: "Claude 3.7 Sonnet".to_string(),
        family: "Claude".to_string(),
        developer: "Anthropic".to_string(),
        context_window: 200_000,
        supports_tools: true,
        supports_structured_outputs: true,
        supports_caching: true,
        is_multimodal: true,
        created_at: "2026-10-12T00:00:00Z".to_string(),
    };

    repo.upsert_canonical_model(&model)
        .await
        .expect("Falha ao inserir modelo canônico");

    repo.upsert_model_alias("anthropic/claude-3-7-sonnet", "claude-3-7-sonnet", "openrouter")
        .await
        .expect("Falha ao inserir alias");

    let pricing = PricingTelemetrySnapshot {
        canonical_id: "claude-3-7-sonnet".to_string(),
        source: "nous_portal".to_string(),
        prompt_price_per_m: 3.40,
        completion_price_per_m: 15.00,
        cache_read_price_per_m: 0.35,
        cache_write_price_per_m: 3.75,
        ttft_ms: Some(400.0),
        throughput_tps: Some(70.0),
        recorded_at: "2026-10-12".to_string(),
    };

    repo.record_pricing_snapshot(&pricing)
        .await
        .expect("Falha ao registrar snapshot de preço");

    let bench = BenchmarkSnapshot {
        canonical_id: "claude-3-7-sonnet".to_string(),
        general_elo: Some(1380.0),
        coding_elo: Some(1420.0),
        hard_prompts_elo: Some(1395.0),
        bfcl_score: Some(95.5),
        ifeval_score: Some(93.8),
        gpqa_score: Some(74.2),
        vqa_multimodal_score: Some(94.0),
        recorded_at: "2026-10-12".to_string(),
    };

    repo.record_benchmark_snapshot(&bench)
        .await
        .expect("Falha ao registrar snapshot de benchmark");

    let aliases = repo.get_model_aliases().await.expect("Falha ao ler aliases");
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].0, "anthropic/claude-3-7-sonnet");
}

#[test]
fn test_resolver_canonical_behavior() {
    let resolver = CanonicalResolver::new();
    assert_eq!(resolver.resolve("~anthropic/claude-sonnet-5.5"), "anthropic/claude-sonnet-5.5");
    assert_eq!(resolver.resolve("anthropic/claude-sonnet-5.5:beta"), "anthropic/claude-sonnet-5.5");
    assert_eq!(resolver.resolve("gemini-4-argon-high"), "google/gemini-4-argon");
    assert_eq!(resolver.resolve("~openai/gpt-astra-latest"), "openai/gpt-6-astra");
}

#[test]
fn test_scoring_and_blended_cost_across_5_profiles() {
    let bench = BenchmarkSnapshot {
        canonical_id: "claude-3-7-sonnet".to_string(),
        general_elo: Some(1380.0),
        coding_elo: Some(1420.0),
        hard_prompts_elo: Some(1395.0),
        bfcl_score: Some(95.5),
        ifeval_score: Some(93.8),
        gpqa_score: Some(74.2),
        vqa_multimodal_score: Some(94.0),
        recorded_at: "2026-10-12".to_string(),
    };

    let price = PricingTelemetrySnapshot {
        canonical_id: "claude-3-7-sonnet".to_string(),
        source: "nous_portal".to_string(),
        prompt_price_per_m: 3.40,
        completion_price_per_m: 15.00,
        cache_read_price_per_m: 0.35,
        cache_write_price_per_m: 3.75,
        ttft_ms: Some(400.0),
        throughput_tps: Some(72.0),
        recorded_at: "2026-10-12".to_string(),
    };

    for &profile in TaskProfileKind::all() {
        let score = compute_composite_score(&bench, &price, profile);
        let cost = compute_blended_cost(&price, profile);
        let roi = compute_roi_index(score, cost);

        assert!(score >= 60.0 && score <= 100.0, "Score fora da faixa: {}", score);
        assert!(cost > 0.5 && cost < 10.0, "Custo fora da faixa: {}", cost);
        assert!(roi > 20.0 && roi < 100.0, "ROI fora da faixa: {}", roi);
    }
}

#[test]
fn test_pareto_dominance_matrix() {
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
            roi_index: 75.0,
        },
        EvaluationCandidate {
            canonical_id: "qwen-2.5-coder-32b".to_string(),
            composite_score: 82.0,
            blended_cost: 0.18,
            roi_index: 70.0,
        },
        EvaluationCandidate {
            canonical_id: "outclassed-model".to_string(),
            composite_score: 80.0,
            blended_cost: 4.00,
            roi_index: 30.0,
        },
    ];

    let evals = calculate_pareto_evaluations(&candidates, TaskProfileKind::Coding, "2026-10-12");

    let claude = evals.iter().find(|e| e.canonical_id == "claude-3-7-sonnet").unwrap();
    assert!(claude.is_pareto_optimal);
    assert_eq!(claude.pareto_tag, ParetoTag::Leader);

    let deepseek = evals.iter().find(|e| e.canonical_id == "deepseek-v3").unwrap();
    assert!(deepseek.is_pareto_optimal);
    assert_eq!(deepseek.pareto_tag, ParetoTag::BestValue);

    let qwen = evals.iter().find(|e| e.canonical_id == "qwen-2.5-coder-32b").unwrap();
    assert!(qwen.is_pareto_optimal);
    assert_eq!(qwen.pareto_tag, ParetoTag::UltraLowCost);

    let outclassed = evals.iter().find(|e| e.canonical_id == "outclassed-model").unwrap();
    assert!(!outclassed.is_pareto_optimal);
    assert_eq!(outclassed.pareto_tag, ParetoTag::Dominated);
}

#[tokio::test]
async fn test_full_coordinator_pipeline_in_memory() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Falha ao abrir SQLite");

    let config = CloudArenaConfig {
        database_path: std::path::PathBuf::from(":memory:"),
        nous_api_key: None,
        openrouter_api_key: None,
        nous_base_url: "https://invalid-host-mock.internal/v1".to_string(),
        openrouter_base_url: "https://invalid-host-mock.internal/api/v1".to_string(),
        request_timeout_secs: 1,
    };

    let coordinator = CloudArenaCoordinator::from_pool(config, pool);

    let outcome = coordinator
        .force_sync(Some("2026-10-12"))
        .await
        .expect("Falha no pipeline de sincronização com fallback gracioso");

    assert!(outcome.models_synced >= 10);
    assert!(outcome.benchmark_snapshots >= 10);
    assert!(outcome.evaluations_calculated >= 25);

    let champions = coordinator
        .get_all_champions(Some("2026-10-12"))
        .await
        .expect("Falha ao obter campeões");

    assert_eq!(champions.len(), 5);

    let report = generate_markdown_report("2026-10-12", &champions);
    assert!(report.contains("Autonomous Coding & SWE"));
    assert!(report.contains("Design & Multimodal UI"));
    assert!(report.contains("Líder (Qualidade)"));
}

#[test]
fn test_foreign_key_protection_against_unknown_models() {
    let resolver = CanonicalResolver::new();

    // Modelos conhecidos resolvem perfeitamente
    assert_eq!(resolver.try_resolve("~anthropic/claude-sonnet-5.5"), Some("anthropic/claude-sonnet-5.5".to_string()));
    assert_eq!(resolver.try_resolve("gemini-4-argon-high"), Some("google/gemini-4-argon".to_string()));
    assert_eq!(resolver.try_resolve("~openai/gpt-astra-latest"), Some("openai/gpt-6-astra".to_string()));

    // Modelos desconhecidos fora da arena retornam None para blindar FK SQLite
    assert_eq!(resolver.try_resolve("random-provider/some-random-unknown-model"), None);
    assert_eq!(resolver.try_resolve("mistralai/mistral-large-2407"), None);
    assert_eq!(resolver.try_resolve("cohere/command-r-plus"), None);
}
