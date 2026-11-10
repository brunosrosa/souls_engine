//! Comprehensive Integration Tests for Dynamic Provider Quarantine (`souls_model_router`).
//!
//! Validates:
//! - Header-driven dynamic quarantine calculation (seconds, HTTP-date RFC 2822, x-ratelimit-reset).
//! - Circuit Breaker cooldown application and health-first fallback candidate ordering.
//! - Bayesian priors ($\alpha, \beta$) preservation under transient HTTP 429 rate limits.

use std::sync::Arc;
use souls_memory::sqlite::create_in_memory_sqlite_pool;
use souls_model_router::{
    extract_cooldown_from_headers, fetch_prior, upsert_model, IncidentKind, ModelRecord,
    ParetoBanditRouter, TaskContext, TaskOutcome,
};

#[test]
fn test_header_quarantine_extraction_matrix() {
    let now = 1_700_000_000;

    // 1. Plain delta-seconds
    let h1 = vec![("Retry-After", "90")];
    assert_eq!(extract_cooldown_from_headers(h1, now), Some(90));

    // 2. HTTP-Date RFC 2822
    let future_dt = chrono::DateTime::from_timestamp(now + 250, 0).unwrap();
    let h2 = vec![("retry-after", future_dt.to_rfc2822())];
    assert_eq!(extract_cooldown_from_headers(h2, now), Some(250));

    // 3. x-ratelimit-reset epoch timestamp
    let h3 = vec![("x-ratelimit-reset", (now + 75).to_string())];
    assert_eq!(extract_cooldown_from_headers(h3, now), Some(75));

    // 4. Combined headers (takes maximum)
    let h4 = vec![
        ("retry-after".to_string(), "60".to_string()),
        ("x-ratelimit-reset".to_string(), (now + 140).to_string()),
    ];
    assert_eq!(extract_cooldown_from_headers(h4, now), Some(140));
}

#[tokio::test]
async fn test_dynamic_quarantine_preserves_bayesian_priors() {
    let pool = Arc::new(create_in_memory_sqlite_pool().await.unwrap());
    souls_model_router::ensure_router_tables(&pool).await.unwrap();

    let model_a = ModelRecord {
        model_id: "deepseek/deepseek-chat".to_string(),
        provider_type: "CLOUD".to_string(),
        vram_base_mb: 0,
        kv_cache_cost_per_k: 0,
        max_context_window: 64_000,
        specialty_tags: "[\"code\"]".to_string(),
        score_lmarena: 1300,
        cost_input_per_m: 0.14,
        cost_output_per_m: 0.28,
        is_active: true,
        ema_latency_ms: 350,
        success_rate_ema: 0.99,
    };

    let model_b = ModelRecord {
        model_id: "qwen/qwen-2.5-coder-32b".to_string(),
        provider_type: "CLOUD".to_string(),
        vram_base_mb: 0,
        kv_cache_cost_per_k: 0,
        max_context_window: 32_000,
        specialty_tags: "[\"code\"]".to_string(),
        score_lmarena: 1280,
        cost_input_per_m: 0.20,
        cost_output_per_m: 0.40,
        is_active: true,
        ema_latency_ms: 400,
        success_rate_ema: 0.98,
    };

    upsert_model(&pool, &model_a).await.unwrap();
    upsert_model(&pool, &model_b).await.unwrap();

    let router = ParetoBanditRouter::new_for_testing((*pool).clone(), 45.0, 1024)
        .await
        .unwrap();

    // Initial prior state for Model A
    let initial_prior = fetch_prior(&pool, &model_a.model_id, "code").await.unwrap();
    assert_eq!(initial_prior.alpha, 2.0);
    assert_eq!(initial_prior.beta, 2.0);

    // Trigger HTTP 429 incident with dynamic header instructing 120s cooldown
    let headers = vec![("retry-after", "120")];
    let eff_cd = router.record_incident_with_headers(
        &model_a.model_id,
        IncidentKind::RateLimited429,
        headers,
    );
    assert_eq!(eff_cd, 120);

    let (quarantined, remaining) = router.cooldown_manager().is_quarantined(&model_a.model_id);
    assert!(quarantined);
    assert!(remaining.unwrap() > 100);

    // Record rate-limit outcome event through worker queue
    let rate_limit_outcome = TaskOutcome {
        task_id: "task_test_429".to_string(),
        tier_name: model_a.model_id.clone(),
        task_category: "code".to_string(),
        success: false,
        structural_score: 0.0,
        direct_cost_usd: 0.0,
        latency_ms: 120.0,
        tokens_input: 500,
        tokens_output: 0,
        is_rate_limit: true,
        retry_after_secs: Some(120),
    };
    router
        .record_outcome("task_test_429", &rate_limit_outcome)
        .await
        .unwrap();

    // Verify Bayesian prior was NOT penalized (beta remains unchanged!)
    let post_prior = fetch_prior(&pool, &model_a.model_id, "code").await.unwrap();
    assert_eq!(post_prior.alpha, initial_prior.alpha);
    assert_eq!(post_prior.beta, initial_prior.beta);

    // Verify fallback chain routes to healthy Model B as primary
    let task = TaskContext::new("task_routing_check", "sess_01", "code", 1000, 200);

    let chain = router.resolve_fallback_chain(&task, 3).await.unwrap();
    // Primary must be Model B because Model A is in quarantine!
    assert_eq!(chain.primary.model_id, model_b.model_id);
    assert!(!chain.primary.in_cooldown);

    // Model A is present in fallbacks marked in_cooldown
    let fallback_a = chain.fallbacks.iter().find(|c| c.model_id == model_a.model_id);
    assert!(fallback_a.is_some());
    assert!(fallback_a.unwrap().in_cooldown);
}

