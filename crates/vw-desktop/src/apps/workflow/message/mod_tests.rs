#![allow(unused_must_use)]
use super::*;
use crate::apps::workflow::model::create_node_from_type;
use crate::apps::workflow::state::WorkflowVariablePanelKind;
use iced::{Point, Vector};

fn app_with_workflow() -> App {
    let mut app = App::new().0;
    app.window_size = (1000.0, 800.0);
    let loaded = load_document_from_text(
        None,
        r#"
app:
  name: 消息入口测试
workflow:
  graph:
    nodes:
      - id: start
        position:
          x: 10
          y: 20
        data:
          title: Start
          type: start
      - id: answer
        position:
          x: 260
          y: 20
        data:
          title: Answer
          type: answer
    edges: []
"#
        .to_string(),
    )
    .expect("workflow should load");
    apply_loaded(&mut app, loaded);
    app
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("mod_tests"));
}

#[test]
fn update_dispatches_to_app_node_document_and_canvas_handlers() {
    let mut app = app_with_workflow();

    update(&mut app, WorkflowMessage::SavedAppSearchChanged("abc".to_string()));
    assert_eq!(app.workflow_state.saved_app_search_query, "abc");

    update(&mut app, WorkflowMessage::ToggleQuickInsertPanel);
    assert!(app.workflow_state.quick_insert_panel_open);

    update(&mut app, WorkflowMessage::OpenVariablePanel(WorkflowVariablePanelKind::System));
    assert_eq!(app.workflow_state.variable_panel, Some(WorkflowVariablePanelKind::System));

    update(&mut app, WorkflowMessage::SelectNode("start".to_string()));
    assert_eq!(app.workflow_state.selected_node_id.as_deref(), Some("start"));
}

#[test]
fn apply_loaded_syncs_top_tab_and_suggested_position_accounts_for_viewport() {
    let mut app = App::new().0;
    app.window_size = (1200.0, 900.0);
    let loaded = load_document_from_text(
        None,
        "app:\n  name: Loaded\nworkflow:\n  graph:\n    nodes: []\n    edges: []\n".to_string(),
    )
    .expect("workflow should load");

    apply_loaded(&mut app, loaded);
    assert_eq!(app.screen, crate::app::Screen::WorkflowTool);
    assert_eq!(app.active_tab_id.as_deref(), Some(crate::apps::workflow::WORKFLOW_TOOL_TAB_ID));
    assert_eq!(app.workflow_state.title(), "Loaded");

    app.workflow_state.pan = Vector::new(20.0, 40.0);
    app.workflow_state.zoom = 2.0;
    app.workflow_state.document.nodes =
        vec![create_node_from_type("answer", "a".to_string(), Point::ORIGIN, 1.0).unwrap()];
    let position = suggested_new_node_position(&app);
    assert_eq!(position, Point::new(330.0, 233.0));
}

#[test]
fn helper_functions_cover_save_export_and_record_mapping() {
    let mut empty_app = App::new().0;
    let _ = load_saved_apps_task();
    let _ = open_saved_app_task("uuid".to_string());
    let _ = delete_saved_app_task("uuid".to_string());
    let _ = open_file();
    let _ = save_active_app(&mut empty_app, false);

    let mut app = app_with_workflow();
    assert_eq!(suggested_export_file_name(" Sales/Bot ", "svg"), "Sales_Bot.svg");
    assert_eq!(suggested_export_file_name("!!!", "png"), "workflow_app.png");

    let entry = app.workflow_state.active_entry_snapshot().expect("active entry");
    let _ = save_entry(entry.clone(), false);
    let _ = save_entry(entry, true);

    let _ = export_svg(&mut empty_app);
    let _ = export_svg(&mut app);
    let _ = export_png(&mut app);
    let _ = export_jpeg(&mut app);
    export_finished(&mut app, Ok(()));
    assert_eq!(app.workflow_state.status_message.as_deref(), Some("已导出工作流图片"));
    export_finished(&mut app, Err("bad export".to_string()));
    assert_eq!(app.workflow_state.error_message.as_deref(), Some("bad export"));

    #[cfg(not(target_arch = "wasm32"))]
    {
        let summary = saved_app_summary_from_record(vw_gateway_client::WorkflowRecordSummary {
            uuid: "u".to_string(),
            name: "n".to_string(),
            description: "d".to_string(),
            created_at_ms: 1,
            updated_at_ms: 2,
        });
        assert_eq!(summary.uuid, "u");
        assert_eq!(summary.name, "n");
        assert_eq!(summary.description, "d");
        assert_eq!(summary.created_at_ms, 1);
        assert_eq!(summary.updated_at_ms, 2);
    }
}
