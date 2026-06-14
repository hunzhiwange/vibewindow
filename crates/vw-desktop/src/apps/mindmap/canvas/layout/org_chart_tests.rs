use super::compute_org_chart_layout;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::OrgChartLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

fn node(text: &str, children: Vec<MindNode>) -> MindNode {
    MindNode { text: text.to_string(), children }
}

fn sample_root() -> MindNode {
    node(
        "root",
        vec![
            node("left", vec![node("leaf a", Vec::new()), node("leaf b", Vec::new())]),
            node("right", Vec::new()),
        ],
    )
}

#[test]
fn top_down_layout_centers_parent_over_children() {
    let layout = compute_org_chart_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        OrgChartLayoutFormat::TopDown,
    );

    let root = layout.nodes.iter().find(|n| n.path.is_empty()).unwrap();
    let left = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();
    let right = layout.nodes.iter().find(|n| n.path == vec![1]).unwrap();
    let leaf_a = layout.nodes.iter().find(|n| n.path == vec![0, 0]).unwrap();
    let leaf_b = layout.nodes.iter().find(|n| n.path == vec![0, 1]).unwrap();

    assert_eq!(layout.nodes.len(), 5);
    assert_eq!(layout.edges.len(), 4);
    assert_eq!(root.pos, Point::ORIGIN);
    assert!(left.pos.y > root.pos.y);
    assert!(right.pos.y > root.pos.y);
    assert!(leaf_a.pos.y > left.pos.y);
    assert!(leaf_a.pos.x < leaf_b.pos.x);
}

#[test]
fn left_right_currently_uses_same_top_down_walk() {
    let top_down = compute_org_chart_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        OrgChartLayoutFormat::TopDown,
    );
    let left_right = compute_org_chart_layout(
        &sample_root(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        OrgChartLayoutFormat::LeftRight,
    );

    assert_eq!(left_right.nodes.len(), top_down.nodes.len());
    assert_eq!(
        left_right
            .edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone()))
            .collect::<Vec<_>>(),
        top_down
            .edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left_right.nodes.iter().map(|n| n.pos).collect::<Vec<_>>(),
        top_down.nodes.iter().map(|n| n.pos).collect::<Vec<_>>()
    );
}

#[test]
fn collapsed_nodes_stop_recursive_layout_and_manual_positions_win() {
    let mut collapsed = HashSet::new();
    collapsed.insert(vec![0]);
    let mut positions = HashMap::new();
    positions.insert(vec![0], Point::new(77.0, 88.0));

    let layout = compute_org_chart_layout(
        &sample_root(),
        &positions,
        &HashMap::new(),
        &HashMap::new(),
        &collapsed,
        OrgChartLayoutFormat::TopDown,
    );

    assert_eq!(layout.nodes.len(), 3);
    assert!(layout.nodes.iter().any(|n| n.path == vec![0]));
    assert!(!layout.nodes.iter().any(|n| n.path == vec![0, 0]));
    assert_eq!(
        layout.nodes.iter().find(|n| n.path == vec![0]).unwrap().pos,
        Point::new(77.0, 88.0)
    );
}

#[test]
fn priority_and_url_metadata_increase_node_width() {
    let root = node("root", vec![node("decorated", Vec::new())]);
    let mut priorities = HashMap::new();
    priorities.insert(vec![0], 10);
    let mut urls = HashMap::new();
    urls.insert(vec![0], "https://example.com".to_string());

    let layout = compute_org_chart_layout(
        &root,
        &HashMap::new(),
        &priorities,
        &urls,
        &HashSet::new(),
        OrgChartLayoutFormat::TopDown,
    );
    let decorated = layout.nodes.iter().find(|n| n.path == vec![0]).unwrap();

    assert!(decorated.size.width > super::node_size("decorated", false, false, false).width);
}
