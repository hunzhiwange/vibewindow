use super::json_format::{
    MINDMAP_JSON_FORMAT, default_bracket_layout_format, default_diagram_type, default_edge_style,
    default_fishbone_layout_format, default_follow_theme_background, default_layout_format,
    default_org_chart_layout_format, default_theme_group, default_timeline_layout_format,
    default_tree_layout_format,
};
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};

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
