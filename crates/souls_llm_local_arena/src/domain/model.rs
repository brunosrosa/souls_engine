//! Domain models representing local AI model descriptors and architectural tiers.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Architectural tier classification for the model in host physical hardware.
///
/// Governed strictly by ADR-007 (exclusively classifies model hardware scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    /// Tier 0: Ultralightweight CPU AVX2 models (0 MB VRAM), e.g. ModernBERT, GLiClass.
    Tier0Bootstrap,
    /// Tier 0.5: Epistemic sensor models (0 MB VRAM) for logit probing and entropy, e.g. SmolLM 360M, Gemma 2B.
    Tier05Epistemic,
    /// Tier 1: Master bare-metal interactive models on dGPU RTX 2060m (< 5.2GB), e.g. Qwen 3.5 4B, Mamba-Codestral 7B.
    Tier1Master,
    /// Tier 2: Background subagent & MoE models running in host DDR4 RAM offload, e.g. Bonsai 27B, Mixtral.
    Tier2Background,
    /// Tier 3: Multimodal Vision models requiring an mmproj projector sidecar.
    Tier3Vision,
    /// Tier 4: Speculative drafter models paired with larger models, e.g. DFlash, DSpark.
    Tier4Drafter,
}

impl ModelTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelTier::Tier0Bootstrap => "Tier 0",
            ModelTier::Tier05Epistemic => "Tier 0.5",
            ModelTier::Tier1Master => "Tier 1",
            ModelTier::Tier2Background => "Tier 2",
            ModelTier::Tier3Vision => "Tier 3",
            ModelTier::Tier4Drafter => "Tier 4",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            ModelTier::Tier0Bootstrap => "TIER 0: BOOTSTRAP & SANITY MODELS (CPU AVX2)",
            ModelTier::Tier05Epistemic => "TIER 0.5: SENSOR EPISTÊMICO & PROBING (CPU AVX2)",
            ModelTier::Tier1Master => "TIER 1: LIVE CHAT & MASTER AGENTS (dGPU RTX 2060m)",
            ModelTier::Tier2Background => "TIER 2: BACKGROUND AGENTS & MOE (HOST RAM OFFLOAD)",
            ModelTier::Tier3Vision => "TIER 3: MULTIMODAL VISION & VLM (mmproj PROJECTOR)",
            ModelTier::Tier4Drafter => "TIER 4: SPECULATIVE DRAFTERS (ACCELERATION COMBAT)",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let clean = s.trim().to_lowercase();
        if clean.contains("0.5") || clean == "tier05" || clean == "05" {
            Some(ModelTier::Tier05Epistemic)
        } else if clean.contains("0") || clean == "tier0" {
            Some(ModelTier::Tier0Bootstrap)
        } else if clean.contains("1") || clean == "tier1" {
            Some(ModelTier::Tier1Master)
        } else if clean.contains("2") || clean == "tier2" {
            Some(ModelTier::Tier2Background)
        } else if clean.contains("3") || clean == "tier3" {
            Some(ModelTier::Tier3Vision)
        } else if clean.contains("4") || clean == "tier4" {
            Some(ModelTier::Tier4Drafter)
        } else {
            None
        }
    }

    pub fn classify(path: &Path, size_mb: f64, has_mmproj: bool) -> Self {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
        
        if has_mmproj || filename.contains("vision") || filename.contains("-vl") || filename.contains("vlm") {
            return ModelTier::Tier3Vision;
        }

        if filename.contains("dspark") || filename.contains("dflash") || filename.contains("draft") {
            return ModelTier::Tier4Drafter;
        }

        if filename.contains("135m") || filename.contains("gliclass") || filename.contains("k1") || (size_mb < 350.0 && !filename.contains("360m")) {
            ModelTier::Tier0Bootstrap
        } else if filename.contains("360m") || filename.contains("500m") || filename.contains("0.5b") || filename.contains("790m") || filename.contains("1.5b") || filename.contains("1.2b") || filename.contains("1b") || (filename.contains("gemma") && filename.contains("2b")) || size_mb <= 1800.0 {
            ModelTier::Tier05Epistemic
        } else if filename.contains("27b") || filename.contains("33b") || filename.contains("moe") || filename.contains("laguna") || filename.contains("14b") || size_mb > 4500.0 {
            ModelTier::Tier2Background
        } else {
            ModelTier::Tier1Master
        }
    }
}

/// Rich descriptor of an inspected local model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub model_id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub tier: ModelTier,
    pub parameters: String,
    pub quant: String,
    pub size_mb: f64,
    pub ctx_max: u32,
    pub has_mmproj: bool,
    pub mmproj_path: Option<PathBuf>,
}

impl ModelDescriptor {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref().to_path_buf();
        let filename = p.file_name().and_then(|f| f.to_str()).unwrap_or("unknown");
        let size_bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        
        let has_mmproj = filename.contains("mmproj") || filename.contains("vision");
        let tier = ModelTier::classify(&p, size_mb, has_mmproj);
        let quant = extract_quant(filename);
        let parameters = extract_parameters(filename);

        let model_id = p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename)
            .to_string();

        Self {
            model_id: model_id.clone(),
            name: model_id,
            file_path: p,
            tier,
            parameters,
            quant,
            size_mb,
            ctx_max: 8192,
            has_mmproj,
            mmproj_path: None,
        }
    }
}

pub fn extract_quant(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.ends_with(".onnx") || lower.contains("onnx") {
        return "ONNX".to_string();
    }
    let quant_patterns = [
        "i1-q4_k_s", "i1-q4_k_m", "i1-iq3_m", "i1-iq4_xs",
        "q4_k_m", "q4_k_s", "q4_0", "q4_1",
        "q5_k_m", "q5_k_s", "q5_0", "q5_1",
        "q8_0", "q8_1", "q6_k", "q3_k_m", "q3_k_s", "q2_k",
        "iq4_xs", "iq4_nl", "iq3_m", "iq3_s", "iq2_xxs", "iq2_xs", "iq1_s",
        "fp16", "bf16", "f16"
    ];
    for pat in quant_patterns {
        if lower.contains(pat) {
            return pat.to_uppercase();
        }
    }
    "GGUF".to_string()
}

pub fn extract_parameters(name: &str) -> String {
    let lower = name.to_lowercase();
    let param_patterns = [
        "135m", "360m", "500m", "790m",
        "0.5b", "1b", "1.2b", "1.5b", "2b", "2.7b", "3b", "4b", "7b", "8b", "9b",
        "14b", "27b", "32b", "33b", "70b"
    ];
    for pat in param_patterns {
        if lower.contains(pat) {
            return pat.to_uppercase();
        }
    }
    "Unknown".to_string()
}
