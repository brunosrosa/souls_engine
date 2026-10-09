//! Reporters for terminal output and dossier markdown generation.

pub mod markdown;
pub mod terminal;

pub use markdown::export_markdown_dossier;
pub use terminal::{
    print_cognitive_eval_table, print_hardware_profile_table, print_model_list_table,
    print_pressure_table, print_speculative_table, print_vision_table,
};
