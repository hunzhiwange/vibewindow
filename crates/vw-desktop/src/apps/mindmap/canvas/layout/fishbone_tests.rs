use super::compute_fishbone_layout;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::FishboneLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

fn node(text: &str, children: Vec<MindNode>) -> MindNode {
    MindNode { text: text.to_string(), children }
}

fn sample_root() -> MindNode {
    node(
        "root",
        vec![
            node("cause a", vec![node("detail a", vec![node("deep a", Vec::new())])]),
            node("cause b", vec![node("detail b", Vec::new())]),
            node("cause c", Vec::new()),
        ],
    )
}

#[test]
fn head_right_and_head_left_mirror_primary_branch_direction() {
    let right = compute_fishbone_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        FishboneLayoutFormat::HeadRight,
    );
    let left = compute_fishbone_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        FishboneLayoutFormat::HeadLeft,
    );

    let right_branch = right.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let left_branch = left.nodes.iter().find(|n| n.path == vec![0]).unwrap();

    assert_eq!(right.nodes.len(), 7);
    assert_eq!(right.edges.len(), 6);
    assert!(right_branch.pos.x < 0.0);
    assert!(left_branch.pos.x > 0.0);
    assert_eq!(right_branch.pos.x, -left_branch.pos.x);
}

#[test]
fn branches_alternate_above_and_below_spine() {
    let layout = compute_fishbone_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        FishboneLayoutFormat::HeadRight,
    );

    let first = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let second = layout.nodes.iter().find(|n| n.path == vec![1]).unwrap();
    let first_detail = layout.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();
    let second_detail = layout.nodes.iter().find(|n| n.path == vec![1, 0]).unwrap();

    assert!(first.pos.y < 0.0);
    assert!(second.pos.y > 0.0);
    assert!(first_detail.pos.y < 0.0);
    assert!(second_detail.pos.y > 0.0);
}

#[test]
fn collapsed_root_or_branch_suppresses_descendants() {
    let mut collapsed_root = HashSet::new();
    collapsed_root.insert(Vec::<usize>::new());
    let root_only = compute_fishbone_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &collapsed_root,
        FishboneLayoutFormat::HeadRight,
    );
    assert_eq!(root_only.nodes.len(), 1);
    assert!(root_only.edges.is_empty());

    let mut collapsed_branch = HashSet::new();
    collapsed_branch.insert(vec![0]);
    let layout = compute_fishbone_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &collapsed_branch,
        FishboneLayoutFormat::HeadRight,
    );

    assert!(layout.nodes.iter().any(|n| n.path == vec![0]));
    assert!(!layout.nodes.iter().any(|n| n.path == vec![0, 0]));
    assert_eq!(layout.edges.iter().filter(|e| e.from == vec![0]).count(), 0);
}

#[test]
fn manual_positions_and_decorations_are_applied() {
    let mut positions = HashMap::new();
    positions.insert(vec![0, 0], Point::new(12.0, 34.0));
    let mut priorities = HashMap::new();
    priorities.insert(vec![0, 0], 5);
    let mut urls = HashMap::new();
    urls.insert(vec![0, 0], "https://example.com".to_string());

    let layout = compute_fishbone_layout(
        &sample_root(),
        &positions,
        &priorities,
        &urls,
        &HashSet::new(),
        FishboneLayoutFormat::HeadRight,
    );
    let detail = layout.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();

    assert_eq!(detail.pos, Point::new(12.0, 34.0));
    assert!(detail.size.width > super::node_size("detail a", false, false, false).width);
}
