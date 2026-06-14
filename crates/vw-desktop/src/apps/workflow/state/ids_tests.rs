#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("ids_tests"));
}

use super::*;
use crate::apps::workflow::model::{WorkflowHandleSide, WorkflowViewport};
use iced::Size;

fn node_at(id: &str, position: Point, size: Size) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: "answer".to_string(),
        title: id.to_string(),
        description: String::new(),
        position,
        size,
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

#[test]
fn fitted_viewport_returns_default_for_empty_document() {
    let (pan, zoom) = fitted_viewport(&WorkflowDocument::default(), (800.0, 600.0));

    assert_eq!(pan, Vector::new(120.0, 120.0));
    assert_eq!(zoom, 1.0);
}

#[test]
fn fitted_viewport_centers_document_and_clamps_zoom() {
    let document = WorkflowDocument {
        nodes: vec![node_at("small", Point::new(0.0, 0.0), Size::new(100.0, 100.0))],
        viewport: WorkflowViewport::default(),
        ..WorkflowDocument::default()
    };
    let (pan, zoom) = fitted_viewport(&document, (5000.0, 5000.0));

    assert_eq!(zoom, 4.0);
    assert_eq!(pan, Vector::new(2134.0, 2214.0));

    let large_document = WorkflowDocument {
        nodes: vec![node_at("large", Point::new(-5000.0, -5000.0), Size::new(10_000.0, 10_000.0))],
        viewport: WorkflowViewport::default(),
        ..WorkflowDocument::default()
    };
    let (_, small_zoom) = fitted_viewport(&large_document, (0.0, 0.0));
    assert_eq!(small_zoom, 0.1);
}

#[test]
fn generated_runtime_ids_use_expected_prefixes_and_sanitized_type_segments() {
    assert!(generate_app_id().starts_with("workflow-app-"));
    assert!(generate_node_id("if-else").starts_with("if_else-node-"));
    assert!(generate_variable_id("env secret").starts_with("env_secret-var-"));
    assert!(generate_start_variable_name().starts_with("input_"));
    assert!(generate_case_id().starts_with("case-"));
    assert!(generate_condition_id().starts_with("condition-"));
    assert!(generate_prompt_item_id("system-prompt").starts_with("system_prompt-prompt-"));
}

#[test]
fn normalize_connection_endpoints_accepts_either_order_and_rejects_same_kind() {
    let source = WorkflowConnectionEndpoint {
        node_id: "source".to_string(),
        handle_id: "out".to_string(),
        kind: WorkflowHandleKind::Source,
    };
    let target = WorkflowConnectionEndpoint {
        node_id: "target".to_string(),
        handle_id: "in".to_string(),
        kind: WorkflowHandleKind::Target,
    };

    let normalized = normalize_connection_endpoints(&source, &target).expect("source to target");
    assert_eq!(normalized.0.node_id, "source");
    assert_eq!(normalized.1.node_id, "target");

    let reversed = normalize_connection_endpoints(&target, &source).expect("target to source");
    assert_eq!(reversed.0.node_id, "source");
    assert_eq!(reversed.1.node_id, "target");

    assert!(normalize_connection_endpoints(&source, &source).is_none());
    assert!(normalize_connection_endpoints(&target, &target).is_none());
}

#[test]
fn generate_edge_id_and_sanitize_handle_id_replace_non_ascii_alphanumeric_chars() {
    let source = WorkflowConnectionEndpoint {
        node_id: "source-node".to_string(),
        handle_id: "out-1".to_string(),
        kind: WorkflowHandleKind::Source,
    };
    let target = WorkflowConnectionEndpoint {
        node_id: "target-node".to_string(),
        handle_id: "in.2".to_string(),
        kind: WorkflowHandleKind::Target,
    };

    let edge_id = generate_edge_id(&source, &target);

    assert!(edge_id.starts_with("manual-source-node-out_1-target-node-in_2-"));
    assert_eq!(sanitize_handle_id("A-b_中 1"), "A_b___1");
}
