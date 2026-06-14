use super::*;

fn new_app() -> App {
    App::new().0
}

#[test]
fn saved_app_list_messages_update_loading_search_menu_and_delete_state() {
    let mut app = new_app();

    assert!(app::handle(&mut app, WorkflowMessage::LoadSavedApps).is_some());
    assert!(app.workflow_state.saved_apps_loading);

    app::handle(
        &mut app,
        WorkflowMessage::LoadSavedAppsFinished(Ok(vec![WorkflowSavedAppSummary {
            uuid: "uuid-1".to_string(),
            name: "应用".to_string(),
            description: "描述".to_string(),
            created_at_ms: 1,
            updated_at_ms: 2,
        }])),
    );
    assert!(!app.workflow_state.saved_apps_loading);
    assert!(app.workflow_state.saved_apps_loaded);
    assert_eq!(app.workflow_state.saved_apps.len(), 1);

    app::handle(&mut app, WorkflowMessage::SavedAppSearchChanged("query".to_string()));
    assert_eq!(app.workflow_state.saved_app_search_query, "query");

    app::handle(&mut app, WorkflowMessage::ToggleSavedAppActions("uuid-1".to_string()));
    assert_eq!(app.workflow_state.saved_app_actions_menu_uuid.as_deref(), Some("uuid-1"));
    app::handle(&mut app, WorkflowMessage::CloseSavedAppActions);
    assert!(app.workflow_state.saved_app_actions_menu_uuid.is_none());

    app::handle(&mut app, WorkflowMessage::RequestDeleteSavedApp("uuid-1".to_string()));
    assert_eq!(app.workflow_state.confirm_delete_saved_app_uuid.as_deref(), Some("uuid-1"));
    app::handle(&mut app, WorkflowMessage::CancelDeleteSavedApp);
    assert!(app.workflow_state.confirm_delete_saved_app_uuid.is_none());

    app::handle(&mut app, WorkflowMessage::DeleteSavedApp("uuid-1".to_string()));
    assert_eq!(app.workflow_state.deleting_saved_app_uuid.as_deref(), Some("uuid-1"));
    app::handle(&mut app, WorkflowMessage::DeleteSavedAppFinished(Ok("uuid-1".to_string())));
    assert!(app.workflow_state.deleting_saved_app_uuid.is_none());
    assert!(app.workflow_state.saved_apps.is_empty());
}

#[test]
fn saved_app_errors_and_copy_state_are_recorded() {
    let mut app = new_app();

    app::handle(&mut app, WorkflowMessage::LoadSavedAppsFinished(Err("load failed".to_string())));
    assert_eq!(app.workflow_state.saved_apps_error.as_deref(), Some("load failed"));

    app::handle(&mut app, WorkflowMessage::OpenSavedAppFinished(Err("open failed".to_string())));
    assert_eq!(app.workflow_state.error_message.as_deref(), Some("open failed"));

    app::handle(
        &mut app,
        WorkflowMessage::DeleteSavedAppFinished(Err("delete failed".to_string())),
    );
    assert_eq!(app.workflow_state.error_message.as_deref(), Some("delete failed"));

    app::handle(&mut app, WorkflowMessage::CopySavedAppUuid("uuid-2".to_string()));
    assert_eq!(app.workflow_state.copied_saved_app_uuid.as_deref(), Some("uuid-2"));
    app::handle(&mut app, WorkflowMessage::ClearCopiedSavedAppUuid("uuid-2".to_string()));
    assert!(app.workflow_state.copied_saved_app_uuid.is_none());
}

#[test]
fn show_saved_apps_loads_once_and_closes_active_panels() {
    let mut app = new_app();
    app.workflow_state.saved_apps_loaded = false;
    app.workflow_state.saved_apps_loading = false;
    app.workflow_state.action_menu_open = true;

    assert!(app::handle(&mut app, WorkflowMessage::ShowSavedApps).is_some());
    assert!(app.workflow_state.saved_apps_loading);
    assert!(!app.workflow_state.action_menu_open);

    assert!(app::handle(&mut app, WorkflowMessage::ShowSavedApps).is_some());
}

#[test]
fn inline_yaml_opens_valid_preview_and_reports_invalid_yaml() {
    let mut app = new_app();
    let yaml = r#"
app:
  name: Inline
workflow:
  graph:
    nodes: []
    edges: []
"#;

    app::handle(
        &mut app,
        WorkflowMessage::OpenInlineYaml {
            workflow_yaml: yaml.to_string(),
            focus_node_id: Some("".to_string()),
        },
    );
    assert_eq!(app.workflow_state.status_message.as_deref(), Some("已打开临时工作流预览"));

    app::handle(
        &mut app,
        WorkflowMessage::OpenInlineYaml {
            workflow_yaml: "not: [valid".to_string(),
            focus_node_id: None,
        },
    );
    assert!(app.workflow_state.error_message.is_some());
}

#[test]
fn app_editor_messages_edit_and_submit_create_flow() {
    let mut app = new_app();

    app::handle(&mut app, WorkflowMessage::OpenCreateAppEditor);
    assert!(app.workflow_state.app_editor.is_some());
    app::handle(&mut app, WorkflowMessage::AppEditorNameChanged("Demo".to_string()));
    app::handle(&mut app, WorkflowMessage::AppEditorDescriptionChanged("Desc".to_string()));
    app::handle(&mut app, WorkflowMessage::AppEditorIconChanged("D".to_string()));
    app::handle(&mut app, WorkflowMessage::AppEditorUseIconAsAnswerIconChanged(true));
    app::handle(&mut app, WorkflowMessage::AppEditorMaxActiveRequestsChanged("3".to_string()));

    let editor = app.workflow_state.app_editor.as_ref().unwrap();
    assert_eq!(editor.name, "Demo");
    assert_eq!(editor.description, "Desc");
    assert_eq!(editor.icon, "D");
    assert!(editor.use_icon_as_answer_icon);
    assert_eq!(editor.max_active_requests_input, "3");

    app::handle(&mut app, WorkflowMessage::SubmitAppEditor);
    assert!(app.workflow_state.app_editor.is_none());
    assert_eq!(app.workflow_state.source_name, "Demo");

    app::handle(&mut app, WorkflowMessage::OpenEditAppEditor(None));
    assert!(app.workflow_state.app_editor.is_some());
    app::handle(&mut app, WorkflowMessage::CloseAppEditor);
    assert!(app.workflow_state.app_editor.is_none());
}

#[test]
fn app_handle_returns_none_for_non_app_messages() {
    let mut app = new_app();

    assert!(app::handle(&mut app, WorkflowMessage::ToggleZoomMenu).is_none());
}
