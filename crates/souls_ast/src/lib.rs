//! `souls_ast` — Static analysis, dehydrated outlines, Myers diff, and gitoxide heatmap.

pub mod error;
pub mod heatmap;
pub mod myers;
pub mod treesitter;

pub use error::AstError;
pub use heatmap::{
    calculate_blast_radius, calculate_repo_frecency, BlastRadiusEntry, BlastRadiusResult,
    FileHeatEntry,
};
pub use myers::{
    compute_safe_myers_diff, myers_diff, myers_diff_with_stats, DiffChange, DiffTag, MyersDiffStats,
    MyersPatch, sanitize_lf,
};
pub use treesitter::{
    dehydrate_outline, extract_symbol_slice, parse_code_isolated, DehydratedOutline,
    ParsedSyntaxTree, SymbolSlice, WasmSandboxEngine, WasmTrap, FUEL_LIMIT,
    MEMORY_LIMIT_BYTES_GRAMMAR,
};
