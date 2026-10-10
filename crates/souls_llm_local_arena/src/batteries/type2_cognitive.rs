//! Type 2: CognitiveEval benchmark battery implementation (Trilhas 2A, 2B e 2C).

use souls_inference_runtime::heal_json_response;
use crate::domain::engine::{ArenaInferEngine, InferenceRunParams};
use crate::domain::model::ModelDescriptor;
use crate::domain::result::CognitiveEvalResult;
use crate::error::ArenaError;

/// Executes Type 2 CognitiveEval assessing Rust AST, JSON Schema adherence, and CoT ratio.
pub async fn run_cognitive_eval(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
    track: &str,
) -> Result<CognitiveEvalResult, ArenaError> {
    match track.to_lowercase().as_str() {
        "json" | "structured_json" | "2b" => run_json_schema_track(descriptor, engine).await,
        "cot" | "reasoning" | "2c" => run_cot_reasoning_track(descriptor, engine).await,
        "ifeval" | "negative" | "strict_constraint" | "2d" => run_ifeval_negative_track(descriptor, engine).await,
        "bughunt" | "debugging" | "bug_hunting" | "2e" => run_bug_hunting_track(descriptor, engine).await,
        "mcp" | "mcp_tools" | "ambiguity" | "2f" => run_mcp_ambiguity_track(descriptor, engine).await,
        "pt" | "portuguese" | "fluency" | "2g" => run_portuguese_technical_track(descriptor, engine).await,
        "hard" | "hard_eval" | "suite" => run_hard_eval_composite(descriptor, engine).await,
        _ => run_rust_ast_track(descriptor, engine).await, // Default: Trilha 2A
    }
}


async fn run_rust_ast_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let params = InferenceRunParams {
        prompt: "Generate a complete Rust module with an enum State, a struct Task with lifetimes, and an impl block implementing Display."
            .to_string(),
        system_prompt: Some("Output strictly pure Rust code with zero markdown commentary.".to_string()),
        max_tokens: 384,
        temperature: 0.1,
        top_p: 0.95,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &params).await?;
    let (clean_code, think_tokens) = extract_thinking_tokens(&output.text);

    let ast_valid = validate_rust_syntax_delimiters(&clean_code);
    let accuracy_pct = if ast_valid { 95.0 } else { 20.0 };

    let status = if ast_valid && accuracy_pct >= 80.0 {
        "QUALIFIED".to_string()
    } else if accuracy_pct >= 50.0 {
        "PROVISIONAL".to_string()
    } else {
        "REJECTED".to_string()
    };

    let e3_score = (accuracy_pct * accuracy_pct) / ((output.ttft_ms / 1000.0) + 1.0);

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "RustAstSyntax (2A)".to_string(),
        accuracy_pct,
        ast_valid,
        json_strict_valid: false,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status,
    })
}

async fn run_json_schema_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "action": { "type": "string" },
            "target": { "type": "string" },
            "confidence": { "type": "number" }
        },
        "required": ["action", "target", "confidence"]
    });

    let params = InferenceRunParams {
        prompt: "Return a JSON object with action='refactor', target='crates/souls_ast', and confidence=0.98."
            .to_string(),
        system_prompt: Some("You must respond strictly with a valid JSON object matching the schema.".to_string()),
        max_tokens: 256,
        temperature: 0.1,
        top_p: 0.95,
        json_schema: Some(schema),
    };

    let output = engine.execute(descriptor, &params).await?;
    let (clean_text, think_tokens) = extract_thinking_tokens(&output.text);

    let direct_valid = serde_json::from_str::<serde_json::Value>(&clean_text).is_ok();
    let healed_valid = heal_json_response(&clean_text).is_ok();

    let (accuracy_pct, status) = if direct_valid {
        (100.0, "QUALIFIED (Strict)")
    } else if healed_valid {
        (85.0, "QUALIFIED (Healed)")
    } else {
        (10.0, "REJECTED")
    };

    let e3_score = (accuracy_pct * accuracy_pct) / ((output.ttft_ms / 1000.0) + 1.0);

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "StructuredJson (2B)".to_string(),
        accuracy_pct,
        ast_valid: true,
        json_strict_valid: direct_valid,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status: status.to_string(),
    })
}

async fn run_cot_reasoning_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let params = InferenceRunParams {
        prompt: "Why does the WDDM Desktop Window Manager consume up to 850 MB of VRAM on Windows 11?"
            .to_string(),
        system_prompt: Some("Think carefully inside <think> tags before giving a concise answer.".to_string()),
        max_tokens: 512,
        temperature: 0.2,
        top_p: 0.95,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &params).await?;
    let (answer_text, think_tokens) = extract_thinking_tokens(&output.text);
    let answer_tokens = (answer_text.split_whitespace().count() as f64 * 1.3) as u32;

    let cot_ratio = if answer_tokens > 0 {
        think_tokens as f64 / answer_tokens as f64
    } else {
        0.0
    };

    // Calculate E3: penalize excessive CoT ratio if answer is short
    let base_accuracy = 90.0;
    let cot_penalty = (1.0 + (1.0 + cot_ratio).ln()).max(1.0);
    let e3_score = ((base_accuracy * base_accuracy) / ((output.ttft_ms / 1000.0) + 1.0)) / cot_penalty;

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "CotEfficiency (2C)".to_string(),
        accuracy_pct: base_accuracy,
        ast_valid: true,
        json_strict_valid: false,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: cot_ratio,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status: "QUALIFIED".to_string(),
    })
}

fn extract_thinking_tokens(text: &str) -> (String, u32) {
    if let Some(start) = text.find("<think>") {
        if let Some(end) = text.find("</think>") {
            let think_slice = &text[start + 7..end];
            let think_tokens = (think_slice.split_whitespace().count() as f64 * 1.3) as u32;
            let answer = text[end + 8..].trim().to_string();
            return (answer, think_tokens.max(1));
        }
    }
    (text.trim().to_string(), 0)
}

fn validate_rust_syntax_delimiters(raw_code: &str) -> bool {
    let code = if let Some(start) = raw_code.find("```") {
        let after_start = &raw_code[start + 3..];
        let code_start = after_start.find('\n').map(|idx| &after_start[idx + 1..]).unwrap_or(after_start);
        if let Some(end) = code_start.find("```") {
            &code_start[..end]
        } else {
            code_start
        }
    } else {
        raw_code
    };

    let mut braces = 0i32;
    let mut parens = 0i32;
    let mut in_str = false;

    for ch in code.chars() {
        if ch == '"' {
            in_str = !in_str;
            continue;
        }
        if in_str {
            continue;
        }
        match ch {
            '{' => braces += 1,
            '}' => braces -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => {}
        }
        if braces < 0 || parens < 0 {
            return false;
        }
    }
    braces == 0 && parens == 0 && !code.trim().is_empty()
}

async fn run_ifeval_negative_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let params = InferenceRunParams {
        prompt: "Produce a Rust module containing exactly 3 functions named `init_state`, `step_state`, and `close_state`.\nStrict Constraints:\n1. Do NOT use the word 'unwrap'.\n2. Do NOT write any code comments (no // and no /* */).\n3. Keep total output under 25 lines of code.\n4. Conclude output strictly with: <|end_canonical|>".to_string(),
        system_prompt: Some("You must obey all strict negative constraints unconditionally.".to_string()),
        max_tokens: 300,
        temperature: 0.1,
        top_p: 0.95,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &params).await?;
    let (text, think_tokens) = extract_thinking_tokens(&output.text);
    let (score, valid) = evaluate_ifeval_constraints(&text);

    let e3_score = (score * score) / ((output.ttft_ms / 1000.0) + 1.0);
    let status = if score >= 80.0 { "QUALIFIED".to_string() } else { "REJECTED".to_string() };

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "StrictConstraintIFEval (2D)".to_string(),
        accuracy_pct: score,
        ast_valid: valid,
        json_strict_valid: false,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status,
    })
}

async fn run_bug_hunting_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let params = InferenceRunParams {
        prompt: "Analyze the following Rust code and identify the bug:\n```rust\nuse std::sync::Mutex;\nstruct Engine { a: Mutex<u32>, b: Mutex<u32> }\nimpl Engine {\n  fn op1(&self) { let _a = self.a.lock().unwrap(); let _b = self.b.lock().unwrap(); }\n  fn op2(&self) { let _b = self.b.lock().unwrap(); let _a = self.a.lock().unwrap(); }\n}\n```\nWhat critical concurrency bug exists here and how to fix it? Be concise.".to_string(),
        system_prompt: Some("You are an expert systems concurrency engineer. Answer directly.".to_string()),
        max_tokens: 256,
        temperature: 0.1,
        top_p: 0.95,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &params).await?;
    let (text, think_tokens) = extract_thinking_tokens(&output.text);
    let (score, identified) = evaluate_bug_hunting(&text);

    let e3_score = (score * score) / ((output.ttft_ms / 1000.0) + 1.0);
    let status = if score >= 80.0 { "QUALIFIED".to_string() } else { "REJECTED".to_string() };

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "BugHuntingConcurrency (2E)".to_string(),
        accuracy_pct: score,
        ast_valid: identified,
        json_strict_valid: false,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status,
    })
}

async fn run_mcp_ambiguity_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let params = InferenceRunParams {
        prompt: "You are an autonomous coding assistant with access to tools:\n1. read_file(path)\n2. souls_ast_slice(path, start, end)\n3. souls_ast_outline(path)\n4. write_file(path, content)\nDirective: Inspecionar tipos e assinaturas de 'crates/souls_ast/src/lib.rs' sem gastar tokens com o corpo das funções.\nSelect the single best tool and respond strictly with JSON: {\"tool\": \"<name>\", \"arguments\": {\"path\": \"<path>\"}}".to_string(),
        system_prompt: Some("Return strict JSON selecting the exact tool for dehydrated AST inspection.".to_string()),
        max_tokens: 128,
        temperature: 0.1,
        top_p: 0.95,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &params).await?;
    let (text, think_tokens) = extract_thinking_tokens(&output.text);
    let (score, valid_json) = evaluate_mcp_tool_choice(&text);

    let e3_score = (score * score) / ((output.ttft_ms / 1000.0) + 1.0);
    let status = if score >= 80.0 { "QUALIFIED".to_string() } else { "REJECTED".to_string() };

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "McpToolAmbiguity (2F)".to_string(),
        accuracy_pct: score,
        ast_valid: true,
        json_strict_valid: valid_json,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status,
    })
}

async fn run_portuguese_technical_track(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let params = InferenceRunParams {
        prompt: "Explique em 3 a 4 frases em português técnico formal por que cargas CUDA que excedem 5.2GB de VRAM em uma RTX 2060m sofrem queda catastrófica de desempenho no Windows 11.".to_string(),
        system_prompt: Some("Responda em português técnico conciso e preciso.".to_string()),
        max_tokens: 256,
        temperature: 0.2,
        top_p: 0.95,
        json_schema: None,
    };

    let output = engine.execute(descriptor, &params).await?;
    let (text, think_tokens) = extract_thinking_tokens(&output.text);
    let (score, valid_pt) = evaluate_portuguese_technical(&text);

    let e3_score = (score * score) / ((output.ttft_ms / 1000.0) + 1.0);
    let status = if score >= 75.0 { "QUALIFIED".to_string() } else { "REJECTED".to_string() };

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "PortugueseTechnicalFluency (2G)".to_string(),
        accuracy_pct: score,
        ast_valid: valid_pt,
        json_strict_valid: false,
        think_tokens_avg: think_tokens,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: output.ttft_ms,
        avg_tps: output.tps,
        e3_score,
        status,
    })
}

async fn run_hard_eval_composite(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<CognitiveEvalResult, ArenaError> {
    let r_ifeval = run_ifeval_negative_track(descriptor, engine).await?;
    let r_bughunt = run_bug_hunting_track(descriptor, engine).await?;
    let r_mcp = run_mcp_ambiguity_track(descriptor, engine).await?;
    let r_pt = run_portuguese_technical_track(descriptor, engine).await?;

    let composite_score = (r_ifeval.accuracy_pct * 0.30)
        + (r_bughunt.accuracy_pct * 0.30)
        + (r_mcp.accuracy_pct * 0.25)
        + (r_pt.accuracy_pct * 0.15);

    let avg_ttft = (r_ifeval.avg_ttft_ms + r_bughunt.avg_ttft_ms + r_mcp.avg_ttft_ms + r_pt.avg_ttft_ms) / 4.0;
    let avg_tps = (r_ifeval.avg_tps + r_bughunt.avg_tps + r_mcp.avg_tps + r_pt.avg_tps) / 4.0;
    let e3_score = (composite_score * composite_score) / ((avg_ttft / 1000.0) + 1.0);

    let status = if composite_score >= 80.0 {
        "QUALIFIED_HARD_EVAL".to_string()
    } else if composite_score >= 50.0 {
        "PROVISIONAL_HARD_EVAL".to_string()
    } else {
        "REJECTED_HARD_EVAL".to_string()
    };

    Ok(CognitiveEvalResult {
        model_id: descriptor.model_id.clone(),
        model_path: descriptor.file_path.to_string_lossy().to_string(),
        track: "HardEvalDiscriminatoryComposite (2H)".to_string(),
        accuracy_pct: composite_score,
        ast_valid: r_bughunt.ast_valid,
        json_strict_valid: r_mcp.json_strict_valid,
        think_tokens_avg: (r_ifeval.think_tokens_avg + r_bughunt.think_tokens_avg) / 2,
        cot_efficiency_ratio: 0.0,
        avg_ttft_ms: avg_ttft,
        avg_tps: avg_tps,
        e3_score,
        status,
    })
}

fn evaluate_ifeval_constraints(text: &str) -> (f64, bool) {
    let lower = text.to_lowercase();
    let mut score: f64 = 0.0;

    let has_init = lower.contains("init_state");
    let has_step = lower.contains("step_state");
    let has_close = lower.contains("close_state");
    if has_init && has_step && has_close {
        score += 30.0;
    } else if has_init || has_step || has_close {
        score += 15.0;
    }

    // Constraint 1: NO 'unwrap'
    if !lower.contains("unwrap") {
        score += 25.0;
    }

    // Constraint 2: NO code comments (// or /*)
    if !text.contains("//") && !text.contains("/*") {
        score += 20.0;
    }

    // Constraint 3: Under 25 lines
    let line_count = text.lines().count();
    if line_count <= 25 && line_count > 3 {
        score += 15.0;
    } else if line_count <= 35 {
        score += 8.0;
    }

    // Constraint 4: Sentinel token
    if text.contains("<|end_canonical|>") {
        score += 10.0;
    }

    let valid = score >= 70.0;
    (score.min(100.0), valid)
}

fn evaluate_bug_hunting(text: &str) -> (f64, bool) {
    let lower = text.to_lowercase();
    let mut score: f64 = 0.0;

    let found_deadlock = lower.contains("deadlock")
        || lower.contains("circular wait")
        || lower.contains("lock order")
        || lower.contains("lock ordering")
        || lower.contains("inversion");

    if found_deadlock {
        score += 50.0;
    }

    let proposed_fix = lower.contains("order")
        || lower.contains("consistent")
        || lower.contains("same order")
        || lower.contains("hierarchical")
        || lower.contains("parking_lot")
        || lower.contains("drop")
        || lower.contains("scope");

    if proposed_fix {
        score += 50.0;
    }

    (score.min(100.0), found_deadlock)
}

fn evaluate_mcp_tool_choice(text: &str) -> (f64, bool) {
    let lower = text.to_lowercase();
    let mut score: f64 = 0.0;

    let chose_outline = lower.contains("souls_ast_outline");
    let chose_wrong = lower.contains("read_file") || lower.contains("souls_ast_slice");

    if chose_outline && !chose_wrong {
        score += 60.0;
    } else if chose_outline {
        score += 30.0;
    }

    let parsed_json = serde_json::from_str::<serde_json::Value>(text.trim()).is_ok()
        || heal_json_response(text.trim()).is_ok();

    if parsed_json {
        score += 40.0;
    }

    (score.min(100.0), parsed_json && chose_outline)
}

fn evaluate_portuguese_technical(text: &str) -> (f64, bool) {
    let lower = text.to_lowercase();
    let mut score: f64 = 0.0;

    let has_pcie_shared = lower.contains("pcie")
        || lower.contains("compartilhada")
        || lower.contains("shared")
        || lower.contains("barramento")
        || lower.contains("paging")
        || lower.contains("paginação");

    if has_pcie_shared {
        score += 40.0;
    }

    let has_dwm_wddm = lower.contains("dwm")
        || lower.contains("wddm")
        || lower.contains("desktop window manager")
        || lower.contains("windows");

    if has_dwm_wddm {
        score += 30.0;
    }

    let has_pt_markers = lower.contains("desempenho")
        || lower.contains("memória")
        || lower.contains("memoria")
        || lower.contains("queda")
        || lower.contains("latência")
        || lower.contains("latencia")
        || lower.contains("sistema");

    if has_pt_markers {
        score += 30.0;
    }

    (score.min(100.0), score >= 60.0)
}

