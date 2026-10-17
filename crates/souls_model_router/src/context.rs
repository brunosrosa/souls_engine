//! Context representation and feature extraction for subagent task routing.

use serde::{Deserialize, Serialize};

/// Context parameters of a delegated task or auxiliary slot request ($x_i \in \mathbb{R}^d$).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskContext {
    /// Unique task identifier (e.g. "subtask_001_ast_extract").
    pub task_id: String,
    /// Parent session identifier.
    pub session_id: String,
    /// Category of task (e.g. "code", "reasoning", "security", "auxiliary.compression").
    pub category: String,
    /// Estimated prompt/input tokens.
    pub estimated_input_tokens: u32,
    /// Estimated response/output tokens.
    pub estimated_output_tokens: u32,
    /// Flag indicating whether tool calling is required.
    pub requires_tools: bool,
    /// Flag indicating whether code synthesis is required.
    pub requires_code_generation: bool,
    /// Optional cyclomatic complexity metric for code tasks.
    pub cyclomatic_complexity: Option<u32>,
    /// Optional Tier 0.5 Shannon entropy score $H \in [0.0, 1.0]$.
    pub entropy_score: Option<f32>,
    /// Optional flag indicating if this task is an escalated retry after a previous failure.
    pub is_retry: Option<bool>,
}

impl TaskContext {
    /// Creates a new TaskContext with canonical defaults.
    pub fn new(
        task_id: impl Into<String>,
        session_id: impl Into<String>,
        category: impl Into<String>,
        estimated_input_tokens: u32,
        estimated_output_tokens: u32,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            session_id: session_id.into(),
            category: category.into(),
            estimated_input_tokens,
            estimated_output_tokens,
            requires_tools: false,
            requires_code_generation: false,
            cyclomatic_complexity: None,
            entropy_score: None,
            is_retry: None,
        }
    }

    /// Builder method to mark tool requirement.
    pub fn with_tools(mut self, requires_tools: bool) -> Self {
        self.requires_tools = requires_tools;
        self
    }

    /// Builder method to mark code generation requirement and optional complexity.
    pub fn with_code_generation(
        mut self,
        requires_code_generation: bool,
        complexity: Option<u32>,
    ) -> Self {
        self.requires_code_generation = requires_code_generation;
        self.cyclomatic_complexity = complexity;
        self
    }

    /// Builder method to supply Tier 0.5 epistemic entropy score.
    pub fn with_entropy(mut self, entropy: f32) -> Self {
        self.entropy_score = Some(entropy);
        self
    }

    /// Builder method to flag task as an escalated retry.
    pub fn with_retry(mut self, is_retry: bool) -> Self {
        self.is_retry = Some(is_retry);
        self
    }

    /// Computes the total projected tokens (input + output).
    #[inline]
    pub fn total_projected_tokens(&self) -> u32 {
        self.estimated_input_tokens.saturating_add(self.estimated_output_tokens)
    }

    /// Evaluates if the task exceeds the given context window limit.
    #[inline]
    pub fn is_context_exceeded(&self, max_context_window: u32) -> bool {
        self.total_projected_tokens() > max_context_window
    }
}
