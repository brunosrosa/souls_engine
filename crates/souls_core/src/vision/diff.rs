//! Structural accessibility tree delta engine (`souls_core::vision::diff`).
//!
//! Computes compact LEAN diffs between consecutive UI accessibility snapshots,
//! emitting only `+`, `-`, and `~` changes to save context tokens.

use crate::vision::ax_tree::{AXNode, AXTreeSerializer};
use souls_protocol::multimodal::AXTreeDiffSummary;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
struct FlattenedNodeInfo {
    compact_tag: String,
    value: Option<String>,
    bounds: [f64; 4],
}

fn node_segment(node: &AXNode) -> String {
    match node.name.as_deref() {
        Some(n) if !n.is_empty() => format!("{}[{n}]", node.role),
        _ => node.role.clone(),
    }
}

fn flatten_tree(
    node: &AXNode,
    prefix: &str,
    sibling_counts: &mut HashMap<String, usize>,
    out: &mut BTreeMap<String, FlattenedNodeInfo>,
) {
    let base_seg = node_segment(node);
    let base_path = if prefix.is_empty() {
        base_seg
    } else {
        format!("{prefix}.{base_seg}")
    };

    let count = sibling_counts.entry(base_path.clone()).or_insert(0);
    *count += 1;
    let unique_path = if *count == 1 {
        base_path
    } else {
        format!("{base_path}#{count}")
    };

    out.insert(
        unique_path.clone(),
        FlattenedNodeInfo {
            compact_tag: node.states.compact_tag(),
            value: node.value.clone(),
            bounds: node.bounds,
        },
    );

    let mut child_counts = HashMap::new();
    for child in &node.children {
        flatten_tree(child, &unique_path, &mut child_counts, out);
    }
}

fn format_state_and_value(info: &FlattenedNodeInfo, include_value: bool) -> String {
    if !include_value {
        return info.compact_tag.clone();
    }
    match info.value.as_deref() {
        Some(v) if !v.is_empty() => {
            if info.compact_tag.is_empty() {
                format!("='{v}'")
            } else {
                format!("='{v}'{}", info.compact_tag)
            }
        }
        _ => info.compact_tag.clone(),
    }
}

/// Computes a compact LEAN structural diff between `before` and `after` accessibility trees.
pub fn compute_ax_tree_diff(before: &AXNode, after: &AXNode) -> AXTreeDiffSummary {
    let mut before_map = BTreeMap::new();
    let mut after_map = BTreeMap::new();
    flatten_tree(before, "", &mut HashMap::new(), &mut before_map);
    flatten_tree(after, "", &mut HashMap::new(), &mut after_map);

    let mut added_nodes = Vec::new();
    let mut removed_nodes = Vec::new();
    let mut changed_states = Vec::new();

    for (path, info_after) in &after_map {
        match before_map.get(path) {
            None => {
                let cx = (info_after.bounds[0] + info_after.bounds[2]) / 2.0;
                let cy = (info_after.bounds[1] + info_after.bounds[3]) / 2.0;
                let val_part = info_after
                    .value
                    .as_deref()
                    .filter(|v| !v.is_empty())
                    .map(|v| format!("='{v}'"))
                    .unwrap_or_default();
                added_nodes.push(format!(
                    "{path}{val_part}{} {{click: ({cx:.0}, {cy:.0})}}",
                    info_after.compact_tag
                ));
            }
            Some(info_before) => {
                let tag_changed = info_before.compact_tag != info_after.compact_tag;
                let val_changed = info_before.value != info_after.value;
                if tag_changed || val_changed {
                    let before_desc = format_state_and_value(info_before, val_changed);
                    let after_desc = format_state_and_value(info_after, val_changed);
                    changed_states.push(format!("{path}: '{before_desc}' -> '{after_desc}'"));
                }
            }
        }
    }

    for path in before_map.keys() {
        if !after_map.contains_key(path) {
            removed_nodes.push(path.clone());
        }
    }

    let mut diff_lines = Vec::new();
    for line in &added_nodes {
        diff_lines.push(format!("+ {line}"));
    }
    for line in &removed_nodes {
        diff_lines.push(format!("- {line}"));
    }
    for line in &changed_states {
        diff_lines.push(format!("~ {line}"));
    }

    let compact_lean_diff = if diff_lines.is_empty() {
        "[NO_CHANGES]".to_string()
    } else {
        diff_lines.join("\n")
    };

    let full_lean_bytes = AXTreeSerializer::new().serialize_to_lean(after).len();
    let diff_lean_bytes = compact_lean_diff.len();
    let token_savings_pct = ((1.0 - (diff_lean_bytes as f64 / full_lean_bytes.max(1) as f64))
        * 100.0)
        .clamp(0.0, 99.9);

    AXTreeDiffSummary {
        added_nodes,
        removed_nodes,
        changed_states,
        compact_lean_diff,
        full_lean_bytes,
        diff_lean_bytes,
        token_savings_pct,
    }
}
