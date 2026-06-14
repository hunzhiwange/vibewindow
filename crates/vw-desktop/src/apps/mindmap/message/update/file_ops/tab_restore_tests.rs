use super::tab_restore::{new_tab_from_json, new_tab_from_md};
use crate::app::App;
use crate::apps::mindmap::state::{
    EdgeStyle, MindMapDiagramType, MindMapLayoutFormat, OrgChartLayoutFormat,
};

fn empty_app() -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;
    app.error_message = None;
    app
}

#[test]
fn markdown_restore_uses_default_doc_for_blank_input_and_file_name_title() {
    let mut app = empty_app();

    new_tab_from_md(&mut app, Some("/tmp/blank.md".to_string()), " \n\t ".to_string());

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.id, "mindmap-1");
    assert_eq!(tab.title, "blank.md");
    assert_eq!(tab.file_path.as_deref(), Some("/tmp/blank.md"));
    assert_eq!(tab.doc.text, "中心主题");
}

#[test]
fn json_restore_applies_state_filters_and_fallbacks() {
    let mut app = empty_app();
    let json = serde_json::json!({
        "format": "vibe-window-mindmap",
        "version": 1,
        "data": {
            "title": "Stored Title",
            "markdown": "# Root\n\n- A\n- B\n",
            "diagram_type": "OrgChart",
            "layout_format": "LeftAligned",
            "org_chart_layout_format": "LeftRight",
            "pan_x": 12.0,
            "pan_y": -5.0,
            "zoom": 0.01,
            "selected_path": [1],
            "node_positions": [{"path": [1], "x": 3.0, "y": 4.0}],
            "node_fills": [{"path": [1], "rgba": 10}],
            "node_text_colors": [{"path": [1], "rgba": 11}],
            "node_border_colors": [{"path": [1], "rgba": 12}],
            "node_border_style": "Dashed",
            "node_border_styles": [{"path": [1], "style": "Dotted"}],
            "node_priorities": [{"path": [0], "priority": 0}, {"path": [1], "priority": 9}],
            "node_urls": [{"path": [0], "url": "``"}, {"path": [1], "url": " ` https://example.test ` "}],
            "collapsed_paths": [[1]],
            "background": 99,
            "follow_theme_background": false,
            "edge_style": "Dotted",
            "edge_styles": [{"path": [1], "style": "Dashed"}],
            "edge_colors": [{"path": [1], "rgba": 13}],
            "doodle_rgba": 0,
            "doodle_width_px": 0.0,
            "doodles": [
                {"rgba": 1, "width_px": 1.0, "points": [{"x": 0.0, "y": 0.0}]},
                {"rgba": 2, "width_px": 2.0, "points": [{"x": 1.0, "y": 1.0}, {"x": 2.0, "y": 2.0}]}
            ],
            "theme_group": "custom",
            "theme_variant": 2,
            "custom_themes": []
        }
    })
    .to_string();

    new_tab_from_json(&mut app, None, json).unwrap();

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.title, "Stored Title");
    assert_eq!(tab.doc.text, "Root");
    assert_eq!(tab.diagram_type, MindMapDiagramType::OrgChart);
    assert_eq!(tab.layout_format, MindMapLayoutFormat::LeftAligned);
    assert_eq!(tab.org_chart_layout_format, OrgChartLayoutFormat::LeftRight);
    assert_eq!(tab.zoom, 0.1);
    assert_eq!(tab.selected_path.as_deref(), Some(&[1][..]));
    assert_eq!(tab.node_positions.get(&vec![1]).unwrap().x, 3.0);
    assert_eq!(tab.node_fills.get(&vec![1]), Some(&10));
    assert_eq!(tab.node_text_colors.get(&vec![1]), Some(&11));
    assert_eq!(tab.node_border_colors.get(&vec![1]), Some(&12));
    assert_eq!(tab.node_border_style, EdgeStyle::Dashed);
    assert_eq!(tab.node_border_styles.get(&vec![1]), Some(&EdgeStyle::Dotted));
    assert!(!tab.node_priorities.contains_key(&vec![0]));
    assert_eq!(tab.node_priorities.get(&vec![1]), Some(&9));
    assert!(!tab.node_urls.contains_key(&vec![0]));
    assert_eq!(tab.node_urls.get(&vec![1]).map(String::as_str), Some("https://example.test"));
    assert!(tab.collapsed_paths.contains(&vec![1]));
    assert_eq!(tab.background, Some(99));
    assert!(!tab.follow_theme_background);
    assert_eq!(tab.edge_style, EdgeStyle::Dotted);
    assert_eq!(tab.edge_styles.get(&vec![1]), Some(&EdgeStyle::Dashed));
    assert_eq!(tab.edge_colors.get(&vec![1]), Some(&13));
    assert_eq!(tab.doodle_rgba, 0x111827FF);
    assert_eq!(tab.doodle_width_px, 3.0);
    assert_eq!(tab.doodles.len(), 1);
    assert_eq!(tab.theme_group, "custom");
    assert_eq!(tab.theme_variant, 2);
    assert!(!tab.custom_themes.is_empty());
}

#[test]
fn json_restore_reports_parse_and_format_errors() {
    let mut app = empty_app();

    let parse_error = new_tab_from_json(&mut app, None, "{".to_string()).unwrap_err();
    assert!(parse_error.starts_with("解析 JSON 失败:"));

    let unsupported = serde_json::json!({
        "format": "wrong",
        "version": 2,
        "data": {"markdown": "", "pan_x": 0.0, "pan_y": 0.0, "zoom": 1.0}
    })
    .to_string();
    assert_eq!(
        new_tab_from_json(&mut app, None, unsupported).unwrap_err(),
        "不支持的思维导图 JSON 格式"
    );
}
