use serde_yaml::Value;

use super::*;

fn handle(kind: WorkflowHandleKind, id: &str, label: &str) -> WorkflowHandle {
    WorkflowHandle { id: id.to_string(), label: label.to_string(), kind }
}

fn node(id: &str, position: Point, z_index: f32) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: "llm".to_string(),
        title: id.to_string(),
        description: String::new(),
        position,
        size: Size::new(120.0, 80.0),
        parent_id: None,
        selected: false,
        source_side: WorkflowHandleSide::Right,
        target_side: WorkflowHandleSide::Left,
        source_handles: vec![
            handle(WorkflowHandleKind::Source, "source", "默认"),
            handle(WorkflowHandleKind::Source, "success", "成功"),
        ],
        target_handles: vec![handle(WorkflowHandleKind::Target, "target", "输入")],
        z_index,
        raw_node: Value::Null,
    }
}

fn edge(id: &str, source_handle: Option<&str>, target_handle: Option<&str>) -> WorkflowEdge {
    WorkflowEdge {
        id: id.to_string(),
        source: "a".to_string(),
        target: "b".to_string(),
        source_handle: source_handle.map(str::to_string),
        target_handle: target_handle.map(str::to_string),
        source_type: "llm".to_string(),
        target_type: "answer".to_string(),
        selected: false,
        z_index: 0.0,
        raw_edge: Value::Null,
    }
}

fn document() -> WorkflowDocument {
    WorkflowDocument {
        nodes: vec![
            node("a", Point::new(10.0, 20.0), 0.0),
            node("b", Point::new(240.0, 20.0), 1.0),
        ],
        edges: vec![edge("e1", Some("true"), None), edge("e2", None, Some("alternate"))],
        ..WorkflowDocument::default()
    }
}

#[test]
fn handle_slots_merge_declared_and_edge_handles() {
    let document = document();
    let slots = build_handle_slots(&document);
    let source_node = &document.nodes[0];
    let target_node = &document.nodes[1];

    assert_eq!(
        anchor_for_handle(
            source_node,
            WorkflowHandleKind::Source,
            "source",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        ),
        Point::new(130.0, 40.0)
    );
    assert_eq!(
        anchor_for_handle(
            source_node,
            WorkflowHandleKind::Source,
            "success",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        ),
        Point::new(130.0, 60.0)
    );
    assert_eq!(
        anchor_for_handle(
            source_node,
            WorkflowHandleKind::Source,
            "true",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        ),
        Point::new(130.0, 80.0)
    );

    let alternate = anchor_for_handle(
        target_node,
        WorkflowHandleKind::Target,
        "alternate",
        &slots,
        Vector::new(0.0, 0.0),
        1.0,
    );
    assert!((alternate.y - 46.666668).abs() < 0.001);

    assert_eq!(
        anchor_for_handle(
            source_node,
            WorkflowHandleKind::Source,
            "missing",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        ),
        Point::new(130.0, 60.0)
    );
}

#[test]
fn node_rect_and_handle_bounds_apply_pan_and_zoom() {
    let document = document();
    let slots = build_handle_slots(&document);
    let node = &document.nodes[0];
    let rect = node_screen_rect(node, Vector::new(5.0, -10.0), 2.0);

    assert_eq!(rect.x, 25.0);
    assert_eq!(rect.y, 30.0);
    assert_eq!(rect.width, 240.0);
    assert_eq!(rect.height, 160.0);

    let bounds = handle_bounds(node, &node.source_handles[0], &slots, Vector::new(5.0, -10.0), 2.0);
    assert!(bounds.width >= 8.0);
    assert!(bounds.height >= 8.0);
}

#[test]
fn anchors_cover_each_handle_side() {
    let mut doc = document();
    let slots = build_handle_slots(&doc);
    let node = &mut doc.nodes[0];

    node.source_side = WorkflowHandleSide::Right;
    assert_eq!(
        anchor_for_handle(
            node,
            WorkflowHandleKind::Source,
            "source",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        )
        .x,
        130.0
    );
    node.source_side = WorkflowHandleSide::Left;
    assert_eq!(
        anchor_for_handle(
            node,
            WorkflowHandleKind::Source,
            "source",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        )
        .x,
        10.0
    );
    node.source_side = WorkflowHandleSide::Top;
    assert_eq!(
        anchor_for_handle(
            node,
            WorkflowHandleKind::Source,
            "source",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        )
        .y,
        20.0
    );
    node.source_side = WorkflowHandleSide::Bottom;
    assert_eq!(
        anchor_for_handle(
            node,
            WorkflowHandleKind::Source,
            "source",
            &slots,
            Vector::new(0.0, 0.0),
            1.0
        )
        .y,
        100.0
    );
}

#[test]
fn control_points_follow_handle_side() {
    let p = Point::new(50.0, 60.0);

    assert_eq!(control_for_side(p, WorkflowHandleSide::Left, 12.0), Point::new(38.0, 60.0));
    assert_eq!(control_for_side(p, WorkflowHandleSide::Right, 12.0), Point::new(62.0, 60.0));
    assert_eq!(control_for_side(p, WorkflowHandleSide::Top, 12.0), Point::new(50.0, 48.0));
    assert_eq!(control_for_side(p, WorkflowHandleSide::Bottom, 12.0), Point::new(50.0, 72.0));
}

#[test]
fn edge_handle_label_maps_and_filters_labels() {
    assert_eq!(edge_handle_label(&edge("true", Some("true"), None)), Some("是".to_string()));
    assert_eq!(edge_handle_label(&edge("false", Some("false"), None)), Some("否".to_string()));
    assert_eq!(
        edge_handle_label(&edge("custom", Some("branch"), None)),
        Some("branch".to_string())
    );
    assert_eq!(edge_handle_label(&edge("default", Some("source"), None)), None);
    assert_eq!(edge_handle_label(&edge("empty", Some("   "), None)), None);
    assert_eq!(edge_handle_label(&edge("dash", Some("branch-a"), None)), None);
    assert_eq!(edge_handle_label(&edge("long", Some("abcdefghijklmnopqrstuvwxyz"), None)), None);
    assert_eq!(edge_handle_label(&edge("none", None, None)), None);
}

#[test]
fn connected_handles_collect_defaults_and_explicit_ids() {
    let doc = document();
    let (sources, targets) = connected_handles(&doc);

    assert!(sources["a"].contains("true"));
    assert!(sources["a"].contains("source"));
    assert!(targets["b"].contains("target"));
    assert!(targets["b"].contains("alternate"));
}

#[test]
fn bezier_hit_test_detects_near_and_far_points() {
    let start = Point::new(0.0, 0.0);
    let c1 = Point::new(50.0, 0.0);
    let c2 = Point::new(50.0, 100.0);
    let end = Point::new(100.0, 100.0);

    assert!(bezier_hit_test(Point::new(50.0, 50.0), start, c1, c2, end, 30.0));
    assert!(!bezier_hit_test(Point::new(200.0, 200.0), start, c1, c2, end, 5.0));
}
