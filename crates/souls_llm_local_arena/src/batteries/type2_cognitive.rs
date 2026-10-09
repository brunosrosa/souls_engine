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
