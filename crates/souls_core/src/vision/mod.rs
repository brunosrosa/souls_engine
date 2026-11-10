//! # Cognitive Vision $\mathcal{O}(1)$ & AXTree LEAN (`souls_core::vision`)
//!
//! Replaces token-heavy Vision Language Models (VLMs) with zero-GPU VRAM structural
//! accessibility maps in LEAN dot-notation format with deterministic geometric click centering.

pub mod ax_tree;

pub use ax_tree::{
    parse_cdp_ax_tree, AXNode, AXNodeState, AXTreeCompressionMetrics, AXTreeSerializer,
    ZERO_VLM_VRAM_CONSUMPTION_MB,
};

