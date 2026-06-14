use iced::Point;

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapDiagramType, MindMapDoodleStroke,
    MindMapTab, OrgChartLayoutFormat,
};

fn doc() -> MindNode {
    MindNode {
        text: "Root & <main>\nline".to_string(),
        children: vec![
            MindNode {
                text: "Alpha".to_string(),
                children: vec![MindNode { text: "Alpha child".to_string(), children: Vec::new() }],
            },
            MindNode { text: "Beta \"quoted\"".to_string(), children: Vec::new() },
        ],
    }
}

fn tab() -> MindMapTab {
    MindMapTab::new("tab-1".to_string(), "Mind map".to_string(), None, doc())
}

#[test]
fn export_svg_draws_mindmap_nodes_edges_decorations_and_doodles() {
    let mut tab = tab();
    tab.zoom = 0.05;
    tab.background = Some(0x11223380);
    tab.follow_theme_background = false;
    tab.node_priorities.insert(vec![], 10);
    tab.node_priorities.insert(vec![0], 3);
    tab.node_urls.insert(vec![1], "https://example.com".to_string());
    tab.node_fills.insert(vec![0], 0xFF0000FF);
    tab.node_text_colors.insert(vec![0], 0x00FF00FF);
    tab.node_border_colors.insert(vec![0], 0x0000FFFF);
    tab.node_border_style = EdgeStyle::Dashed;
    tab.node_border_styles.insert(vec![1], EdgeStyle::Dotted);
    tab.edge_style = EdgeStyle::Dotted;
    tab.edge_colors.insert(vec![0], 0xABCDEFCC);
    tab.edge_styles.insert(vec![1], EdgeStyle::Dashed);
    tab.doodles.push(MindMapDoodleStroke {
        points_world: vec![Point::new(-20.0, -10.0)],
        rgba: 0x123456FF,
        width_px: 2.0,
    });
    tab.doodles.push(MindMapDoodleStroke {
        points_world: vec![Point::new(0.0, 0.0), Point::new(12.0, 24.0)],
        rgba: 0x123456FF,
        width_px: 99.0,
    });

    let svg = super::export_svg(&tab);

    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("rgba(17,34,51,0.502)"));
    assert!(svg.contains("Root &amp; &lt;main&gt;"));
    assert!(svg.contains("Beta &quot;quoted&quot;"));
    assert!(svg.contains("stroke-dasharray="));
    assert!(svg.contains("stroke-linecap=\"round\""));
    assert!(svg.contains("<circle "));
    assert!(svg.contains("scale("));
    assert!(svg.contains("stroke-width=\"18.00\""));
    assert!(svg.ends_with("</g></svg>"));
}

#[test]
fn export_svg_draws_org_chart_curve_and_elbow_edges() {
    let mut top_down = tab();
    top_down.diagram_type = MindMapDiagramType::OrgChart;
    top_down.org_chart_layout_format = OrgChartLayoutFormat::TopDown;
    let top_down_svg = super::export_svg(&top_down);

    let mut left_right = tab();
    left_right.diagram_type = MindMapDiagramType::OrgChart;
    left_right.org_chart_layout_format = OrgChartLayoutFormat::LeftRight;
    let left_right_svg = super::export_svg(&left_right);

    assert!(top_down_svg.contains(" C "));
    assert!(left_right_svg.contains(" L "));
}

#[test]
fn export_svg_draws_fishbone_for_both_head_directions() {
    let mut right = tab();
    right.diagram_type = MindMapDiagramType::Fishbone;
    right.fishbone_layout_format = FishboneLayoutFormat::HeadRight;
    let right_svg = super::export_svg(&right);

    let mut left = tab();
    left.diagram_type = MindMapDiagramType::Fishbone;
    left.fishbone_layout_format = FishboneLayoutFormat::HeadLeft;
    let left_svg = super::export_svg(&left);

    assert!(right_svg.contains("<polygon points="));
    assert!(right_svg.contains("<line "));
    assert!(left_svg.contains("<polygon points="));
    assert!(left_svg.contains("<line "));
}

#[test]
fn export_svg_draws_brackets_for_multiple_children_and_line_for_single_child() {
    let mut right = tab();
    right.diagram_type = MindMapDiagramType::Bracket;
    right.bracket_layout_format = BracketLayoutFormat::BraceRight;
    right.edge_style = EdgeStyle::Dashed;
    let right_svg = super::export_svg(&right);

    let mut left = tab();
    left.diagram_type = MindMapDiagramType::Bracket;
    left.bracket_layout_format = BracketLayoutFormat::BraceLeft;
    left.edge_styles.insert(vec![0], EdgeStyle::Dotted);
    let left_svg = super::export_svg(&left);

    assert!(right_svg.contains("<path d=\"M "));
    assert!(right_svg.contains("<line "));
    assert!(left_svg.contains("<path d=\"M "));
    assert!(left_svg.contains("stroke-dasharray="));
}
