use super::json_format::{
    MINDMAP_JSON_FORMAT, MindMapJsonFile, default_bracket_layout_format, default_diagram_type,
    default_edge_style, default_fishbone_layout_format, default_follow_theme_background,
    default_layout_format, default_org_chart_layout_format, default_theme_group,
    default_timeline_layout_format, default_tree_layout_format, tab_to_json,
};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapDiagramType, MindMapDoodleStroke,
    MindMapLayoutFormat, MindMapTab, OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use iced::Point;
use serde_json::json;

#[test]
fn json_format_defaults_are_stable() {
    assert_eq!(MINDMAP_JSON_FORMAT, "vibe-window-mindmap");
    assert_eq!(default_edge_style(), EdgeStyle::Solid);
    assert!(default_follow_theme_background());
    assert_eq!(default_theme_group(), "classic");
    assert_eq!(default_diagram_type(), MindMapDiagramType::default());
    assert_eq!(default_layout_format(), MindMapLayoutFormat::default());
    assert_eq!(default_org_chart_layout_format(), OrgChartLayoutFormat::default());
    assert_eq!(default_fishbone_layout_format(), FishboneLayoutFormat::default());
    assert_eq!(default_bracket_layout_format(), BracketLayoutFormat::default());
    assert_eq!(default_timeline_layout_format(), TimelineLayoutFormat::default());
    assert_eq!(default_tree_layout_format(), TreeLayoutFormat::default());
}

fn tab() -> MindMapTab {
    let doc = MindNode {
        text: "Root".to_string(),
        children: vec![MindNode { text: "Child".to_string(), children: Vec::new() }],
    };
    let mut tab = MindMapTab::new(
        "tab-1".to_string(),
        "Exported".to_string(),
        Some("/tmp/map.vwmm.json".to_string()),
        doc,
    );
    tab.diagram_type = MindMapDiagramType::Timeline;
    tab.layout_format = MindMapLayoutFormat::Bidirectional;
    tab.org_chart_layout_format = OrgChartLayoutFormat::LeftRight;
    tab.fishbone_layout_format = FishboneLayoutFormat::HeadLeft;
    tab.timeline_layout_format = TimelineLayoutFormat::AllDown;
    tab.bracket_layout_format = BracketLayoutFormat::BraceLeft;
    tab.tree_layout_format = TreeLayoutFormat::LeftAligned;
    tab.pan.x = 11.0;
    tab.pan.y = -22.0;
    tab.zoom = 2.5;
    tab.selected_path = Some(vec![0]);
    tab.node_positions.insert(vec![0], Point::new(1.0, 2.0));
    tab.node_fills.insert(vec![0], 0x11223344);
    tab.node_text_colors.insert(vec![0], 0x22334455);
    tab.node_border_colors.insert(vec![0], 0x33445566);
    tab.node_border_style = EdgeStyle::Dashed;
    tab.node_border_styles.insert(vec![0], EdgeStyle::Dotted);
    tab.node_priorities.insert(vec![0], 4);
    tab.node_urls.insert(vec![0], "https://example.com".to_string());
    tab.collapsed_paths.insert(vec![0, 1]);
    tab.background = Some(0x44556677);
    tab.follow_theme_background = false;
    tab.edge_style = EdgeStyle::Dotted;
    tab.edge_styles.insert(vec![0], EdgeStyle::Dashed);
    tab.edge_colors.insert(vec![0], 0x55667788);
    tab.doodle_rgba = 0x66778899;
    tab.doodle_width_px = 6.0;
    tab.doodles.push(MindMapDoodleStroke {
        points_world: vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
        rgba: 0x778899AA,
        width_px: 7.0,
    });
    tab.theme_group = "retro".to_string();
    tab.theme_variant = 2;
    tab
}

#[test]
fn tab_to_json_captures_complete_tab_state() {
    let tab = tab();

    let file = tab_to_json(&tab);

    assert_eq!(file.format, MINDMAP_JSON_FORMAT);
    assert_eq!(file.version, 1);
    assert_eq!(file.data.title.as_deref(), Some("Exported"));
    assert!(file.data.markdown.contains("Root"));
    assert!(file.data.markdown.contains("Child"));
    assert_eq!(file.data.diagram_type, MindMapDiagramType::Timeline);
    assert_eq!(file.data.layout_format, MindMapLayoutFormat::Bidirectional);
    assert_eq!(file.data.org_chart_layout_format, OrgChartLayoutFormat::LeftRight);
    assert_eq!(file.data.fishbone_layout_format, FishboneLayoutFormat::HeadLeft);
    assert_eq!(file.data.timeline_layout_format, TimelineLayoutFormat::AllDown);
    assert_eq!(file.data.bracket_layout_format, BracketLayoutFormat::BraceLeft);
    assert_eq!(file.data.tree_layout_format, TreeLayoutFormat::LeftAligned);
    assert_eq!(file.data.pan_x, 11.0);
    assert_eq!(file.data.pan_y, -22.0);
    assert_eq!(file.data.zoom, 2.5);
    assert_eq!(file.data.selected_path.as_deref(), Some(&[0][..]));
    assert_eq!(file.data.node_positions[0].path, vec![0]);
    assert_eq!(file.data.node_positions[0].x, 1.0);
    assert_eq!(file.data.node_fills[0].rgba, 0x11223344);
    assert_eq!(file.data.node_text_colors[0].rgba, 0x22334455);
    assert_eq!(file.data.node_border_colors[0].rgba, 0x33445566);
    assert_eq!(file.data.node_border_style, EdgeStyle::Dashed);
    assert_eq!(file.data.node_border_styles[0].style, EdgeStyle::Dotted);
    assert_eq!(file.data.node_priorities[0].priority, 4);
    assert_eq!(file.data.node_urls[0].url, "https://example.com");
    assert_eq!(file.data.collapsed_paths, vec![vec![0, 1]]);
    assert_eq!(file.data.background, Some(0x44556677));
    assert!(!file.data.follow_theme_background);
    assert_eq!(file.data.edge_style, EdgeStyle::Dotted);
    assert_eq!(file.data.edge_styles[0].style, EdgeStyle::Dashed);
    assert_eq!(file.data.edge_colors[0].rgba, 0x55667788);
    assert_eq!(file.data.doodle_rgba, 0x66778899);
    assert_eq!(file.data.doodle_width_px, 6.0);
    assert_eq!(file.data.doodles[0].points.len(), 2);
    assert_eq!(file.data.theme_group, "retro");
    assert_eq!(file.data.theme_variant, 2);
    assert_eq!(file.data.custom_themes.len(), tab.custom_themes.len());
}

#[test]
fn json_file_deserialization_applies_backward_compatible_defaults() {
    let file: MindMapJsonFile = serde_json::from_value(json!({
        "format": MINDMAP_JSON_FORMAT,
        "version": 1,
        "data": {
            "markdown": "# Root",
            "pan_x": 0.0,
            "pan_y": 0.0,
            "zoom": 1.0
        }
    }))
    .unwrap();

    assert_eq!(file.data.title, None);
    assert_eq!(file.data.diagram_type, MindMapDiagramType::default());
    assert_eq!(file.data.layout_format, MindMapLayoutFormat::default());
    assert_eq!(file.data.org_chart_layout_format, OrgChartLayoutFormat::default());
    assert_eq!(file.data.fishbone_layout_format, FishboneLayoutFormat::default());
    assert_eq!(file.data.timeline_layout_format, TimelineLayoutFormat::default());
    assert_eq!(file.data.bracket_layout_format, BracketLayoutFormat::default());
    assert_eq!(file.data.tree_layout_format, TreeLayoutFormat::default());
    assert_eq!(file.data.selected_path, None);
    assert!(file.data.node_positions.is_empty());
    assert!(file.data.node_fills.is_empty());
    assert!(file.data.node_text_colors.is_empty());
    assert!(file.data.node_border_colors.is_empty());
    assert_eq!(file.data.node_border_style, EdgeStyle::Solid);
    assert!(file.data.node_border_styles.is_empty());
    assert!(file.data.node_priorities.is_empty());
    assert!(file.data.node_urls.is_empty());
    assert!(file.data.collapsed_paths.is_empty());
    assert_eq!(file.data.background, None);
    assert!(file.data.follow_theme_background);
    assert_eq!(file.data.edge_style, EdgeStyle::Solid);
    assert!(file.data.edge_styles.is_empty());
    assert!(file.data.edge_colors.is_empty());
    assert_eq!(file.data.doodle_rgba, 0);
    assert_eq!(file.data.doodle_width_px, 0.0);
    assert!(file.data.doodles.is_empty());
    assert_eq!(file.data.theme_group, "classic");
    assert_eq!(file.data.theme_variant, 0);
    assert!(file.data.custom_themes.is_empty());
}
