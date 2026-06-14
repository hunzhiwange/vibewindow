use super::persist::{
    default_bracket_layout_format, default_edge_style, default_fishbone_layout_format,
    default_follow_theme_background, default_org_chart_layout_format, default_theme_group,
    default_tree_layout_format, load_persisted_finished,
};
use crate::app::{App, Screen};
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, OrgChartLayoutFormat, TreeLayoutFormat,
};
use iced::Point;
use serde_json::json;

#[test]
fn persisted_state_defaults_match_runtime_defaults() {
    assert_eq!(default_edge_style(), EdgeStyle::Solid);
    assert_eq!(default_org_chart_layout_format(), OrgChartLayoutFormat::default());
    assert_eq!(default_fishbone_layout_format(), FishboneLayoutFormat::default());
    assert_eq!(default_bracket_layout_format(), BracketLayoutFormat::default());
    assert_eq!(default_tree_layout_format(), TreeLayoutFormat::default());
    assert_eq!(default_theme_group(), "classic");
    assert!(default_follow_theme_background());
}

fn app() -> App {
    App::new().0
}

fn persisted_tab(id: &str, title: &str, markdown: &str, zoom: f32) -> serde_json::Value {
    json!({
        "id": id,
        "title": title,
        "file_path": "/tmp/map.md",
        "markdown": markdown,
        "pan_x": 12.0,
        "pan_y": -7.5,
        "zoom": zoom,
        "selected_path": [0],
        "node_positions": [{ "path": [0], "x": 1.5, "y": 2.5 }],
        "node_fills": [{ "path": [0], "rgba": 0x11223344_u32 }],
        "node_text_colors": [{ "path": [0], "rgba": 0x55667788_u32 }],
        "node_border_colors": [{ "path": [0], "rgba": 0x99AABBCC_u32 }],
        "node_border_style": "Dashed",
        "node_border_styles": [{ "path": [0], "style": "Dotted" }],
        "node_priorities": [
            { "path": [0], "priority": 1 },
            { "path": [1], "priority": 0 },
            { "path": [2], "priority": 10 }
        ],
        "node_urls": [
            { "path": [0], "url": " `https://example.com` " },
            { "path": [1], "url": "   " }
        ],
        "collapsed_paths": [[0, 1]],
        "background": 0x01020304_u32,
        "follow_theme_background": false,
        "edge_style": "Dashed",
        "edge_styles": [{ "path": [0], "style": "Dotted" }],
        "edge_colors": [{ "path": [0], "rgba": 0xAABBCCDD_u32 }],
        "doodle_rgba": 0,
        "doodle_width_px": 0.0,
        "doodles": [
            {
                "rgba": 0x12345678_u32,
                "width_px": 4.0,
                "points": [{ "x": 1.0, "y": 2.0 }]
            },
            {
                "rgba": 0x87654321_u32,
                "width_px": 5.0,
                "points": [{ "x": 1.0, "y": 2.0 }, { "x": 3.0, "y": 4.0 }]
            }
        ],
        "theme_group": "retro",
        "theme_variant": 3,
        "custom_themes": []
    })
}

#[test]
fn load_persisted_finished_applies_state_and_sanitizes_persisted_fields() {
    let mut app = app();
    let state = json!({
        "active_id": "tab-1",
        "tabs": [persisted_tab("tab-1", "Loaded", "# Root\n\n- Child", 25.0)]
    });

    let _ = load_persisted_finished(&mut app, Ok(Some(state)));

    assert_eq!(app.mindmap_active_tab_id.as_deref(), Some("tab-1"));
    assert_eq!(app.active_tab_id.as_deref(), Some("mindmap:tab-1"));
    assert!(matches!(app.screen, Screen::MindMapTool));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.title, "Loaded");
    assert_eq!(tab.file_path.as_deref(), Some("/tmp/map.md"));
    assert_eq!(tab.doc.text, "Root");
    assert_eq!(tab.zoom, 10.0);
    assert_eq!(tab.pan.x, 12.0);
    assert_eq!(tab.pan.y, -7.5);
    assert_eq!(tab.selected_path.as_deref(), Some(&[0][..]));
    assert_eq!(tab.node_positions.get(&vec![0]), Some(&Point::new(1.5, 2.5)));
    assert_eq!(tab.node_fills.get(&vec![0]), Some(&0x11223344));
    assert_eq!(tab.node_text_colors.get(&vec![0]), Some(&0x55667788));
    assert_eq!(tab.node_border_colors.get(&vec![0]), Some(&0x99AABBCC));
    assert_eq!(tab.node_border_style, EdgeStyle::Dashed);
    assert_eq!(tab.node_border_styles.get(&vec![0]), Some(&EdgeStyle::Dotted));
    assert_eq!(tab.node_priorities.get(&vec![0]), Some(&1));
    assert_eq!(tab.node_priorities.len(), 1);
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://example.com"));
    assert_eq!(tab.node_urls.len(), 1);
    assert!(tab.collapsed_paths.contains(&vec![0, 1]));
    assert_eq!(tab.background, Some(0x01020304));
    assert!(!tab.follow_theme_background);
    assert_eq!(tab.edge_style, EdgeStyle::Dashed);
    assert_eq!(tab.edge_styles.get(&vec![0]), Some(&EdgeStyle::Dotted));
    assert_eq!(tab.edge_colors.get(&vec![0]), Some(&0xAABBCCDD));
    assert_eq!(tab.doodle_rgba, 0x111827FF);
    assert_eq!(tab.doodle_width_px, 3.0);
    assert_eq!(tab.doodles.len(), 1);
    assert_eq!(tab.theme_group, "retro");
    assert_eq!(tab.theme_variant, 3);
    assert!(!tab.custom_themes.is_empty());
}

#[test]
fn load_persisted_finished_uses_default_doc_for_blank_markdown_and_clamps_low_zoom() {
    let mut app = app();
    let state = json!({
        "active_id": "blank",
        "tabs": [persisted_tab("blank", "Blank", "   ", 0.01)]
    });

    let _ = load_persisted_finished(&mut app, Ok(Some(state)));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.text, "中心主题");
    assert_eq!(tab.zoom, 0.1);
}

#[test]
fn load_persisted_finished_ignores_missing_or_invalid_payload() {
    let mut app = app();

    let _ = load_persisted_finished(&mut app, Ok(None));
    let _ = load_persisted_finished(&mut app, Err("failed".to_string()));
    let _ = load_persisted_finished(&mut app, Ok(Some(json!({ "tabs": "invalid" }))));

    assert!(app.mindmap_tabs.is_empty());
    assert_eq!(app.mindmap_active_tab_id, None);
}
