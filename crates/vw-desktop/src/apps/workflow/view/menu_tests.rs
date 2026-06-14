//! 工作流菜单视图测试模块，覆盖菜单入口的元素构建与节点类型过滤分支。

use super::*;
use iced::Size;
use serde_yaml::Value;

use crate::apps::workflow::model::{
    WorkflowDocument, WorkflowHandleSide, WorkflowNodeIconDescriptor,
};

fn workflow_node(id: &str, block_type: &str) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: block_type.to_string(),
        title: "节点".to_string(),
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

#[test]
fn workflow_node_icon_badge_builds_known_icon_element() {
    let icon = workflow_node_icon("start");

    let _element = workflow_node_icon_badge(icon, workflow_node_accent_color("start"), 14.0);
}

#[test]
fn workflow_node_icon_badge_builds_fallback_dot_for_unknown_icon() {
    let icon = WorkflowNodeIconDescriptor { family: "missing-family", name: "missing-icon" };

    let _element = workflow_node_icon_badge(icon, workflow_node_accent_color("unknown"), 14.0);
}

#[test]
fn context_menu_content_button_builds_action_message() {
    let content: Element<'static, Message> = text("创建").into();

    let _element =
        context_menu_content_button(content, WorkflowMessage::CreateContextNode("llm".to_string()));
}

#[test]
fn context_menu_button_accepts_owned_label() {
    let _element = context_menu_button(
        String::from("删除"),
        WorkflowMessage::OpenCreateNodeEditor("code".to_string()),
    );
}

#[test]
fn context_node_picker_button_builds_create_context_node_action() {
    let node_type = supported_node_types()
        .iter()
        .copied()
        .find(|node_type| node_type.block_type == "llm")
        .expect("llm node type should be registered");

    let _element = context_node_picker_button(node_type);
}

#[test]
fn context_node_picker_menu_includes_start_when_state_has_no_start() {
    let state = WorkflowState::default();

    assert!(
        available_node_types(&state, false)
            .iter()
            .any(|node_type| { node_type.block_type == "start" })
    );

    let _element =
        build_context_node_picker_menu(&state, "添加节点", "选择要创建的节点类型。", false);
}

#[test]
fn context_node_picker_menu_excludes_start_when_requested() {
    let state = WorkflowState::default();

    assert!(
        !available_node_types(&state, true)
            .iter()
            .any(|node_type| { node_type.block_type == "start" })
    );

    let _element =
        build_context_node_picker_menu(&state, "添加下游节点", "选择下游节点类型。", true);
}

#[test]
fn context_node_picker_menu_excludes_start_when_document_already_has_start() {
    let state = WorkflowState {
        document: WorkflowDocument {
            nodes: vec![workflow_node("start_1", "start")],
            ..WorkflowDocument::default()
        },
        ..WorkflowState::default()
    };

    assert!(
        !available_node_types(&state, false)
            .iter()
            .any(|node_type| { node_type.block_type == "start" })
    );

    let _element = build_context_node_picker_menu(&state, "添加节点", "开始节点已存在。", false);
}

#[test]
fn error_banner_builds_dismissible_error_element() {
    let _element = error_banner("保存失败");
}
