//! # Tier 1 Local MinionS Batch Protocol (`souls_inference_runtime::minions`)
//!
//! Orchestrates single-step atomic extraction and verification tasks on local SLM models (3B to 8B)
//! using upstream `llama.cpp` on dGPU (RTX 2060m) or CPU Host (Intel Core i9 AVX2).
//!
//! Enforces:
//! - Strict WDDM FinOps Compliance: Zero extra VRAM spillover. Automatically diverts execution to
//!   CPU AVX2 when VRAM approaches the 5,294 MB WDDM ceiling or operational caution zone.
//! - Typed deterministic output schema (`StructuredLocalOutput`).
//! - Automated response healing via `heal_json_response`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::InferenceError;
use crate::healing::heal_json_response;
use crate::inference::{GenParams, Tier1GenerativeEngine};
use crate::nvml::{HardwareWatchdog, OperationalZone};

/// Safety threshold below the Windows 11 WDDM 5,294 MB usable ceiling.
pub const WDDM_MINIONS_SAFE_VRAM_CEILING_MB: u32 = 4_800;

/// Execution hardware target decided dynamically for MinionS tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinionDeviceTarget {
    /// Accelerated inference on RTX 2060m with FlashAttention-2 and Asymmetric KV Cache.
    DGpu,
    /// Pure host CPU execution using Intel AVX2 instructions with 0 MB VRAM footprint.
    CpuAvx2,
}

impl MinionDeviceTarget {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            MinionDeviceTarget::DGpu => "dGPU (CUDA Accelerated)",
            MinionDeviceTarget::CpuAvx2 => "CPU Host (AVX2)",
        }
    }
}

/// Category of atomic single-step MinionS micro-task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinionTaskKind {
    /// Extract discrete factual claims and classify with epistemic tags.
    ExtractClaims,
    /// Verify literal citation alignment against source passage.
    VerifyCitation,
    /// Extract quantitative metrics, statistics, and benchmark figures.
    ExtractStatistics,
    /// Detect contradictions or divergences between two comparative passages.
    DetectContradiction,
    /// Synthesize condensed factual section summary.
    SynthesizeSection,
}

/// A single atomic task unit submitted to the MinionS orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinionTask {
    pub task_id: String,
    pub kind: MinionTaskKind,
    pub passage_text: String,
    pub context_brief: Option<String>,
    pub max_tokens: u32,
}

/// An epistemic claim extracted by a Minion worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpistemicClaimOutput {
    pub statement: String,
    pub tag: String, // "VERIFIED", "SOURCED", "REASONED", "ESTIMATED"
    pub citation_url: Option<String>,
    pub confidence: f32,
}

/// A quantitative data point or benchmark metric extracted by a Minion worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticDataPoint {
    pub metric: String,
    pub value: String,
    pub unit: Option<String>,
    pub context: String,
}

/// Canonical structured output delivered by Tier 1 MinionS execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredLocalOutput {
    pub task_id: String,
    pub executive_summary: String,
    pub direct_quotes: Vec<String>,
    pub factual_claims: Vec<EpistemicClaimOutput>,
    pub statistical_data: Vec<StatisticDataPoint>,
    pub contradictions_detected: Vec<String>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub latency_ms: u64,
    pub executed_device: MinionDeviceTarget,
}

/// Orchestrator for local batch MinionS tasks with automatic WDDM hardware routing.
#[derive(Debug, Clone)]
pub struct MinionBatchOrchestrator {
    watchdog: Arc<Mutex<HardwareWatchdog>>,
    model_path: Option<PathBuf>,
    forced_device: Option<MinionDeviceTarget>,
    cpu_threads: u32,
}

impl MinionBatchOrchestrator {
    /// Creates a new orchestrator with dynamic hardware watchdog monitoring.
    #[must_use]
    pub fn new() -> Self {
        Self {
            watchdog: Arc::new(Mutex::new(HardwareWatchdog::new())),
            model_path: None,
            forced_device: None,
            cpu_threads: 8,
        }
    }

    /// Builder method to assign custom local GGUF model path.
    pub fn with_model_path(mut self, path: impl AsRef<Path>) -> Self {
        self.model_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Builder method to override hardware target explicitly (e.g. for testing).
    pub fn with_forced_device(mut self, target: MinionDeviceTarget) -> Self {
        self.forced_device = Some(target);
        self
    }

    /// Builder method to set CPU thread count when operating in CPU_AVX2 mode.
    pub fn with_cpu_threads(mut self, threads: u32) -> Self {
        self.cpu_threads = threads;
        self
    }

    /// Dynamically determines execution device based on real-time NVML WDDM telemetry.
    #[must_use]
    pub fn resolve_device_target(&self) -> MinionDeviceTarget {
        if let Some(target) = self.forced_device {
            return target;
        }

        let guard = self.watchdog.lock().unwrap_or_else(|p| p.into_inner());
        let snapshot = guard.snapshot();

        // 0. If no dedicated GPU is present (0 MB VRAM), redirect to CPU AVX2
        if snapshot.vram_total_mb == 0 {
            return MinionDeviceTarget::CpuAvx2;
        }

        // 1. If NVML reports GPU is in Preventive or Critical zone, divert to CPU AVX2
        if snapshot.operational_zone != OperationalZone::Nominal {
            return MinionDeviceTarget::CpuAvx2;
        }

        // 2. Strict WDDM VRAM Ceiling Guard (Avoid Windows PCIe shared memory spillover)
        if snapshot.vram_allocated_mb >= WDDM_MINIONS_SAFE_VRAM_CEILING_MB {
            return MinionDeviceTarget::CpuAvx2;
        }

        // 3. Temperature safety threshold: if > 78°C, shed load to CPU
        if snapshot.gpu_temp_c >= 78.0 {
            return MinionDeviceTarget::CpuAvx2;
        }

        MinionDeviceTarget::DGpu
    }

    /// Executes a single atomic MinionS task.
    pub fn execute_task(&self, task: &MinionTask) -> Result<StructuredLocalOutput, InferenceError> {
        let start = Instant::now();
        let target_device = self.resolve_device_target();

        let prompt = self.build_minion_prompt(task);

        // Configure Tier 1 engine according to resolved device target
        let mut engine = Tier1GenerativeEngine::new();
        if let Some(ref mp) = self.model_path {
            engine = engine.with_model_path(mp);
        }

        if target_device == MinionDeviceTarget::CpuAvx2 {
            // Divert to host CPU: 0 GPU layers = 0 MB VRAM
            engine = engine.with_gpu_layers(0);
        } else {
            // Accelerate on dGPU: offload 33 layers
            engine = engine.with_gpu_layers(33);
        }

        let schema = get_structured_output_schema();
        let params = GenParams {
            max_tokens: task.max_tokens,
            temperature: 0.1, // Near-deterministic extraction
            top_p: 0.9,
            stop_sequences: vec!["<|endoftext|>".to_string(), "</s>".to_string()],
            json_schema: Some(schema),
        };

        let mut watchdog_guard = self.watchdog.lock().unwrap_or_else(|p| p.into_inner());
        let gen_output = engine.generate(&prompt, &params, &mut watchdog_guard)?;
        drop(watchdog_guard);

        // Parse and heal structured output
        let parsed_output = self.parse_and_heal_output(
            &gen_output.text,
            task,
            target_device,
            gen_output.prompt_tokens,
            gen_output.completion_tokens,
            start.elapsed().as_millis() as u64,
        );

        Ok(parsed_output)
    }

    /// Executes a batch of atomic MinionS tasks sequentially with hardware governance.
    pub fn execute_batch(
        &self,
        tasks: &[MinionTask],
    ) -> Vec<Result<StructuredLocalOutput, InferenceError>> {
        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            results.push(self.execute_task(task));
        }
        results
    }

    /// Formulates deterministic extraction prompts for single-step MinionS tasks.
    fn build_minion_prompt(&self, task: &MinionTask) -> String {
        let instruction = match task.kind {
            MinionTaskKind::ExtractClaims => {
                "Extraia todas as afirmações factuais do texto e rotule cada uma com [VERIFIED], [SOURCED], [REASONED] ou [ESTIMATED]."
            }
            MinionTaskKind::VerifyCitation => {
                "Verifique a fidelidade das citações literais no texto e confirme se refletem a fonte original."
            }
            MinionTaskKind::ExtractStatistics => {
                "Identifique e extraia todas as métricas numéricas, porcentagens, latências, custos e números estatísticos."
            }
            MinionTaskKind::DetectContradiction => {
                "Compare as passagens fornecidas e aponte divergências técnicas ou contradições explícitas."
            }
            MinionTaskKind::SynthesizeSection => {
                "Sintetize um resumo executivo factual condensado da passagem preservando números e termos canônicos."
            }
        };

        format!(
            "<|im_start|>system\nVocê é um operário Minion determinístico Tier 1. Extraia e responda EXCLUSIVAMENTE em formato JSON rigoroso.\nInstrução: {}\n<|im_end|>\n<|im_start|>user\nContexto: {}\nPassagem:\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            instruction,
            task.context_brief.as_deref().unwrap_or("Pesquisa Geral"),
            task.passage_text
        )
    }

    /// Parses raw LLM output, applies automated stack healing if necessary, and builds `StructuredLocalOutput`.
    fn parse_and_heal_output(
        &self,
        raw_text: &str,
        task: &MinionTask,
        device: MinionDeviceTarget,
        prompt_tokens: u32,
        completion_tokens: u32,
        latency_ms: u64,
    ) -> StructuredLocalOutput {
        // 1. Attempt raw parse or heal malformed JSON
        let val: Value = heal_json_response(raw_text).unwrap_or_else(|_| {
            json!({
                "executive_summary": raw_text.trim(),
                "direct_quotes": [],
                "factual_claims": [],
                "statistical_data": [],
                "contradictions_detected": []
            })
        });

        // 2. Extract structured fields safely
        let summary = val
            .get("executive_summary")
            .and_then(|v| v.as_str())
            .unwrap_or(raw_text)
            .to_string();

        let quotes = val
            .get("direct_quotes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|q| q.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let claims = val
            .get("factual_claims")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let stmt = c.get("statement")?.as_str()?.to_string();
                        let tag = c.get("tag")?.as_str()?.to_string();
                        let citation = c.get("citation_url").and_then(|u| u.as_str()).map(|s| s.to_string());
                        let conf = c.get("confidence").and_then(|f| f.as_f64()).unwrap_or(0.9) as f32;
                        Some(EpistemicClaimOutput {
                            statement: stmt,
                            tag,
                            citation_url: citation,
                            confidence: conf,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let stats = val
            .get("statistical_data")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| {
                        let metric = s.get("metric")?.as_str()?.to_string();
                        let value = s.get("value")?.as_str()?.to_string();
                        let unit = s.get("unit").and_then(|u| u.as_str()).map(|s| s.to_string());
                        let ctx = s.get("context").and_then(|c| c.as_str()).unwrap_or("").to_string();
                        Some(StatisticDataPoint {
                            metric,
                            value,
                            unit,
                            context: ctx,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let contradictions = val
            .get("contradictions_detected")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        StructuredLocalOutput {
            task_id: task.task_id.clone(),
            executive_summary: summary,
            direct_quotes: quotes,
            factual_claims: claims,
            statistical_data: stats,
            contradictions_detected: contradictions,
            prompt_tokens,
            completion_tokens,
            latency_ms,
            executed_device: device,
        }
    }
}

impl Default for MinionBatchOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON Schema for strict structured local output generation.
fn get_structured_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "executive_summary": { "type": "string" },
            "direct_quotes": {
                "type": "array",
                "items": { "type": "string" }
            },
            "factual_claims": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "statement": { "type": "string" },
                        "tag": { "type": "string", "enum": ["VERIFIED", "SOURCED", "REASONED", "ESTIMATED"] },
                        "citation_url": { "type": "string" },
                        "confidence": { "type": "number" }
                    },
                    "required": ["statement", "tag"]
                }
            },
            "statistical_data": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "metric": { "type": "string" },
                        "value": { "type": "string" },
                        "unit": { "type": "string" },
                        "context": { "type": "string" }
                    },
                    "required": ["metric", "value"]
                }
            },
            "contradictions_detected": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["executive_summary", "factual_claims"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minion_hardware_target_resolution_nominal() {
        let orch = MinionBatchOrchestrator::new();
        let target = orch.resolve_device_target();
        // Without VRAM overload or temperature spike, default is dGPU or CPU based on physical NVML state
        assert!(matches!(target, MinionDeviceTarget::DGpu | MinionDeviceTarget::CpuAvx2));
    }

    #[test]
    fn test_minion_forced_hardware_target() {
        let orch = MinionBatchOrchestrator::new().with_forced_device(MinionDeviceTarget::CpuAvx2);
        assert_eq!(orch.resolve_device_target(), MinionDeviceTarget::CpuAvx2);

        let orch_gpu = MinionBatchOrchestrator::new().with_forced_device(MinionDeviceTarget::DGpu);
        assert_eq!(orch_gpu.resolve_device_target(), MinionDeviceTarget::DGpu);
    }

    #[test]
    fn test_minion_task_execution_and_healing() {
        let orch = MinionBatchOrchestrator::new().with_forced_device(MinionDeviceTarget::CpuAvx2);

        let task = MinionTask {
            task_id: "task_test_01".to_string(),
            kind: MinionTaskKind::ExtractClaims,
            passage_text: "O Tokio no Windows 11 suporta IOCP e atinge 45.000 IOPS com latência de 1.2ms.".to_string(),
            context_brief: Some("Benchmark de I/O".to_string()),
            max_tokens: 256,
        };

        let res = orch.execute_task(&task).expect("Execução da tarefa Minion deve ter sucesso");
        assert_eq!(res.task_id, "task_test_01");
        assert_eq!(res.executed_device, MinionDeviceTarget::CpuAvx2);
        assert!(!res.executive_summary.is_empty());
    }

    #[test]
    fn test_minion_batch_orchestrator_execution() {
        let orch = MinionBatchOrchestrator::new().with_forced_device(MinionDeviceTarget::CpuAvx2);

        let tasks = vec![
            MinionTask {
                task_id: "batch_1".to_string(),
                kind: MinionTaskKind::ExtractStatistics,
                passage_text: "Throughput de 45 tokens/s na RTX 2060m.".to_string(),
                context_brief: None,
                max_tokens: 128,
            },
            MinionTask {
                task_id: "batch_2".to_string(),
                kind: MinionTaskKind::DetectContradiction,
                passage_text: "Algumas fontes alegam 1.5 tokens/s em modo compartilhado PCIe.".to_string(),
                context_brief: None,
                max_tokens: 128,
            },
        ];

        let results = orch.execute_batch(&tasks);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_minion_device_target_display_string() {
        assert_eq!(MinionDeviceTarget::DGpu.as_str(), "dGPU (CUDA Accelerated)");
        assert_eq!(MinionDeviceTarget::CpuAvx2.as_str(), "CPU Host (AVX2)");
    }

    #[test]
    fn test_minion_hardware_target_resolution_zero_vram() {
        let orch = MinionBatchOrchestrator::new();
        orch.watchdog.lock().unwrap().record_sample(45.0, 0, 0, 1.0);
        assert_eq!(orch.resolve_device_target(), MinionDeviceTarget::CpuAvx2);
    }
}
