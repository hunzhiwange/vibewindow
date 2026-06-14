use std::collections::{HashMap, HashSet};

use iced::{Point, Size, Vector};

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};

fn sample_doc() -> MindNode {
    MindNode {
        text: "Root".to_string(),
        children: vec![
            MindNode { text: "Left".to_string(), children: Vec::new() },
            MindNode { text: "Right".to_string(), children: Vec::new() },
        ],
    }
}

#[test]
fn layout_node_rect_centers_size_around_position() {
    let node = super::NodeLayout {
        path: vec![1],
        text: "Node".to_string(),
        pos: Point::new(50.0, 80.0),
        size: Size::new(40.0, 20.0),
    };

    let rect = super::layout_node_rect(&node);

    assert_eq!(rect.x, 30.0);
    assert_eq!(rect.y, 70.0);
    assert_eq!(rect.width, 40.0);
    assert_eq!(rect.height, 20.0);
}

#[test]
fn layout_bounds_world_returns_zero_rect_for_empty_layout() {
    let bounds = super::layout_bounds_world(&super::Layout { nodes: Vec::new(), edges: Vec::new() });

    assert_eq!(bounds.x, 0.0);
    assert_eq!(bounds.y, 0.0);
    assert_eq!(bounds.width, 0.0);
    assert_eq!(bounds.height, 0.0);
}

#[test]
fn layout_bounds_world_encloses_all_node_rects() {
    let layout = super::Layout {
        nodes: vec![
            super::NodeLayout {
                path: vec![],
                text: "A".to_string(),
                pos: Point::new(10.0, 10.0),
                size: Size::new(20.0, 20.0),
            },
            super::NodeLayout {
                path: vec![0],
                text: "B".to_string(),
                pos: Point::new(60.0, 80.0),
                size: Size::new(10.0, 30.0),
            },
        ],
        edges: vec![super::EdgeLayout { from: vec![], to: vec![0] }],
    };

    let bounds = super::layout_bounds_world(&layout);

    assert_eq!(bounds.x, 0.0);
    assert_eq!(bounds.y, 0.0);
    assert_eq!(bounds.width, 65.0);
    assert_eq!(bounds.height, 95.0);
}

#[test]
fn selected_node_helpers_return_none_for_missing_path() {
    let doc = sample_doc();
    let positions = HashMap::new();
    let collapsed = HashSet::new();

    let top = super::selected_node_top_center_screen(
        &doc,
        &positions,
        &collapsed,
        Vector::new(0.0, 0.0),
        1.0,
        &[9],
        MindMapDiagramType::MindMap,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    );
    let rect = super::selected_node_rect_screen(
        &doc,
        &positions,
        &collapsed,
        Vector::new(0.0, 0.0),
        1.0,
        &[9],
        MindMapDiagramType::MindMap,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    );

    assert!(top.is_none());
    assert!(rect.is_none());
}

#[test]
fn selected_node_helpers_apply_pan_and_zoom_for_existing_path() {
    let doc = sample_doc();
    let positions = HashMap::new();
    let collapsed = HashSet::new();
    let pan = Vector::new(12.0, 34.0);
    let zoom = 1.5;

    let top = super::selected_node_top_center_screen(
        &doc,
        &positions,
        &collapsed,
        pan,
        zoom,
        &[],
        MindMapDiagramType::MindMap,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    )
    .expect("root top center should exist");
    let rect = super::selected_node_rect_screen(
        &doc,
        &positions,
        &collapsed,
        pan,
        zoom,
        &[],
        MindMapDiagramType::MindMap,
        MindMapLayoutFormat::RightAligned,
        OrgChartLayoutFormat::TopDown,
        FishboneLayoutFormat::HeadRight,
        TimelineLayoutFormat::UpDown,
        BracketLayoutFormat::BraceRight,
        TreeLayoutFormat::FanDown,
    )
    .expect("root rect should exist");

    assert!(top.x.is_finite());
    assert!(top.y.is_finite());
    assert_eq!(rect.width / zoom, rect.width / 1.5);
    assert!(rect.height > 0.0);
}
