use super::compute_tree_layout;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::TreeLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

fn node(text: &str, children: Vec<MindNode>) -> MindNode {
    MindNode { text: text.to_string(), children }
}

fn sample_root() -> MindNode {
    node(
        "root",
        vec![
            node("a", vec![node("a1", Vec::new()), node("a2", Vec::new())]),
            node("b", vec![node("b1", Vec::new())]),
            node("c", Vec::new()),
        ],
    )
}

#[test]
fn fan_down_layout_places_children_below_root_and_builds_edges() {
    let layout = compute_tree_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TreeLayoutFormat::FanDown,
    );
    let root = layout.nodes.iter().find(|n| n.path.is_empty()).unwrap();
    let first = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let second = layout.nodes.iter().find(|n| n.path == vec![1]).unwrap();

    assert_eq!(layout.nodes.len(), 7);
    assert_eq!(layout.edges.len(), 6);
    assert!(first.pos.y > root.pos.y);
    assert!(first.pos.x < second.pos.x);
}

#[test]
fn symmetric_split_places_odd_and_even_branches_on_opposite_sides() {
    let layout = compute_tree_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TreeLayoutFormat::SymmetricSplit,
    );
    let first = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let second = layout.nodes.iter().find(|n| n.path == vec![1]).unwrap();
    let third = layout.nodes.iter().find(|n| n.path == vec![2]).unwrap();

    assert!(first.pos.x > 0.0);
    assert!(third.pos.x > first.pos.x);
    assert!(second.pos.x < 0.0);
}

#[test]
fn spine_layouts_move_children_left_or_right() {
    let left = compute_tree_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TreeLayoutFormat::LeftAligned,
    );
    let right = compute_tree_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TreeLayoutFormat::RightAligned,
    );
    let left_child = left.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let right_child = right.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let right_grand = right.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();

    assert!(left_child.pos.x < 0.0);
    assert!(right_child.pos.x > 0.0);
    assert!(right_grand.pos.x > right_child.pos.x);
}

#[test]
fn collapsed_root_or_child_stops_descendant_layout() {
    let mut collapsed_root = HashSet::new();
    collapsed_root.insert(Vec::<usize>::new());
    let root_only = compute_tree_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &collapsed_root,
        TreeLayoutFormat::FanDown,
    );
    assert_eq!(root_only.nodes.len(), 1);
    assert!(root_only.edges.is_empty());

    let mut collapsed_child = HashSet::new();
    collapsed_child.insert(vec![0]);
    let layout = compute_tree_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &collapsed_child,
        TreeLayoutFormat::FanDown,
    );
    assert!(layout.nodes.iter().any(|n| n.path == vec![0]));
    assert!(!layout.nodes.iter().any(|n| n.path == vec![0, 0]));
}

#[test]
fn manual_positions_and_metadata_are_respected() {
    let mut positions = HashMap::new();
    positions.insert(vec![0], Point::new(11.0, 22.0));
    let mut priorities = HashMap::new();
    priorities.insert(vec![0], 7);
    let mut urls = HashMap::new();
    urls.insert(vec![0], "https://example.com".to_string());

    let layout = compute_tree_layout(
        &sample_root(),
        &positions,
        &priorities,
        &urls,
        &HashSet::new(),
        TreeLayoutFormat::FanDown,
    );
    let first = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();

    assert_eq!(first.pos, Point::new(11.0, 22.0));
    assert!(first.size.width > super::node_size("a", false, false, false).width);
}
