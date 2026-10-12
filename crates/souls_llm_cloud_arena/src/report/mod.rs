//! # report/mod.rs — Subsistema de Relatórios da Cloud Arena

pub mod markdown;
pub mod terminal;

pub use markdown::generate_markdown_report;
pub use terminal::print_champions_table;
