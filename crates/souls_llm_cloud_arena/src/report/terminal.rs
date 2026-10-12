//! # terminal.rs — Visualização em Tabela para Terminal
//!
//! Formata saídas ricas no terminal usando comfy_table.

use crate::domain::{ProfileChampionReport, TaskProfileKind};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};

pub fn print_champions_table(champions: &[ProfileChampionReport]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Perfil").fg(Color::Cyan),
            Cell::new("Líder Qualidade").fg(Color::Green),
            Cell::new("Custo x Benefício").fg(Color::Yellow),
            Cell::new("Ultra Low-Cost").fg(Color::Blue),
        ]);

    for c in champions {
        let profile_str = match c.profile {
            TaskProfileKind::Coding => "Coding & SWE",
            TaskProfileKind::Agentic => "Agentic & Tools",
            TaskProfileKind::Reasoning => "Deep Reasoning",
            TaskProfileKind::Fast => "Fast Triage",
            TaskProfileKind::Design => "Design & Multimodal",
        };

        let leader_str = c.quality_leader.as_ref().map_or("—".to_string(), |e| {
            format!("{} (${:.2}/M)", e.display_name, e.blended_cost)
        });

        let best_val_str = c.best_value.as_ref().map_or("—".to_string(), |e| {
            format!("{} (${:.2}/M)", e.display_name, e.blended_cost)
        });

        let ultra_str = c.ultra_low_cost.as_ref().map_or("—".to_string(), |e| {
            format!("{} (${:.2}/M)", e.display_name, e.blended_cost)
        });

        table.add_row(Row::from(vec![
            Cell::new(profile_str),
            Cell::new(&leader_str),
            Cell::new(&best_val_str),
            Cell::new(&ultra_str),
        ]));
    }

    println!("{table}");
}
