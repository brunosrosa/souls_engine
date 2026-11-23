//! Multimodal Voice, Prosody, Rapport & Visual Perception DTOs.
//!
//! Pure data transfer objects shared across voice turn-taking, prosodic shaping,
//! user rapport modeling, Set-of-Mark (SoM) screen accessibility inspection, and
//! OpenAI-compatible audio endpoints, with zero I/O and zero unsafe code.

use serde::{Deserialize, Serialize};

/// Maximum allowed lexical mirroring ratio to avoid unnatural caricature.
pub const MAX_MIRRORING_RATIO: f64 = 0.35;

/// Clamps the lexical mirroring ratio strictly to `[0.0, 0.35]`.
pub fn clamp_mirroring_ratio(ratio: f64) -> f64 {
    if ratio.is_nan() {
        return 0.0;
    }
    ratio.clamp(0.0, MAX_MIRRORING_RATIO)
}

// ============================================================================
// 1. Cognitive Voice, Turn-Taking & Audio/Visual Enums
// ============================================================================

/// Cognitive stance that shapes turn-by-turn vocal pacing and response style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveVoiceMode {
    #[default]
    Auto,
    HandsOn,
    Sparring,
    Decompression,
    CrisisParalysis,
    Philosophical,
}

impl CognitiveVoiceMode {
    /// Parses a cognitive voice mode from snake_case, kebab-case, or Portuguese labels.
    ///
    /// Unknown values safely fall back to [`CognitiveVoiceMode::Auto`].
    pub fn from_str_lossy(raw: &str) -> Self {
        let normalized = raw
            .trim()
            .to_lowercase()
            .replace(['-', ' '], "_");

        match normalized.as_str() {
            "hands_on" | "handson" | "mao_na_massa" | "mão_na_massa" | "maonamassa"
            | "pratico" | "prático" | "execucao" | "execução" => Self::HandsOn,
            "sparring" | "debate" | "debate_critico" | "debate_crítico" | "desafio"
            | "provocacao" | "provocação" => Self::Sparring,
            "decompression" | "descompressao" | "descompressão" | "desabafo" | "leve"
            | "descontraido" | "descontraído" => Self::Decompression,
            "crisis_paralysis" | "crisis" | "crise" | "paralisia" | "crise_paralisia"
            | "apoio_na_crise" | "bloqueio" => Self::CrisisParalysis,
            "philosophical" | "filosofico" | "filosófico" | "filosofia" | "reflexivo"
            | "reflexao_profunda" | "reflexão_profunda" => Self::Philosophical,
            _ => Self::Auto,
        }
    }

    /// Returns the canonical `snake_case` identifier for this mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::HandsOn => "hands_on",
            Self::Sparring => "sparring",
            Self::Decompression => "decompression",
            Self::CrisisParalysis => "crisis_paralysis",
            Self::Philosophical => "philosophical",
        }
    }

    /// Returns a clean, jargon-free Portuguese label suitable for end-user UIs.
    pub fn human_label_pt(&self) -> &'static str {
        match self {
            Self::Auto => "Automático",
            Self::HandsOn => "Mão na massa",
            Self::Sparring => "Debate crítico",
            Self::Decompression => "Descompressão",
            Self::CrisisParalysis => "Apoio na crise",
            Self::Philosophical => "Reflexão profunda",
        }
    }

    /// Returns a concise, jargon-free Portuguese description of how the assistant behaves in this mode.
    pub fn human_description_pt(&self) -> &'static str {
        match self {
            Self::Auto => {
                "Adapta o ritmo e o tom da conversa automaticamente ao seu estado e contexto atual."
            }
            Self::HandsOn => {
                "Respostas curtas, diretas e orientadas a ação imediata, sem rodeios."
            }
            Self::Sparring => {
                "Testa ideias com perguntas incisivas e aponta pontos cegos de forma construtiva."
            }
            Self::Decompression => {
                "Escuta acolhedora, tom leve e zero cobrança para ajudar você a espairecer."
            }
            Self::CrisisParalysis => {
                "Acolhimento calmo que divide o momento em um único passo simples e alcançável."
            }
            Self::Philosophical => {
                "Ritmo pausado e aberto para explorar sentidos, nuances e perguntas profundas."
            }
        }
    }

    /// Builds an ephemeral per-turn Portuguese guidance note tailored to this mode and rapport profile.
    ///
    /// Keeps lexical mirroring clamped to `<= 0.35`, adds casual vocabulary hints only when
    /// `formality_score < 5.0`, and includes Layer-3 prosodic punctuation guidance (`...`, `—`, `,`)
    /// without mutating any static system prompt.
    pub fn ephemeral_turn_note_pt(&self, profile: &UserRapportProfileDto) -> String {
        let ratio = clamp_mirroring_ratio(profile.mirroring_ratio);
        let avg_len = profile.avg_sentence_length.max(1);
        let formality = if profile.formality_score.is_nan() {
            3.5
        } else {
            profile.formality_score.clamp(0.0, 10.0)
        };

        let mode_guidance = match self {
            Self::Auto => {
                "Modo Automático: ajuste ritmo e acolhimento ao estado imediato do usuário, mantendo clareza e naturalidade."
            }
            Self::HandsOn => {
                "Modo Mão na Massa: vá direto ao ponto, priorize passos práticos curtos e evite introduções longas."
            }
            Self::Sparring => {
                "Modo Debate Crítico: desafie premissas com respeito, aponte falhas lógicas e faça perguntas objetivas."
            }
            Self::Decompression => {
                "Modo Descompressão: use tom leve e empático, valide o cansaço sem pressionar por produtividade."
            }
            Self::CrisisParalysis => {
                "Modo Apoio na Crise: mantenha voz calma e estável; proponha apenas um micro-passo imediato e seguro."
            }
            Self::Philosophical => {
                "Modo Reflexão Profunda: sustente um ritmo contemplativo, explore nuances e deixe perguntas abertas respirarem."
            }
        };

        let casual_hint = if formality < 5.0 {
            if profile.slang_lexicon.is_empty() {
                format!(
                    "Tom casual (formalidade {:.1}/10): linguagem cotidiana natural permitida, com espelhamento sutil de até {:.0}%.",
                    formality,
                    ratio * 100.0
                )
            } else {
                let sample: Vec<&str> = profile
                    .slang_lexicon
                    .iter()
                    .take(4)
                    .map(String::as_str)
                    .collect();
                format!(
                    "Tom casual (formalidade {:.1}/10): espelhamento sutil máximo de {:.0}%; se couber naturalmente, reutilize expressões como [{}].",
                    formality,
                    ratio * 100.0,
                    sample.join(", ")
                )
            }
        } else {
            format!(
                "Tom estruturado (formalidade {:.1}/10): evite gírias e mantenha espelhamento contido em até {:.0}%.",
                formality,
                ratio * 100.0
            )
        };

        format!(
            "{} | {} | Cadência alvo: '{}' (~{} palavras/frase). Prosódia Camada 3: use '...' para pausas suaves de respiro, '—' para ênfase ou virada de ideia, e ',' para cadência falada fluida.",
            mode_guidance,
            casual_hint,
            profile.preferred_pacing,
            avg_len
        )
    }
}

/// Paralinguistic emotion tag inferred from user speech acoustics or transcript cues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DetectedEmotionTag {
    #[default]
    Neutral,
    Happy,
    Angry,
    Hesitant,
    Laughter,
    Sad,
    Focused,
}

impl DetectedEmotionTag {
    /// Parses an emotion tag from canonical snake_case or common English/Portuguese labels.
    pub fn from_str_lossy(raw: &str) -> Self {
        let normalized = raw.trim().to_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "happy" | "feliz" | "alegre" | "alegria" | "animado" | "excited" => Self::Happy,
            "angry" | "bravo" | "raiva" | "irritado" | "frustrated" | "frustrado" => Self::Angry,
            "hesitant" | "hesitante" | "duvida" | "dúvida" | "incerto" | "uncertain" => {
                Self::Hesitant
            }
            "laughter" | "laugh" | "riso" | "risada" | "rindo" => Self::Laughter,
            "sad" | "triste" | "tristeza" | "desanimado" | "melancolico" | "melancólico" => {
                Self::Sad
            }
            "focused" | "focado" | "concentrado" | "serio" | "sério" => Self::Focused,
            _ => Self::Neutral,
        }
    }
}

/// Turn-taking decision emitted by the prosodic completion evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TurnTakingVerdict {
    ContinueListening,
    EmitBackchannel,
    ContemplativeWait,
    #[default]
    TakeTurnImmediate,
    BargeInAbort,
    StandbyTimeout,
}

/// Execution tier used by local speech-to-text or text-to-speech synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioEngineTier {
    OnnxPrimary,
    LocalSecondary,
    #[default]
    WindowsNativeFallback,
}

/// Reason why structural UI inspection escalated to pixel-level visual analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisionEscalationReason {
    #[default]
    NoneAxSufficient,
    CanvasOrWebGlDetected,
    ZeroActionableNodes,
    RemoteDesktopOrVm,
    ExplicitVisualQaRequest,
}

/// Strategy selected for visual screen understanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisualAnalysisMode {
    #[default]
    Auto,
    CloudFrontier,
    LocalEphemeralVlm,
    DeterministicSomOnly,
}

// ============================================================================
// 2. Visual Set-of-Mark (SoM) & Accessibility Diff DTOs
// ============================================================================

/// Indexed Set-of-Mark bounding overlay mapped to an actionable accessibility element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SomOverlayMark {
    pub index: u32,
    pub node_id: String,
    pub role: String,
    pub label: String,
    pub bounds: [f64; 4],
    pub click_point: (f64, f64),
}

/// Assessment of whether structural accessibility data suffices or pixel capture is required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlindSpotAssessment {
    pub requires_pixel_snapshot: bool,
    pub reason: VisionEscalationReason,
    pub actionable_node_count: usize,
    pub canvas_node_count: usize,
    pub coverage_ratio: f64,
    pub recommendation_pt: String,
}

/// Compact delta between two consecutive UI accessibility tree snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AXTreeDiffSummary {
    #[serde(default)]
    pub added_nodes: Vec<String>,
    #[serde(default)]
    pub removed_nodes: Vec<String>,
    #[serde(default)]
    pub changed_states: Vec<String>,
    pub compact_lean_diff: String,
    pub full_lean_bytes: usize,
    pub diff_lean_bytes: usize,
    pub token_savings_pct: f64,
}

/// Consolidated visual perception snapshot combining SoM marks, blind-spot checks, and optional image data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualSnapshotResultDto {
    pub snapshot_id: String,
    pub window_title: String,
    pub app_name: String,
    pub width: u32,
    pub height: u32,
    pub sha256_hash: String,
    pub screen_unchanged: bool,
    #[serde(default)]
    pub som_marks: Vec<SomOverlayMark>,
    pub blind_spot: BlindSpotAssessment,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub image_base64_png: Option<String>,
    #[serde(default)]
    pub visual_analysis: Option<String>,
    pub analysis_engine_used: String,
    #[serde(default)]
    pub vram_consumed_mb: usize,
    #[serde(default)]
    pub latency_ms: f64,
}

// ============================================================================
// 3. Voice Turn-Taking, Rapport Profile & Audio Engine DTOs
// ============================================================================

/// Prosodic completion and turn-taking evaluation for a live speech segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PcompTurnEvaluationDto {
    pub transcript_tail: String,
    pub silence_duration_ms: u64,
    pub ends_with_open_connective: bool,
    pub syntactic_closure_score: f32,
    pub acoustic_stress_score: f32,
    pub recommended_wait_ms: u64,
    pub verdict: TurnTakingVerdict,
    #[serde(default)]
    pub suggested_backchannel: Option<String>,
}

/// Persistent user linguistic rapport and pacing profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserRapportProfileDto {
    pub user_id: String,
    pub formality_score: f64,
    pub avg_sentence_length: u32,
    pub preferred_pacing: String,
    pub slang_lexicon: Vec<String>,
    pub mirroring_ratio: f64,
    pub active_cognitive_mode: CognitiveVoiceMode,
    pub contemplative_base_ms: u64,
    pub backchannel_enabled: bool,
    pub last_emotion: DetectedEmotionTag,
    pub turn_count: u64,
    pub updated_at: i64,
}

impl Default for UserRapportProfileDto {
    fn default() -> Self {
        Self {
            user_id: "default".to_string(),
            formality_score: 3.5,
            avg_sentence_length: 12,
            preferred_pacing: "short_direct".to_string(),
            slang_lexicon: Vec::new(),
            mirroring_ratio: 0.25,
            active_cognitive_mode: CognitiveVoiceMode::Auto,
            contemplative_base_ms: 250,
            backchannel_enabled: true,
            last_emotion: DetectedEmotionTag::Neutral,
            turn_count: 0,
            updated_at: 0,
        }
    }
}

impl UserRapportProfileDto {
    /// Clamps profile parameters within safe bounds for vocal mirroring and turn pacing.
    pub fn sanitize(&mut self) {
        self.formality_score = if self.formality_score.is_nan() {
            3.5
        } else {
            self.formality_score.clamp(0.0, 10.0)
        };
        self.avg_sentence_length = self.avg_sentence_length.max(1);
        self.mirroring_ratio = clamp_mirroring_ratio(self.mirroring_ratio);
        self.contemplative_base_ms = self.contemplative_base_ms.clamp(100, 1000);
    }
}

/// Output of speech-to-text transcription enriched with emotion and turn-taking evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceTranscriptionResultDto {
    pub text: String,
    pub clean_text: String,
    #[serde(default)]
    pub emotion_tag: DetectedEmotionTag,
    #[serde(default)]
    pub emotion_confidence: f32,
    #[serde(default)]
    pub is_silence_hallucination_filtered: bool,
    pub turn_evaluation: PcompTurnEvaluationDto,
    #[serde(default)]
    pub engine_tier: AudioEngineTier,
    pub engine_name: String,
    pub language: String,
    #[serde(default)]
    pub vram_consumed_mb: usize,
    #[serde(default)]
    pub latency_ms: f64,
}

/// Request payload for prosody-aware voice synthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceSynthesisRequestDto {
    pub input: String,
    #[serde(default)]
    pub voice: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub response_format: Option<String>,
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub apply_prosody_shaping: Option<bool>,
    #[serde(default)]
    pub cognitive_mode: Option<CognitiveVoiceMode>,
}

/// Result payload returned after prosodic text shaping and audio synthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceSynthesisResultDto {
    pub shaped_text: String,
    #[serde(default)]
    pub sentences_chunks: Vec<String>,
    pub audio_format: String,
    pub audio_bytes_len: usize,
    pub audio_base64: String,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub contemplative_delay_ms: u64,
    #[serde(default)]
    pub engine_tier: AudioEngineTier,
    pub engine_name: String,
    #[serde(default)]
    pub vram_consumed_mb: usize,
    #[serde(default)]
    pub latency_ms: f64,
}

/// Diagnostic inspection report of Hermes voice configuration alignment with PT-BR local defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesVoiceCustomsReportDto {
    pub config_path: String,
    pub stt_enabled: bool,
    pub current_stt_language: String,
    pub current_stt_provider: String,
    pub current_tts_provider: String,
    pub current_voice_mode: String,
    pub is_synced_with_souls_pt_br: bool,
    #[serde(default)]
    pub issues_found: Vec<String>,
    pub human_summary_pt: String,
}

// ============================================================================
// 4. OpenAI-Compatible Audio Endpoint DTOs
// ============================================================================

/// OpenAI `/v1/audio/speech` compatible request payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAiSpeechRequest {
    #[serde(default)]
    pub model: Option<String>,
    pub input: String,
    #[serde(default)]
    pub voice: Option<String>,
    #[serde(default)]
    pub response_format: Option<String>,
    #[serde(default)]
    pub speed: Option<f32>,
}

/// OpenAI `/v1/audio/transcriptions` compatible response payload with optional emotion metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAiTranscriptionResponse {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotion: Option<DetectedEmotionTag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
