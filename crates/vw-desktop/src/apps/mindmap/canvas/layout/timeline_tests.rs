use super::compute_timeline_layout;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::TimelineLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

fn node(text: &str, children: Vec<MindNode>) -> MindNode {
    MindNode { text: text.to_string(), children }
}

fn sample_root() -> MindNode {
    node(
        "root",
        vec![
            node("phase a", vec![node("task a1", vec![node("subtask", Vec::new())])]),
            node("phase b", vec![node("task b1", Vec::new())]),
        ],
    )
}

#[test]
fn up_down_alternates_descendant_direction_by_branch_index() {
    let layout = compute_timeline_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TimelineLayoutFormat::UpDown,
    );
    let first_task = layout.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();
    let second_task = layout.nodes.iter().find(|n| n.path == vec![1, 0]).unwrap();

    assert_eq!(layout.nodes.len(), 6);
    assert_eq!(layout.edges.len(), 5);
    assert!(first_task.pos.y < 0.0);
    assert!(second_task.pos.y > 0.0);
}

#[test]
fn all_up_and_all_down_force_descendant_direction() {
    let up = compute_timeline_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TimelineLayoutFormat::AllUp,
    );
    let down = compute_timeline_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        TimelineLayoutFormat::AllDown,
    );

    assert!(up.nodes.iter().filter(|n| n.path.len() > 1).all(|n| n.pos.y < 0.0));
    assert!(down.nodes.iter().filter(|n| n.path.len() > 1).all(|n| n.pos.y > 0.0));
}

#[test]
fn collapsed_root_or_branch_suppresses_later_nodes() {
    let mut collapsed_root = HashSet::new();
    collapsed_root.insert(Vec::<usize>::new());
    let root_only = compute_timeline_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &collapsed_root,
        TimelineLayoutFormat::UpDown,
    );
    assert_eq!(root_only.nodes.len(), 1);
    assert!(root_only.edges.is_empty());

    let mut collapsed_branch = HashSet::new();
    collapsed_branch.insert(vec![0]);
    let layout = compute_timeline_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &collapsed_branch,
        TimelineLayoutFormat::UpDown,
    );
    assert!(layout.nodes.iter().any(|n| n.path == vec![0]));
    assert!(!layout.nodes.iter().any(|n| n.path == vec![0, 0]));
}

#[test]
fn manual_positions_and_metadata_affect_layout_nodes() {
    let mut positions = HashMap::new();
    positions.insert(Vec::<usize>::new(), Point::new(5.0, 6.0));
    positions.insert(vec![0, 0], Point::new(44.0, -55.0));
    let mut priorities = HashMap::new();
    priorities.insert(vec![0], 1);
    let mut urls = HashMap::new();
    urls.insert(vec![0], "https://example.com".to_string());

    let layout = compute_timeline_layout(
        &sample_root(),
        &positions,
        &priorities,
        &urls,
        &HashSet::new(),
        TimelineLayoutFormat::UpDown,
    );
    let root = layout.nodes.iter().find(|n| n.path.is_empty()).unwrap();
    let first = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let task = layout.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();

    assert_eq!(root.pos, Point::new(5.0, 6.0));
    assert_eq!(task.pos, Point::new(44.0, -55.0));
    assert!(first.size.width > super::node_size("phase a", false, false, false).width);
}
