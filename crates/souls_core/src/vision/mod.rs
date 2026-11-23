//! # Cognitive Vision $\mathcal{O}(1)$ & AXTree LEAN (`souls_core::vision`)
//!
//! Replaces token-heavy Vision Language Models (VLMs) with zero-GPU VRAM structural
//! accessibility maps in LEAN dot-notation format with deterministic geometric click centering.

pub mod ax_tree;
pub mod diff;
pub mod snapshot;

pub use ax_tree::{
    capture_live_windows_ax_tree, filter_ax_tree_by_intent, parse_cdp_ax_tree,
    standard_base64_encode, AXNode, AXNodeState, AXTreeCompressionMetrics, AXTreeSerializer,
    ZERO_VLM_VRAM_CONSUMPTION_MB,
};
pub use diff::compute_ax_tree_diff;
pub use snapshot::{assess_blind_spot, capture_on_demand_snapshot, generate_som_marks};
