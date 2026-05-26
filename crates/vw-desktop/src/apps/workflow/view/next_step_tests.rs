//! 工作流下一步视图测试模块，覆盖失败分支能力从已保存节点和编辑草稿中推导的行为。

use super::*;
use iced::{Point, Size, widget::text_editor};
use serde_yaml::Value;

use crate::apps::workflow::model::{
    WorkflowDocument, WorkflowHandle, WorkflowHandleKind, WorkflowHandleSide,
};
use crate::apps::workflow::state::{
    WorkflowNodeEditorDraft, WorkflowNodeEditorValidation,
};

fn workflow_node(id: &str, block_type: &str, source_handles: Vec<WorkflowHandle>) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: block_type.to_string(),
        title: "代码节点".to_string(),
        description: String::new(),
        position: Point::new(0.0, 0.0),
        size: Size::new(180.0, 120.0),
        parent_id: None,
        selected: false,
        source_side: WorkflowHandleSide::Right,
        target_side: WorkflowHandleSide::Left,
        source_handles,
        target_handles: Vec::new(),
        z_index: 0.0,
        raw_node: Value::Null,
    }
}

fn code_editor_with_error_strategy(node_id: &str, error_strategy: &str) -> WorkflowNodeEditorDraft {
    WorkflowNodeEditorDraft {
        mode: WorkflowNodeEditorMode::Edit(node_id.to_string()),
        active_tab: WorkflowNodeEditorTab::Basic,
        block_type: "code".to_string(),
        title: "代码节点".to_string(),
        description: String::new(),
        description_editor: text_editor::Content::with_text(""),
        position: Point::new(0.0, 0.0),
        visual_draft: Some(WorkflowNodeVisualDraft::Code {
            language: "python3".to_string(),
            inputs: Vec::new(),
            code_editor: text_editor::Content::with_text("def main():\n    return {}\n"),
            outputs: Vec::new(),
            retry_config: WorkflowNodeRetryDraft {
                enabled: false,
                max_retries: 3,
                retry_interval: 500,
            },
            error_strategy: error_strategy.to_string(),
            default_value_editor: text_editor::Content::with_text("[]"),
        }),
        validation: WorkflowNodeEditorValidation::default(),
        show_raw_data_editor: false,
        raw_data_editor: text_editor::Content::with_text("{}"),
        hovered_start_variable_index: None,
        start_variable_focus_index: None,
        start_variable_editor: None,
    }
}

#[test]
fn next_step_supports_fail_branch_from_saved_node_handles() {
    let state = WorkflowState {
        document: WorkflowDocument {
            nodes: vec![workflow_node(
                "code_1",
                "code",
                vec![WorkflowHandle {
                    id: "fail-branch".to_string(),
                    label: "异常".to_string(),
                    kind: WorkflowHandleKind::Source,
                }],
            )],
            ..WorkflowDocument::default()
        },
        ..WorkflowState::default()
    };
    let editor = code_editor_with_error_strategy("code_1", "none");

    assert!(node_next_step_supports_fail_branch(&state, &editor, "code_1"));
}

#[test]
fn next_step_supports_fail_branch_from_unsaved_code_editor_draft() {
    let state = WorkflowState {
        document: WorkflowDocument {
            nodes: vec![workflow_node("code_1", "code", Vec::new())],
            ..WorkflowDocument::default()
        },
        ..WorkflowState::default()
    };
    let editor = code_editor_with_error_strategy("code_1", "fail-branch");

    assert!(node_next_step_supports_fail_branch(&state, &editor, "code_1"));
}
