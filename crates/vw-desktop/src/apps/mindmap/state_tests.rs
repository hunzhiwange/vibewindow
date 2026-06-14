use super::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapColorPicker,
    MindMapColorTarget, MindMapDiagramType, MindMapDoodleStroke, MindMapLayoutFormat, MindMapTab,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use crate::app::components::mind_map::MindNode;
use crate::app::views::design::models::ColorFormat;
use iced::{Color, Point};

#[test]
fn layout_label_methods_cover_each_variant() {
    assert_eq!(MindMapLayoutFormat::RightAligned.label(), "右侧展开");
    assert_eq!(MindMapLayoutFormat::LeftAligned.label(), "左侧展开");
    assert_eq!(MindMapLayoutFormat::Bidirectional.label(), "双侧展开");
    assert_eq!(OrgChartLayoutFormat::TopDown.label(), "自上而下（曲线）");
    assert_eq!(OrgChartLayoutFormat::LeftRight.label(), "自上而下（折线）");
    assert_eq!(FishboneLayoutFormat::HeadRight.label(), "鱼头在右");
    assert_eq!(FishboneLayoutFormat::HeadLeft.label(), "鱼头在左");
    assert_eq!(BracketLayoutFormat::BraceRight.label(), "括号在右");
    assert_eq!(BracketLayoutFormat::BraceLeft.label(), "括号在左");
    assert_eq!(TimelineLayoutFormat::UpDown.label(), "上下");
    assert_eq!(TimelineLayoutFormat::AllUp.label(), "全上");
    assert_eq!(TimelineLayoutFormat::AllDown.label(), "全下");
    assert_eq!(TreeLayoutFormat::SymmetricSplit.label(), "左右对称分支");
    assert_eq!(TreeLayoutFormat::FanDown.label(), "顶部中心多分支");
    assert_eq!(TreeLayoutFormat::LeftAligned.label(), "左侧单边分支");
    assert_eq!(TreeLayoutFormat::RightAligned.label(), "右侧单边分支");
}

#[test]
fn diagram_type_label_methods_cover_each_variant() {
    assert_eq!(MindMapDiagramType::MindMap.label(), "思维导图");
    assert_eq!(MindMapDiagramType::OrgChart.label(), "组织结构图");
    assert_eq!(MindMapDiagramType::Fishbone.label(), "鱼骨图");
    assert_eq!(MindMapDiagramType::Timeline.label(), "时间轴");
    assert_eq!(MindMapDiagramType::Tree.label(), "树形图");
    assert_eq!(MindMapDiagramType::Bracket.label(), "括号图");
}

#[test]
fn enum_defaults_match_initial_editor_modes() {
    assert_eq!(MindMapDiagramType::default(), MindMapDiagramType::MindMap);
    assert_eq!(MindMapLayoutFormat::default(), MindMapLayoutFormat::RightAligned);
    assert_eq!(OrgChartLayoutFormat::default(), OrgChartLayoutFormat::TopDown);
    assert_eq!(FishboneLayoutFormat::default(), FishboneLayoutFormat::HeadRight);
    assert_eq!(BracketLayoutFormat::default(), BracketLayoutFormat::BraceRight);
    assert_eq!(TimelineLayoutFormat::default(), TimelineLayoutFormat::UpDown);
    assert_eq!(TreeLayoutFormat::default(), TreeLayoutFormat::FanDown);
}

#[test]
fn simple_state_structs_preserve_supplied_values() {
    let stroke = MindMapDoodleStroke {
        points_world: vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)],
        rgba: 0x11223344,
        width_px: 5.5,
    };
    assert_eq!(stroke.points_world, vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)]);
    assert_eq!(stroke.rgba, 0x11223344);
    assert_eq!(stroke.width_px, 5.5);

    let picker = MindMapColorPicker {
        color: Color::from_rgb(0.1, 0.2, 0.3),
        format: ColorFormat::Hex,
        target: MindMapColorTarget::EdgeStroke,
        picking: true,
    };
    assert_eq!(picker.color, Color::from_rgb(0.1, 0.2, 0.3));
    assert_eq!(picker.format, ColorFormat::Hex);
    assert_eq!(picker.target, MindMapColorTarget::EdgeStroke);
    assert!(picker.picking);
}

#[test]
fn mindmap_tab_new_initializes_canvas_and_layout_defaults() {
    let doc = MindNode {
        text: "Root".to_string(),
        children: vec![MindNode { text: "Child".to_string(), children: Vec::new() }],
    };

    let tab = MindMapTab::new(
        "tab-1".to_string(),
        "Planning".to_string(),
        Some("/tmp/plan.json".to_string()),
        doc,
    );

    assert_eq!(tab.id, "tab-1");
    assert_eq!(tab.title, "Planning");
    assert_eq!(tab.file_path.as_deref(), Some("/tmp/plan.json"));
    assert_eq!(tab.doc.text, "Root");
    assert_eq!(tab.doc.children.len(), 1);
    assert_eq!(tab.diagram_type, MindMapDiagramType::MindMap);
    assert_eq!(tab.layout_format, MindMapLayoutFormat::RightAligned);
    assert_eq!(tab.org_chart_layout_format, OrgChartLayoutFormat::TopDown);
    assert_eq!(tab.fishbone_layout_format, FishboneLayoutFormat::HeadRight);
    assert_eq!(tab.timeline_layout_format, TimelineLayoutFormat::UpDown);
    assert_eq!(tab.bracket_layout_format, BracketLayoutFormat::BraceRight);
    assert_eq!(tab.tree_layout_format, TreeLayoutFormat::FanDown);
    assert_eq!(tab.pan, iced::Vector::new(300.0, 200.0));
    assert_eq!(tab.zoom, 1.0);
    assert_eq!(tab.node_border_style, EdgeStyle::Solid);
    assert_eq!(tab.edge_style, EdgeStyle::Solid);
    assert_eq!(tab.canvas_tool, MindMapCanvasTool::Select);
    assert_eq!(tab.doodle_rgba, 0x111827FF);
    assert_eq!(tab.doodle_width_px, 3.0);
    assert_eq!(tab.theme_group, "classic");
    assert_eq!(tab.theme_variant, 0);
    assert!(!tab.custom_themes.is_empty());
}

#[test]
fn mindmap_tab_new_starts_with_empty_optional_and_collection_state() {
    let tab = MindMapTab::new(
        "tab-2".to_string(),
        "Blank".to_string(),
        None,
        MindNode { text: "Root".to_string(), children: Vec::new() },
    );

    assert!(tab.selected_path.is_none());
    assert!(tab.last_click_screen.is_none());
    assert!(tab.node_positions.is_empty());
    assert!(tab.node_fills.is_empty());
    assert!(tab.node_text_colors.is_empty());
    assert!(tab.node_border_colors.is_empty());
    assert!(tab.node_border_styles.is_empty());
    assert!(tab.node_priorities.is_empty());
    assert!(tab.node_urls.is_empty());
    assert!(tab.collapsed_paths.is_empty());
    assert!(tab.background.is_none());
    assert!(tab.follow_theme_background);
    assert!(tab.edge_styles.is_empty());
    assert!(tab.edge_colors.is_empty());
    assert!(tab.active_color_picker.is_none());
    assert!(!tab.show_diagram_type_picker);
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_export_menu);
    assert!(!tab.show_zoom_menu);
    assert!(!tab.show_priority_picker);
    assert!(!tab.show_url_editor);
    assert!(!tab.show_text_editor);
    assert!(!tab.show_action_menu);
    assert!(tab.url_editor_value.is_empty());
    assert!(tab.clipboard_node.is_none());
    assert!(!tab.show_context_menu);
    assert!(tab.context_menu_anchor.is_none());
    assert!(tab.undo_stack.is_empty());
    assert!(tab.redo_stack.is_empty());
    assert!(tab.doodles.is_empty());
    assert!(!tab.show_theme_panel);
}
