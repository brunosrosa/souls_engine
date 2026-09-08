//! `souls_ast` — Static analysis, dehydrated outlines, Myers diff, and gitoxide heatmap.

pub mod error;
pub mod heatmap;
pub mod myers;
pub mod treesitter;

pub use error::AstError;
pub use heatmap::{calculate_repo_frecency, FileHeatEntry};
pub use myers::{
    compute_safe_myers_diff, myers_diff, myers_diff_with_stats, DiffChange, DiffTag, MyersDiffStats,
    MyersPatch,
};
pub use treesitter::{
    parse_code_isolated, ParsedSyntaxTree, WasmSandboxEngine, WasmTrap, FUEL_LIMIT,
    MEMORY_LIMIT_BYTES_GRAMMAR,
};
