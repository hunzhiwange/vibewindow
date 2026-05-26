use super::persist::{
    default_bracket_layout_format, default_edge_style, default_fishbone_layout_format,
    default_follow_theme_background, default_org_chart_layout_format, default_theme_group,
    default_tree_layout_format,
};
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, OrgChartLayoutFormat, TreeLayoutFormat,
};

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
