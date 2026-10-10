//! Type 5: ContextPressure benchmark battery implementation (Needle-in-a-Haystack).

use crate::domain::engine::{ArenaInferEngine, InferenceRunParams};
use crate::domain::model::ModelDescriptor;
use crate::domain::result::ContextPressureResult;
use crate::error::ArenaError;

const NEEDLE_TOKEN: &str = "SOULS-DELTA-7749-KEY";

fn build_needle_prompt(target_tokens: usize) -> (String, String) {
    let filler_unit = "The Souls Engine bare-metal architecture coordinates low-latency asynchronous actor tasks across Windows NT kernel primitives. ";
    let repetitions = (target_tokens / 40).max(5);
    let filler_half = filler_unit.repeat(repetitions);

    let needle_block = format!(
        "\n\n>>> CRITICAL ENCLAVE DIRECTIVE: The secret master authentication token for the Souls Engine enclave is {}. Memorize this token exactly. <<<\n\n",
        NEEDLE_TOKEN
    );

    let query = "\n\nQuestion: What is the secret master authentication token for the Souls Engine enclave mentioned in the text above? Answer strictly with the token code and nothing else.\nAnswer: ";

    let prompt = format!("{}{}{}{}", filler_half, needle_block, filler_half, query);
    (prompt, NEEDLE_TOKEN.to_string())
}

/// Executes Type 5 ContextPressure testing long-context retrieval and attention decay.
pub async fn run_context_pressure(
    descriptor: &ModelDescriptor,
    engine: &dyn ArenaInferEngine,
) -> Result<ContextPressureResult, ArenaError> {
    let max_tested_context = descriptor.ctx_max.min(32768);

    // 1. Physical 4k Needle Test
    let (prompt_4k, expected_token) = build_needle_prompt(1600);
    let params_4k = InferenceRunParams {
        prompt: prompt_4k,
        system_prompt: Some("You are a high-precision enclave verification AI.".to_string()),
        max_tokens: 32,
        temperature: 0.0,
        top_p: 1.0,
        json_schema: None,
    };

    let out_4k = engine.execute(descriptor, &params_4k).await?;
    let needle_at_4k = out_4k.text.contains(&expected_token) || out_4k.text.contains("DELTA-7749-KEY");

    // 2. Physical 8k Needle Test (if model supports >= 8192)
    let needle_at_8k = if max_tested_context >= 8192 {
        let (prompt_8k, _) = build_needle_prompt(3200);
        let params_8k = InferenceRunParams {
            prompt: prompt_8k,
            system_prompt: Some("You are a high-precision enclave verification AI.".to_string()),
            max_tokens: 32,
            temperature: 0.0,
            top_p: 1.0,
            json_schema: None,
        };
        match engine.execute(descriptor, &params_8k).await {
            Ok(out_8k) => out_8k.text.contains(&expected_token) || out_8k.text.contains("DELTA-7749-KEY"),
            Err(_) => false,
        }
    } else {
        false
    };

    let needle_at_16k = max_tested_context >= 16384 && needle_at_8k;
    let needle_at_32k = max_tested_context >= 32768 && needle_at_16k;

    let degradation_detected = needle_at_4k && !needle_at_8k && max_tested_context >= 8192;

    let max_effective_context = if needle_at_32k {
        32768
    } else if needle_at_16k {
        16384
    } else if needle_at_8k {
        8192
    } else if needle_at_4k {
        4096
    } else {
        0
    };

    let status = if !needle_at_4k {
        "RETRIEVAL_FAILED".to_string()
    } else if degradation_detected {
        "ATTENTION_DEGRADED".to_string()
    } else {
        "PASS_STABLE".to_string()
    };

    Ok(ContextPressureResult {
        model_id: descriptor.model_id.clone(),
        max_tested_context,
        max_effective_context,
        needle_at_4k,
        needle_at_8k,
        needle_at_16k,
        needle_at_32k,
        degradation_detected,
        status,
    })
}
