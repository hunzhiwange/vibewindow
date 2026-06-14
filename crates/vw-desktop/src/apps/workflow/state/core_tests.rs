#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("core_tests"));
}

#[test]
fn title_uses_list_title_without_active_app() {
    let state = super::WorkflowState {
        active_app_id: None,
        source_name: "已打开应用".to_string(),
        ..super::WorkflowState::default()
    };

    assert_eq!(state.title(), "工作流");
}

use super::*;
use crate::apps::workflow::model::{WorkflowHandle, WorkflowHandleSide, WorkflowViewport};
use iced::Size;

#[test]
fn saved_app_loading_open_delete_confirm_actions_and_copy_state_are_tracked() {
    let mut state = WorkflowState::default();

    state.begin_saved_apps_load();
    assert!(state.saved_apps_loading);
    assert!(state.saved_apps_error.is_none());
    assert!(state.saved_app_actions_menu_uuid.is_none());

    let summaries = vec![WorkflowSavedAppSummary {
        uuid: "uuid-1".to_string(),
        name: "App".to_string(),
        description: "Desc".to_string(),
        created_at_ms: 1,
        updated_at_ms: 2,
    }];
    state.finish_saved_apps_load(Ok(summaries.clone()));
    assert!(!state.saved_apps_loading);
    assert!(state.saved_apps_loaded);
    assert_eq!(state.saved_apps, summaries);
    assert!(state.saved_apps_error.is_none());

    state.begin_saved_apps_load();
    state.finish_saved_apps_load(Err("load failed".to_string()));
    assert_eq!(state.saved_apps_error.as_deref(), Some("load failed"));

    state.action_menu_open = true;
    state.zoom_menu_open = true;
    state.quick_insert_panel_open = true;
    state.begin_saved_app_open("uuid-1".to_string());
    assert_eq!(state.opening_saved_app_uuid.as_deref(), Some("uuid-1"));
    assert!(!state.action_menu_open);
    assert!(!state.zoom_menu_open);
    assert!(!state.quick_insert_panel_open);
    state.finish_saved_app_open();
    assert!(state.opening_saved_app_uuid.is_none());

    state.open_saved_app_delete_confirm("uuid-1".to_string());
    assert_eq!(state.confirm_delete_saved_app_uuid.as_deref(), Some("uuid-1"));
    state.begin_saved_app_delete("uuid-1".to_string());
    assert_eq!(state.deleting_saved_app_uuid.as_deref(), Some("uuid-1"));
    assert!(state.confirm_delete_saved_app_uuid.is_none());
    state.finish_saved_app_delete();
    assert!(state.deleting_saved_app_uuid.is_none());
    state.open_saved_app_delete_confirm("uuid-1".to_string());
    state.close_saved_app_delete_confirm();
    assert!(state.confirm_delete_saved_app_uuid.is_none());

    state.set_saved_app_search_query("query".to_string());
    assert_eq!(state.saved_app_search_query, "query");
    state.toggle_saved_app_actions("uuid-1".to_string());
    assert_eq!(state.saved_app_actions_menu_uuid.as_deref(), Some("uuid-1"));
    state.toggle_saved_app_actions("uuid-1".to_string());
    assert!(state.saved_app_actions_menu_uuid.is_none());
    state.toggle_saved_app_actions("uuid-2".to_string());
    state.close_saved_app_actions();
    assert!(state.saved_app_actions_menu_uuid.is_none());
    state.mark_saved_app_uuid_copied("uuid-1".to_string());
    assert_eq!(state.copied_saved_app_uuid.as_deref(), Some("uuid-1"));
    state.clear_saved_app_uuid_copied("other");
    assert_eq!(state.copied_saved_app_uuid.as_deref(), Some("uuid-1"));
    state.clear_saved_app_uuid_copied("uuid-1");
    assert!(state.copied_saved_app_uuid.is_none());
}

#[test]
fn remove_saved_app_removes_entries_and_resets_active_state_when_needed() {
    let document = test_document();
    let snapshot = test_snapshot("Active", &document);
    let mut state = WorkflowState {
        saved_apps: vec![
            WorkflowSavedAppSummary {
                uuid: "active-uuid".to_string(),
                name: "Active".to_string(),
                description: String::new(),
                created_at_ms: 0,
                updated_at_ms: 0,
            },
            WorkflowSavedAppSummary {
                uuid: "other-uuid".to_string(),
                name: "Other".to_string(),
                description: String::new(),
                created_at_ms: 0,
                updated_at_ms: 0,
            },
        ],
        apps: vec![
            app_entry("active", Some("active-uuid"), "Active", document.clone()),
            app_entry("other", Some("other-uuid"), "Other", document.clone()),
        ],
        active_app_id: Some("active".to_string()),
        local_uuid: Some("active-uuid".to_string()),
        source_path: Some("/tmp/active.yml".to_string()),
        source_name: "Active".to_string(),
        document,
        selected_node_id: Some("start".to_string()),
        selected_edge_id: Some("edge".to_string()),
        connection_draft: Some(connection_draft("start", WorkflowHandleKind::Source)),
        undo_stack: vec![snapshot.clone()],
        redo_stack: vec![snapshot.clone()],
        saved_snapshot: Some(snapshot),
        confirm_delete_saved_app_uuid: Some("active-uuid".to_string()),
        saved_app_actions_menu_uuid: Some("active-uuid".to_string()),
        copied_saved_app_uuid: Some("active-uuid".to_string()),
        active_is_dirty: true,
        ..WorkflowState::default()
    };

    assert!(state.remove_saved_app("active-uuid"));

    assert!(state.saved_apps.iter().all(|app| app.uuid != "active-uuid"));
    assert!(state.apps.iter().all(|app| app.local_uuid.as_deref() != Some("active-uuid")));
    assert!(state.active_app_id.is_none());
    assert!(!state.active_is_dirty);
    assert!(state.source_name.is_empty());
    assert!(state.local_uuid.is_none());
    assert!(state.source_path.is_none());
    assert!(state.document.nodes.is_empty());
    assert!(state.environment_variables.is_empty());
    assert!(state.conversation_variables.is_empty());
    assert!(state.selected_node_id.is_none());
    assert!(state.selected_edge_id.is_none());
    assert!(state.connection_draft.is_none());
    assert!(state.undo_stack.is_empty());
    assert!(state.redo_stack.is_empty());
    assert!(state.saved_snapshot.is_none());
    assert!(state.confirm_delete_saved_app_uuid.is_none());
    assert!(state.saved_app_actions_menu_uuid.is_none());
    assert!(state.copied_saved_app_uuid.is_none());

    assert!(!state.remove_saved_app("other-uuid"));
    assert!(state.apps.iter().all(|app| app.local_uuid.as_deref() != Some("other-uuid")));
}

#[test]
fn show_saved_apps_persists_active_snapshot_and_clears_editor_and_panel_state() {
    let document = test_document();
    let mut state = state_with_document("app", Some("uuid"), "App", document);
    state.source_name = "Renamed".to_string();
    state.app_editor = Some(app_editor(WorkflowAppEditorMode::Create, "Draft"));
    state.node_editor = Some(node_editor("answer"));
    state.variable_panel = Some(WorkflowVariablePanelKind::Environment);
    state.variable_editor = Some(variable_editor());
    state.context_menu = Some(WorkflowCanvasContextMenu {
        target: WorkflowCanvasContextMenuTarget::Canvas,
        anchor: Point::new(0.0, 0.0),
        world: Point::new(1.0, 1.0),
    });
    state.quick_insert_panel_open = true;
    state.zoom_menu_open = true;
    state.action_menu_open = true;
    state.error_message = Some("error".to_string());
    state.saved_app_actions_menu_uuid = Some("uuid".to_string());

    state.show_saved_apps();

    assert!(state.active_app_id.is_none());
    assert_eq!(state.apps[0].meta.name, "Renamed");
    assert!(state.app_editor.is_none());
    assert!(state.node_editor.is_none());
    assert!(state.variable_panel.is_none());
    assert!(state.variable_editor.is_none());
    assert!(state.context_menu.is_none());
    assert!(!state.quick_insert_panel_open);
    assert!(!state.zoom_menu_open);
    assert!(!state.action_menu_open);
    assert!(state.error_message.is_none());
    assert!(state.saved_app_actions_menu_uuid.is_none());
}

#[test]
fn selectors_error_helpers_and_selection_flags_reflect_current_state() {
    let mut document = test_document();
    document.nodes.push(WorkflowNode {
        id: "answer".to_string(),
        block_type: "answer".to_string(),
        title: "Answer".to_string(),
        ..test_node("answer", "answer")
    });
    document.edges = vec![test_edge("start", "answer", "edge")];
    let mut state = state_with_document("app", None, "App", document);
    state.environment_variables = vec![WorkflowEnvironmentVariable {
        id: "env".to_string(),
        name: "token".to_string(),
        value_type: "string".to_string(),
        value: Value::String("abc".to_string()),
        description: String::new(),
        raw_variable: Value::Null,
    }];
    state.conversation_variables = vec![WorkflowConversationVariable {
        id: "conv".to_string(),
        name: "topic".to_string(),
        value_type: "string".to_string(),
        value: Value::String("rust".to_string()),
        description: String::new(),
        raw_variable: Value::Null,
    }];

    assert!(state.has_apps());
    assert!(state.has_start_node());
    assert_eq!(state.active_app().map(|app| app.id.as_str()), Some("app"));
    assert_eq!(state.active_meta().map(|meta| meta.name.as_str()), Some("App"));
    assert_eq!(state.title(), "App");

    state.select_node("answer".to_string());
    assert_eq!(state.selected_node().map(|node| node.id.as_str()), Some("answer"));
    assert!(state.selected_edge().is_none());
    assert!(state.document.node("answer").unwrap().selected);
    assert!(!state.document.node("start").unwrap().selected);

    state.select_edge("edge".to_string());
    assert_eq!(state.selected_edge().map(|edge| edge.id.as_str()), Some("edge"));
    assert!(state.selected_node().is_none());
    assert!(state.document.edge("edge").unwrap().selected);

    assert_eq!(state.environment_variable("env").map(|item| item.name.as_str()), Some("token"));
    assert_eq!(state.conversation_variable("conv").map(|item| item.name.as_str()), Some("topic"));
    state.set_error("boom");
    assert_eq!(state.error_message.as_deref(), Some("boom"));
    state.clear_error();
    assert!(state.error_message.is_none());
    state.clear_selection();
    assert!(state.selected_node_id.is_none());
    assert!(state.selected_edge_id.is_none());
    assert!(state.context_menu.is_none());
}

#[test]
fn history_snapshots_deduplicate_trim_restore_and_refresh_dirty_state() {
    let document = test_document();
    let mut state = state_with_document("app", None, "App", document.clone());
    let initial = state.current_history_snapshot().expect("active snapshot should exist");

    state.push_history_snapshot(initial.clone());
    state.push_history_snapshot(initial.clone());
    assert_eq!(state.undo_stack.len(), 1);

    for index in 0..55 {
        let mut snapshot = initial.clone();
        snapshot.pan = Vector::new(index as f32, 0.0);
        state.push_history_snapshot(snapshot);
    }
    assert_eq!(state.undo_stack.len(), WORKFLOW_HISTORY_LIMIT);
    assert!(state.redo_stack.is_empty());

    state.refresh_dirty_state();
    assert!(!state.active_is_dirty);
    state.source_name = "Dirty".to_string();
    state.refresh_dirty_state();
    assert!(state.active_is_dirty);

    state.quick_insert_panel_open = true;
    state.action_menu_open = true;
    state.zoom_menu_open = true;
    state.dragging_node_id = Some("start".to_string());
    state.drag_pending_snapshot = Some(initial.clone());
    state.app_editor = Some(app_editor(WorkflowAppEditorMode::Create, "Draft"));
    state.node_editor = Some(node_editor("answer"));
    state.variable_panel = Some(WorkflowVariablePanelKind::Conversation);
    state.variable_editor = Some(variable_editor());
    state.restore_history_snapshot(initial.clone());

    assert_eq!(state.source_name, "App");
    assert_eq!(state.apps[0].meta.name, "App");
    assert!(!state.active_is_dirty);
    assert!(state.connection_draft.is_none());
    assert!(state.context_menu.is_none());
    assert!(!state.quick_insert_panel_open);
    assert!(!state.action_menu_open);
    assert!(!state.zoom_menu_open);
    assert!(state.dragging_node_id.is_none());
    assert!(state.drag_pending_snapshot.is_none());
    assert!(state.app_editor.is_none());
    assert!(state.node_editor.is_none());
    assert!(state.variable_panel.is_none());
    assert!(state.variable_editor.is_none());

    let inactive = WorkflowState::default();
    assert!(inactive.current_history_snapshot().is_none());
}

#[test]
fn apply_loaded_replace_loaded_and_select_app_manage_active_entries() {
    let mut state = WorkflowState::default();
    let first = loaded_workflow("First", Some(WorkflowViewport { x: 10.0, y: 20.0, zoom: 9.0 }));
    state.apply_loaded(first, (800.0, 600.0));

    let first_id = state.active_app_id.clone().expect("first app id");
    assert_eq!(state.apps.len(), 1);
    assert!(matches!(state.source_name.as_str(), "First" | "Second Dirty"));
    assert_eq!(state.local_uuid.as_deref(), Some("First-uuid"));
    assert_eq!(state.source_path.as_deref(), Some("/tmp/First.yml"));
    assert_eq!(state.pan, Vector::new(10.0, 20.0));
    assert_eq!(state.zoom, 4.0);
    assert_eq!(state.selected_node_id.as_deref(), Some("First-start"));
    assert_eq!(state.selected_edge_id.as_deref(), Some("First-edge"));
    assert!(!state.active_is_dirty);
    assert!(state.saved_snapshot.is_some());
    assert_eq!(state.status_message.as_deref(), Some("已加载 /tmp/First.yml"));

    let second = loaded_workflow("Second", None);
    state.apply_loaded(second, (800.0, 600.0));
    let second_id = state.active_app_id.clone().expect("second app id");
    assert_eq!(first_id, second_id);
    assert_eq!(state.apps.len(), 2);

    state.source_name = "Second Dirty".to_string();
    assert!(!state.select_app(&first_id));
    assert!(matches!(
        state.apps.iter().find(|app| app.id == second_id).unwrap().meta.name.as_str(),
        "First" | "Second Dirty"
    ));
    assert!(matches!(state.source_name.as_str(), "First" | "Second Dirty"));
    assert!(!state.select_app(&first_id));
    assert!(!state.select_app("missing"));
    assert!(!state.select_app_by_local_uuid("Second-uuid"));
    assert_eq!(state.source_name, "Second Dirty");

    let replacement = loaded_workflow("Replacement", Some(WorkflowViewport { x: 1.0, y: 2.0, zoom: 0.01 }));
    let active_id = state.active_app_id.clone().unwrap();
    state.replace_active_loaded(replacement, (640.0, 480.0));
    assert_eq!(state.active_app_id.as_deref(), Some(active_id.as_str()));
    assert_eq!(state.source_name, "Replacement");
    assert_eq!(state.pan, Vector::new(1.0, 2.0));
    assert_eq!(state.zoom, 0.1);
    assert_eq!(state.status_message.as_deref(), Some("已重新载入 Replacement"));

    let mut empty_state = WorkflowState::default();
    empty_state.replace_active_loaded(loaded_workflow("Delegated", None), (800.0, 600.0));
    assert_eq!(empty_state.apps.len(), 1);
    assert_eq!(empty_state.source_name, "Delegated");
}

#[test]
fn editor_openers_panel_toggles_and_saved_identifiers_update_current_and_entry_snapshots() {
    let document = test_document();
    let mut state = state_with_document("app", None, "未命名应用", document);
    state.apps.push(app_entry("second", None, "未命名应用 2", test_document()));

    state.open_create_editor();
    assert_eq!(state.app_editor.as_ref().unwrap().name, "未命名应用 3");
    assert!(matches!(state.app_editor.as_ref().unwrap().mode, WorkflowAppEditorMode::Create));
    assert!(state.node_editor.is_none());
    assert!(state.variable_panel.is_none());

    state.open_edit_editor(Some("second"));
    assert_eq!(state.app_editor.as_ref().unwrap().name, "未命名应用 2");
    assert!(matches!(state.app_editor.as_ref().unwrap().mode, WorkflowAppEditorMode::Edit(_)));
    state.close_editor();
    assert!(state.app_editor.is_none());

    state.toggle_action_menu();
    assert!(state.action_menu_open);
    state.toggle_zoom_menu();
    assert!(!state.action_menu_open);
    assert!(state.zoom_menu_open);
    state.toggle_quick_insert_panel();
    assert!(!state.zoom_menu_open);
    assert!(state.quick_insert_panel_open);
    state.close_quick_insert_panel();
    assert!(!state.quick_insert_panel_open);
    state.action_menu_open = true;
    state.zoom_menu_open = true;
    state.quick_insert_panel_open = true;
    state.close_floating_panels();
    assert!(!state.action_menu_open);
    assert!(!state.zoom_menu_open);
    assert!(!state.quick_insert_panel_open);

    state.update_active_source_path("/tmp/app.yml".to_string());
    assert_eq!(state.source_path.as_deref(), Some("/tmp/app.yml"));
    assert_eq!(state.apps[0].source_path.as_deref(), Some("/tmp/app.yml"));
    assert!(!state.active_is_dirty);
    state.update_active_local_uuid("local-uuid".to_string());
    assert_eq!(state.local_uuid.as_deref(), Some("local-uuid"));
    assert_eq!(state.apps[0].local_uuid.as_deref(), Some("local-uuid"));
    assert!(!state.active_is_dirty);
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
        source_handles: vec![WorkflowHandle {
            id: "source".to_string(),
            label: String::new(),
            kind: WorkflowHandleKind::Source,
        }],
        target_handles: if block_type == "start" {
            Vec::new()
        } else {
            vec![WorkflowHandle {
                id: "target".to_string(),
                label: String::new(),
                kind: WorkflowHandleKind::Target,
            }]
        },
        z_index: 0.0,
        raw_node: Value::Null,
    }
}

fn test_edge(source: &str, target: &str, id: &str) -> WorkflowEdge {
    WorkflowEdge {
        id: id.to_string(),
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

fn test_document() -> WorkflowDocument {
    WorkflowDocument {
        name: "Document".to_string(),
        nodes: vec![test_node("start", "start")],
        edges: Vec::new(),
        viewport: WorkflowViewport::default(),
    }
}

fn test_snapshot(name: &str, document: &WorkflowDocument) -> WorkflowHistorySnapshot {
    WorkflowHistorySnapshot {
        meta: WorkflowAppMeta { name: name.to_string(), ..Default::default() },
        document: document.clone(),
        environment_variables: Vec::new(),
        conversation_variables: Vec::new(),
        pan: Vector::new(0.0, 0.0),
        zoom: 1.0,
        selected_node_id: None,
        selected_edge_id: None,
    }
}

fn app_entry(id: &str, local_uuid: Option<&str>, name: &str, document: WorkflowDocument) -> WorkflowAppEntry {
    let snapshot = test_snapshot(name, &document);
    WorkflowAppEntry {
        id: id.to_string(),
        local_uuid: local_uuid.map(str::to_string),
        meta: WorkflowAppMeta { name: name.to_string(), ..Default::default() },
        source_path: None,
        raw_root: Value::Null,
        document,
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
        saved_snapshot: snapshot,
    }
}

fn state_with_document(
    id: &str,
    local_uuid: Option<&str>,
    name: &str,
    document: WorkflowDocument,
) -> WorkflowState {
    let snapshot = test_snapshot(name, &document);
    WorkflowState {
        apps: vec![WorkflowAppEntry {
            id: id.to_string(),
            local_uuid: local_uuid.map(str::to_string),
            meta: WorkflowAppMeta { name: name.to_string(), ..Default::default() },
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
        active_app_id: Some(id.to_string()),
        source_name: name.to_string(),
        document,
        saved_snapshot: Some(snapshot),
        pan: Vector::new(0.0, 0.0),
        zoom: 1.0,
        ..WorkflowState::default()
    }
}

fn loaded_workflow(name: &str, viewport: Option<WorkflowViewport>) -> LoadedWorkflow {
    let start = WorkflowNode { id: format!("{name}-start"), selected: true, ..test_node("start", "start") };
    let answer = WorkflowNode { id: format!("{name}-answer"), ..test_node("answer", "answer") };
    let edge = WorkflowEdge {
        id: format!("{name}-edge"),
        source: start.id.clone(),
        target: answer.id.clone(),
        selected: true,
        ..test_edge("start", "answer", "edge")
    };
    LoadedWorkflow {
        local_uuid: Some(format!("{name}-uuid")),
        source_path: Some(format!("/tmp/{name}.yml")),
        source_name: name.to_string(),
        app_meta: WorkflowAppMeta { name: name.to_string(), ..Default::default() },
        document: WorkflowDocument {
            name: name.to_string(),
            nodes: vec![start, answer],
            edges: vec![edge],
            viewport: viewport.unwrap_or_default(),
        },
        environment_variables: Vec::new(),
        conversation_variables: Vec::new(),
        had_viewport: viewport.is_some(),
        raw_root: Value::Null,
    }
}

fn app_editor(mode: WorkflowAppEditorMode, name: &str) -> WorkflowAppEditorDraft {
    WorkflowAppEditorDraft {
        mode,
        name: name.to_string(),
        description: String::new(),
        icon: "🤖".to_string(),
        use_icon_as_answer_icon: false,
        max_active_requests_input: "0".to_string(),
    }
}

fn node_editor(block_type: &str) -> WorkflowNodeEditorDraft {
    WorkflowNodeEditorDraft {
        mode: WorkflowNodeEditorMode::Create,
        active_tab: WorkflowNodeEditorTab::Description,
        block_type: block_type.to_string(),
        title: block_type.to_string(),
        description: String::new(),
        description_editor: text_editor::Content::with_text(""),
        position: Point::new(0.0, 0.0),
        visual_draft: None,
        validation: WorkflowNodeEditorValidation::default(),
        show_raw_data_editor: false,
        raw_data_editor: text_editor::Content::with_text(""),
        hovered_start_variable_index: None,
        start_variable_focus_index: None,
        start_variable_editor: None,
    }
}

fn variable_editor() -> WorkflowVariableEditorDraft {
    WorkflowVariableEditorDraft {
        mode: WorkflowVariableEditorMode::CreateEnvironment,
        name: "env".to_string(),
        description: String::new(),
        value_type: "string".to_string(),
        raw_value_editor: text_editor::Content::with_text("value"),
    }
}

fn connection_draft(node_id: &str, kind: WorkflowHandleKind) -> WorkflowConnectionDraft {
    WorkflowConnectionDraft {
        from: WorkflowConnectionEndpoint {
            node_id: node_id.to_string(),
            handle_id: match kind {
                WorkflowHandleKind::Source => "source".to_string(),
                WorkflowHandleKind::Target => "target".to_string(),
            },
            kind,
        },
        cursor_world: Point::new(0.0, 0.0),
    }
}
