use super::compute_layout;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

fn node(text: &str, children: Vec<MindNode>) -> MindNode {
    MindNode { text: text.to_string(), children }
}

fn layout(format: MindMapLayoutFormat) -> super::Layout {
    let root = node(
        "root",
        vec![
            node("a", vec![node("a1", Vec::new())]),
            node("b", vec![node("b1", Vec::new())]),
        ],
    );
    compute_layout(
        &root,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        format,
    )
}

#[test]
fn right_and_left_aligned_layouts_mirror_child_x_positions() {
    let right = layout(MindMapLayoutFormat::RightAligned);
    let left = layout(MindMapLayoutFormat::LeftAligned);
    let right_child = right.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let left_child = left.nodes.iter().find(|n| n.path == vec![0]).unwrap();

    assert_eq!(right.nodes.len(), 5);
    assert_eq!(right.edges.len(), 4);
    assert!(right_child.pos.x > 0.0);
    assert!(left_child.pos.x < 0.0);
    assert_eq!(right_child.pos.x, -left_child.pos.x);
}

#[test]
fn bidirectional_layout_places_first_level_branches_on_both_sides() {
    let layout = layout(MindMapLayoutFormat::Bidirectional);
    let first = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let second = layout.nodes.iter().find(|n| n.path == vec![1]).unwrap();
    let first_grand = layout.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();
    let second_grand = layout.nodes.iter().find(|n| n.path == vec![1, 0]).unwrap();

    assert!(first.pos.x > 0.0);
    assert!(first_grand.pos.x > first.pos.x);
    assert!(second.pos.x < 0.0);
    assert!(second_grand.pos.x < second.pos.x);
}

#[test]
fn collapsed_paths_hide_descendants_and_manual_positions_win() {
    let root = node("root", vec![node("a", vec![node("hidden", Vec::new())])]);
    let mut collapsed = HashSet::new();
    collapsed.insert(vec![0]);
    let mut positions = HashMap::new();
    positions.insert(vec![0], Point::new(42.0, 24.0));

    let layout = compute_layout(
        &root,
        &positions,
        &HashMap::new(),
        &HashMap::new(),
        &collapsed,
        MindMapLayoutFormat::RightAligned,
    );

    assert_eq!(layout.nodes.len(), 2);
    assert_eq!(layout.edges.len(), 1);
    assert_eq!(layout.edges[0].from, Vec::<usize>::new());
    assert_eq!(layout.edges[0].to, vec![0]);
    assert_eq!(
        layout.nodes.iter().find(|n| n.path == vec![0]).unwrap().pos,
        Point::new(42.0, 24.0)
    );
}
