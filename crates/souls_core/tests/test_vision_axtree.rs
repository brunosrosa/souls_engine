//! Integration tests for Cognitive Vision $\mathcal{O}(1)$ & LEAN AXTree (ADR-029).

use souls_core::vision::{
    parse_cdp_ax_tree, AXNode, AXNodeState, AXTreeSerializer, ZERO_VLM_VRAM_CONSUMPTION_MB,
};

#[test]
fn test_geometric_click_centering() {
    // Bounds: [x_min, y_min, x_max, y_max]
    let node = AXNode::new("btn_save", "Button", [100.0, 200.0, 300.0, 260.0])
        .with_name("Salvar");

    let (cx, cy) = node.click_point();
    assert_eq!(cx, 200.0, "x_center must be (100 + 300) / 2");
    assert_eq!(cy, 230.0, "y_center must be (200 + 260) / 2");
    assert!(node.contains_point(200.0, 230.0));
    assert!(!node.contains_point(50.0, 230.0));
}

#[test]
fn test_state_encoding_collapsed_tags() {
    let clean = AXNodeState::new();
    assert_eq!(clean.compact_tag(), "");

    let focused = AXNodeState::new().with_focused(true);
    assert_eq!(focused.compact_tag(), "(F)");

    let checked_disabled = AXNodeState::new()
        .with_checked(true)
        .with_disabled(true);
    assert_eq!(checked_disabled.compact_tag(), "(T_)");

    let combo = AXNodeState::new()
        .with_focused(true)
        .with_checked(true)
        .with_selected(true);
    assert_eq!(combo.compact_tag(), "(FTS)");
}

#[test]
fn test_lean_dot_notation_and_token_compression() {
    // Build realistic desktop window accessibility tree
    let mut root = AXNode::new("win_1", "Desktop", [0.0, 0.0, 1920.0, 1080.0]);

    let mut window = AXNode::new("win_hermes", "Window", [50.0, 50.0, 1400.0, 900.0])
        .with_name("Hermes Agent");

    let mut main_pane = AXNode::new("pane_main", "Pane", [60.0, 100.0, 1380.0, 880.0])
        .with_name("Main View");

    let btn_save = AXNode::new("btn_save", "Button", [1200.0, 800.0, 1350.0, 850.0])
        .with_name("Salvar")
        .with_states(AXNodeState::new().with_focused(true));

    let edit_prompt = AXNode::new("edit_prompt", "Edit", [80.0, 200.0, 1100.0, 400.0])
        .with_name("Prompt Input")
        .with_value("Refatorar crate souls_core");

    let chk_rollback = AXNode::new("chk_rb", "CheckBox", [80.0, 450.0, 300.0, 480.0])
        .with_name("Auto-Rollback")
        .with_states(AXNodeState::new().with_checked(true));

    main_pane.add_child(edit_prompt);
    main_pane.add_child(chk_rollback);
    main_pane.add_child(btn_save);
    window.add_child(main_pane);
    root.add_child(window);

    // 1. Verify LEAN serialization text
    let serializer = AXTreeSerializer::new().with_click_coords(true);
    let lean_text = serializer.serialize_to_lean(&root);

    assert!(lean_text.contains("Desktop.Window[Hermes Agent].Pane[Main View].Button[Salvar](F) {click: (1275, 825)}"));
    assert!(lean_text.contains("CheckBox[Auto-Rollback](T) {click: (190, 465)}"));
    assert!(lean_text.contains("Edit[Prompt Input]='Refatorar crate souls_core'"));

    // 2. Verify compression metrics and 0 MB VRAM rule
    let metrics = AXTreeSerializer::measure_lean_compression(&root).unwrap();
    assert_eq!(metrics.vram_consumed_mb, ZERO_VLM_VRAM_CONSUMPTION_MB);
    assert_eq!(metrics.total_nodes, 6);
    assert!(
        metrics.reduction_ratio >= 0.60,
        "LEAN format should achieve strong token reduction (measured {:.1}%)",
        metrics.reduction_ratio * 100.0
    );
}

#[test]
fn test_cursor_hit_test_alt_space() {
    let mut root = AXNode::new("desktop", "Desktop", [0.0, 0.0, 1920.0, 1080.0]);
    let mut window = AXNode::new("win", "Window", [100.0, 100.0, 800.0, 600.0])
        .with_name("Souls Dashboard");
    let btn_sync = AXNode::new("btn", "Button", [200.0, 300.0, 350.0, 350.0])
        .with_name("Sincronizar");

    window.add_child(btn_sync);
    root.add_child(window);

    // Point inside Button (250, 320)
    let hit = root.hit_test(250.0, 320.0);
    assert!(hit.is_some());
    let (node, path) = hit.unwrap();
    assert_eq!(node.id, "btn");
    assert_eq!(path, "Desktop.Window[Souls Dashboard].Button[Sincronizar]");

    // Point outside Button but inside Window (150, 150)
    let hit_win = root.hit_test(150.0, 150.0);
    assert!(hit_win.is_some());
    let (node_win, path_win) = hit_win.unwrap();
    assert_eq!(node_win.id, "win");
    assert_eq!(path_win, "Desktop.Window[Souls Dashboard]");

    // Point outside Desktop
    let hit_miss = root.hit_test(2500.0, 2500.0);
    assert!(hit_miss.is_none());
}

#[test]
fn test_cdp_ax_tree_parser() {
    let cdp_payload = serde_json::json!([
        {
            "nodeId": "1",
            "role": { "value": "RootWebArea" },
            "name": { "value": "Souls Engine Web" },
            "bounds": { "x": 0.0, "y": 0.0, "width": 1920.0, "height": 1080.0 },
            "childIds": ["2"]
        },
        {
            "nodeId": "2",
            "role": { "value": "button" },
            "name": { "value": "Executar no Silício" },
            "bounds": { "x": 500.0, "y": 600.0, "width": 200.0, "height": 40.0 },
            "properties": [
                { "name": "focused", "value": { "value": true } }
            ]
        }
    ]);

    let tree_opt = parse_cdp_ax_tree(cdp_payload.as_array().unwrap()).unwrap();
    assert!(tree_opt.is_some());
    let tree = tree_opt.unwrap();

    assert_eq!(tree.id, "1");
    assert_eq!(tree.role, "RootWebArea");
    assert_eq!(tree.children.len(), 1);

    let child = &tree.children[0];
    assert_eq!(child.id, "2");
    assert_eq!(child.role, "button");
    assert_eq!(child.name.as_deref(), Some("Executar no Silício"));
    assert!(child.states.is_focused);
    assert_eq!(child.click_point(), (600.0, 620.0));
}

