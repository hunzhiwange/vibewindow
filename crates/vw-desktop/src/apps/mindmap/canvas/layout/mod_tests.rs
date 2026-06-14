use super::compute_layout_for_diagram;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use std::collections::{HashMap, HashSet};

fn node(text: &str, children: Vec<MindNode>) -> MindNode {
    MindNode { text: text.to_string(), children }
}

fn sample_root() -> MindNode {
    node(
        "root",
        vec![
            node("a", vec![node("a1", Vec::new())]),
            node("b", vec![node("b1", Vec::new())]),
        ],
    )
}

fn dispatched(diagram_type: MindMapDiagramType) -> super::Layout {
    compute_layout_for_diagram(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        diagram_type,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    )
}

#[test]
fn dispatches_all_diagram_types_to_visible_layouts() {
    for diagram_type in [
        MindMapDiagramType::OrgChart,
        MindMapDiagramType::Fishbone,
        MindMapDiagramType::Tree,
        MindMapDiagramType::Timeline,
        MindMapDiagramType::MindMap,
        MindMapDiagramType::Bracket,
    ] {
        let layout = dispatched(diagram_type);
        assert_eq!(layout.nodes.len(), 5);
        assert_eq!(layout.edges.len(), 4);
        assert!(layout.nodes.iter().any(|n| n.path.is_empty()));
    }
}

#[test]
fn bracket_format_maps_to_left_or_right_mindmap_alignment() {
    let right = compute_layout_for_diagram(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        MindMapDiagramType::Bracket,
        MindMapLayoutFormat::Bidirectional,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    );
    let left = compute_layout_for_diagram(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        MindMapDiagramType::Bracket,
        MindMapLayoutFormat::Bidirectional,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceLeft,
        TreeLayoutFormat::FanDown,
    );

    assert!(right.nodes.iter().find(|n| n.path == vec![0]).unwrap().pos.x > 0.0);
    assert!(left.nodes.iter().find(|n| n.path == vec![0]).unwrap().pos.x < 0.0);
}

#[test]
fn selected_format_arguments_are_used_by_matching_diagram_type() {
    let fishbone_left = compute_layout_for_diagram(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        MindMapDiagramType::Fishbone,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadLeft,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    );
    let timeline_down = compute_layout_for_diagram(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        MindMapDiagramType::Timeline,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::AllDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    );

    assert!(fishbone_left.nodes.iter().find(|n| n.path == vec![0]).unwrap().pos.x > 0.0);
    assert!(timeline_down.nodes.iter().filter(|n| n.path.len() > 1).all(|n| n.pos.y > 0.0));
}
