#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("app_ui_tests"));
}

use super::*;
use crate::apps::workflow::model::WorkflowHandleSide;
use iced::Size;

#[test]
fn organize_active_app_separates_overlapped_connected_nodes() {
    let document = WorkflowDocument {
        nodes: vec![test_node("start", "start"), test_node("answer", "answer")],
        edges: vec![test_edge("start", "answer")],
        ..WorkflowDocument::default()
    };
    let snapshot = test_snapshot(&document);
    let mut state = WorkflowState {
        apps: vec![WorkflowAppEntry {
            id: "app".to_string(),
            local_uuid: None,
            meta: WorkflowAppMeta::default(),
            source_path: None,
            raw_root: Value::Null,
            document: document.clone(),
            environment_variables: Vec::new(),
            conversation_variables: Vec::new(),
            pan: Vector::new(0.0, 0.0),
            zoom: 1.0,
            selected_node_id: None,
            selected_edge_id: None,
            connection_draft: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            saved_snapshot: snapshot.clone(),
        }],
        active_app_id: Some("app".to_string()),
        document,
        saved_snapshot: Some(snapshot),
        ..WorkflowState::default()
    };

    state.organize_active_app((1280.0, 860.0)).expect("layout should succeed");

    let start = state.document.node("start").expect("start node should exist");
    let answer = state.document.node("answer").expect("answer node should exist");
    assert!(answer.position.x > start.position.x + start.size.width);
    assert_ne!(answer.position.y, 0.0);
    assert!(state.active_is_dirty);
    assert_eq!(state.undo_stack.len(), 1);
}

fn test_node(id: &str, block_type: &str) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: block_type.to_string(),
        title: id.to_string(),
        description: String::new(),
        position: Point::new(0.0, 0.0),
        size: Size::new(180.0, 120.0),
        parent_id: None,
        selected: false,
        source_side: WorkflowHandleSide::Right,
        target_side: WorkflowHandleSide::Left,
        source_handles: Vec::new(),
        target_handles: Vec::new(),
        z_index: 0.0,
        raw_node: Value::Null,
    }
}

fn test_edge(source: &str, target: &str) -> WorkflowEdge {
    WorkflowEdge {
        id: format!("{source}-{target}"),
        source: source.to_string(),
        target: target.to_string(),
        source_handle: Some("source".to_string()),
        target_handle: Some("target".to_string()),
        source_type: "start".to_string(),
        target_type: "answer".to_string(),
        selected: false,
        z_index: 0.0,
        raw_edge: Value::Null,
    }
}

fn test_snapshot(document: &WorkflowDocument) -> WorkflowHistorySnapshot {
    WorkflowHistorySnapshot {
        meta: WorkflowAppMeta::default(),
        document: document.clone(),
        environment_variables: Vec::new(),
        conversation_variables: Vec::new(),
        pan: Vector::new(0.0, 0.0),
        zoom: 1.0,
        selected_node_id: None,
        selected_edge_id: None,
    }
}
