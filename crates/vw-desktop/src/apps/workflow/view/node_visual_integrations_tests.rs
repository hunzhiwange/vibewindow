//! 工作流集成节点视图测试模块，覆盖代码节点变量引用候选项的筛选与回退。

use super::*;
use iced::{Point, Size, Vector, widget::text_editor};
use serde_yaml::Value;

use crate::apps::workflow::model::{
    WorkflowAppMeta, WorkflowConversationVariable, WorkflowDocument, WorkflowEnvironmentVariable,
    WorkflowHandleSide, WorkflowViewport,
};
use crate::apps::workflow::state::{
    WorkflowAppEntry, WorkflowHistorySnapshot, WorkflowNodeEditorDraft,
    WorkflowNodeEditorValidation,
};

fn workflow_node(id: &str, block_type: &str, title: &str, raw_yaml: &str) -> WorkflowNode {
    WorkflowNode {
        id: id.to_string(),
        block_type: block_type.to_string(),
        title: title.to_string(),
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
        raw_node: serde_yaml::from_str::<Value>(raw_yaml).expect("node yaml should parse"),
    }
}

fn minimal_code_editor(node_id: &str) -> WorkflowNodeEditorDraft {
    WorkflowNodeEditorDraft {
        mode: WorkflowNodeEditorMode::Edit(node_id.to_string()),
        active_tab: WorkflowNodeEditorTab::Visual,
        block_type: "code".to_string(),
        title: "代码节点".to_string(),
        description: String::new(),
        description_editor: text_editor::Content::with_text(""),
        position: Point::new(0.0, 0.0),
        visual_draft: None,
        validation: WorkflowNodeEditorValidation::default(),
        show_raw_data_editor: false,
        raw_data_editor: text_editor::Content::with_text("{}"),
        hovered_start_variable_index: None,
        start_variable_focus_index: None,
        start_variable_editor: None,
    }
}

fn workflow_state_with_active_app(
    document: WorkflowDocument,
    environment_variables: Vec<WorkflowEnvironmentVariable>,
    conversation_variables: Vec<WorkflowConversationVariable>,
) -> WorkflowState {
    let meta = WorkflowAppMeta::default();
    let saved_snapshot = WorkflowHistorySnapshot {
        meta: meta.clone(),
        document: document.clone(),
        environment_variables: environment_variables.clone(),
        conversation_variables: conversation_variables.clone(),
        pan: Vector::new(0.0, 0.0),
        zoom: 1.0,
        selected_node_id: None,
        selected_edge_id: None,
    };

    WorkflowState {
        apps: vec![WorkflowAppEntry {
            id: "app_1".to_string(),
            local_uuid: None,
            meta,
            source_path: None,
            raw_root: Value::Null,
            document: document.clone(),
            environment_variables: environment_variables.clone(),
            conversation_variables: conversation_variables.clone(),
            pan: Vector::new(0.0, 0.0),
            zoom: 1.0,
            selected_node_id: None,
            selected_edge_id: None,
            connection_draft: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            saved_snapshot: saved_snapshot.clone(),
        }],
        active_app_id: Some("app_1".to_string()),
        saved_apps: Vec::new(),
        saved_apps_loading: false,
        saved_apps_loaded: false,
        saved_apps_error: None,
        opening_saved_app_uuid: None,
        deleting_saved_app_uuid: None,
        confirm_delete_saved_app_uuid: None,
        saved_app_actions_menu_uuid: None,
        saved_app_search_query: String::new(),
        copied_saved_app_uuid: None,
        active_is_dirty: false,
        app_editor: None,
        node_editor: None,
        variable_panel: None,
        variable_editor: None,
        context_menu: None,
        quick_insert_panel_open: false,
        action_menu_open: false,
        zoom_menu_open: false,
        source_name: "测试工作流".to_string(),
        local_uuid: None,
        source_path: None,
        document,
        environment_variables,
        conversation_variables,
        pan: Vector::new(0.0, 0.0),
        zoom: 1.0,
        selected_node_id: None,
        selected_edge_id: None,
        connection_draft: None,
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
        saved_snapshot: Some(saved_snapshot),
        dragging_node_id: None,
        drag_pending_snapshot: None,
        status_message: None,
        error_message: None,
    }
}

#[test]
fn selected_code_variable_reference_option_falls_back_to_selector_when_missing() {
    let input = WorkflowCodeVariableDraft {
        variable: String::new(),
        value_type: "number".to_string(),
        selector: vec!["start".to_string(), "limit".to_string()],
    };

    let selected = selected_code_variable_reference_option(&input, &[])
        .expect("missing selector should still build fallback option");

    assert_eq!(selected.selector_key, "start.limit");
    assert_eq!(selected.label, "start.limit");
    assert_eq!(selected.value_type, "number");
}

#[test]
fn code_variable_reference_options_include_upstream_and_context_sources_only() {
    let start_node = workflow_node(
        "start",
        "start",
        "开始",
        r#"
data:
  variables:
    - variable: query
      type: paragraph
    - variable: uploads
      type: file-list
"#,
    );
    let llm_node = workflow_node(
        "llm_1",
        "llm",
        "LLM",
        r#"
data:
  outputs:
    text:
      type: string
      children: null
    score:
      type: number
      children: null
"#,
    );
    let tool_node = workflow_node(
        "tool_1",
        "tool",
        "工具",
        r#"
data:
  outputs:
    payload:
      type: object
      children: null
"#,
    );
    let code_node = workflow_node(
        "code_1",
        "code",
        "代码",
        r#"
data:
  code_language: python3
  code: |
    def main():
      return {}
"#,
    );

    let document = WorkflowDocument {
        name: "测试工作流".to_string(),
        nodes: vec![start_node, llm_node, tool_node, code_node],
        edges: vec![
            WorkflowEdge {
                id: "edge_start_llm".to_string(),
                source: "start".to_string(),
                target: "llm_1".to_string(),
                source_handle: None,
                target_handle: None,
                source_type: "start".to_string(),
                target_type: "llm".to_string(),
                selected: false,
                z_index: 0.0,
                raw_edge: Value::Null,
            },
            WorkflowEdge {
                id: "edge_llm_code".to_string(),
                source: "llm_1".to_string(),
                target: "code_1".to_string(),
                source_handle: None,
                target_handle: None,
                source_type: "llm".to_string(),
                target_type: "code".to_string(),
                selected: false,
                z_index: 0.0,
                raw_edge: Value::Null,
            },
        ],
        viewport: WorkflowViewport::default(),
    };
    let state = workflow_state_with_active_app(
        document,
        vec![WorkflowEnvironmentVariable {
            id: "env_api_key".to_string(),
            name: "api_key".to_string(),
            value_type: "string".to_string(),
            value: Value::String("secret".to_string()),
            description: String::new(),
            raw_variable: Value::Null,
        }],
        vec![WorkflowConversationVariable {
            id: "conv_thread_id".to_string(),
            name: "thread_id".to_string(),
            value_type: "string".to_string(),
            value: Value::String("thread-1".to_string()),
            description: String::new(),
            raw_variable: Value::Null,
        }],
    );
    let editor = minimal_code_editor("code_1");

    let options = build_code_variable_reference_options(&state, &editor);
    let selector_keys =
        options.iter().map(|option| option.selector_key.as_str()).collect::<Vec<_>>();

    assert!(selector_keys.contains(&"start.query"));
    assert!(selector_keys.contains(&"start.uploads"));
    assert!(selector_keys.contains(&"llm_1.text"));
    assert!(selector_keys.contains(&"llm_1.score"));
    assert!(!selector_keys.contains(&"tool_1.payload"));
    assert!(selector_keys.contains(&"env.api_key"));
    assert!(selector_keys.contains(&"conversation.thread_id"));
    assert!(selector_keys.contains(&"sys.user_id"));
    assert!(selector_keys.contains(&"sys.conversation_id"));

    let uploads = options
        .iter()
        .find(|option| option.selector_key == "start.uploads")
        .expect("start file-list variable should be available");
    assert_eq!(uploads.value_type, "array[file]");

    let llm_text = options
        .iter()
        .find(|option| option.selector_key == "llm_1.text")
        .expect("llm output should be available");
    assert_eq!(llm_text.label, "LLM · text");
    assert_eq!(llm_text.value_type, "string");
}
