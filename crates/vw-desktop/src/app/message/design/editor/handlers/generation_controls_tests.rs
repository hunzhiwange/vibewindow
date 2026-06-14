#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("generation_controls_tests"));
}

use super::generation_controls::{
    close_design_generation_device_popover, close_design_generation_executor_popover,
    close_design_generation_model_popover, close_design_generation_style_popover,
    close_design_generation_theme_popover, close_design_planner_quick_menu,
    design_generation_acp_agent_selected, design_generation_apply_partial_regenerate,
    design_generation_cancel, design_generation_device_selected, design_generation_model_changed,
    design_generation_model_selected, design_generation_parallel_pages_changed,
    design_generation_style_selected, design_generation_theme_selected,
    design_planner_new_chat_session, design_planner_select_chat_session, design_planner_select_tab,
    design_planner_set_corner, open_design_planner_quick_menu, set_design_page_target_frame,
    toggle_design_generation_device_popover, toggle_design_generation_executor_popover,
    toggle_design_generation_model_popover, toggle_design_generation_style_popover,
    toggle_design_generation_theme_popover, toggle_design_planner_panel_collapsed,
};
use crate::app::App;
use crate::app::task::{TASK_MODEL_AUTO, TaskExecutorBackend};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus,
    DesignGenerationTheme, DesignPlannerCorner, DesignPlannerTab, DesignState, DesignStyle,
};

fn element(id: &str, children: Vec<DesignElement>) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "type": "frame",
        "children": children
    }))
    .unwrap()
}

fn text_element(id: &str) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "type": "text",
        "content": "",
        "visible": false
    }))
    .unwrap()
}

fn placeholder(id: &str) -> DesignElement {
    element(
        id,
        vec![
            text_element(&format!("{id}-status")),
            element(&format!("{id}-badge"), Vec::new()),
            text_element(&format!("{id}-badge-text")),
            text_element(&format!("{id}-slot-hint")),
            text_element(&format!("{id}-status-id")),
        ],
    )
}

fn module(
    module_id: &str,
    target_frame_id: &str,
    status: DesignGenerationStatus,
) -> DesignGenerationModule {
    DesignGenerationModule {
        module_id: module_id.to_string(),
        title: module_id.to_string(),
        description: String::new(),
        status,
        target_frame_id: target_frame_id.to_string(),
        target_frame_options: vec![target_frame_id.to_string()],
        generated_doc: None,
        is_generating: false,
        logs: Vec::new(),
    }
}

fn app_with_state(mut state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    state.design_generation_model = String::new();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

fn app_with_design() -> App {
    app_with_state(DesignState::new(DesignDoc::default()))
}

#[test]
fn popover_toggles_open_one_menu_at_a_time_and_close_handlers_reset_flags() {
    let mut app = app_with_design();
    {
        let state = app.active_design_state_mut().unwrap();
        state.design_generation_model_popover = true;
        state.design_generation_theme_popover = true;
        state.design_generation_device_popover = true;
        state.design_generation_style_popover = true;
    }

    let _ = toggle_design_generation_executor_popover(&mut app);
    let state = app.active_design_state().unwrap();
    assert!(state.design_generation_executor_popover);
    assert!(!state.design_generation_model_popover);
    assert!(!state.design_generation_theme_popover);
    assert!(!state.design_generation_device_popover);
    assert!(!state.design_generation_style_popover);

    let _ = close_design_generation_executor_popover(&mut app);
    let _ = toggle_design_generation_model_popover(&mut app);
    let _ = close_design_generation_model_popover(&mut app);
    let _ = toggle_design_generation_theme_popover(&mut app);
    let _ = close_design_generation_theme_popover(&mut app);
    let _ = toggle_design_generation_device_popover(&mut app);
    let _ = close_design_generation_device_popover(&mut app);
    let _ = toggle_design_generation_style_popover(&mut app);
    let _ = close_design_generation_style_popover(&mut app);

    let state = app.active_design_state().unwrap();
    assert!(!state.design_generation_executor_popover);
    assert!(!state.design_generation_model_popover);
    assert!(!state.design_generation_theme_popover);
    assert!(!state.design_generation_device_popover);
    assert!(!state.design_generation_style_popover);
}

#[test]
fn selection_handlers_update_generation_controls_and_summaries() {
    let mut app = app_with_design();

    let _ = design_generation_acp_agent_selected(&mut app, Some("agent-a".to_string()));
    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_executor, TaskExecutorBackend::Internal);
    assert_eq!(state.design_generation_model, TASK_MODEL_AUTO);
    assert_eq!(app.acp_agent.as_deref(), Some("agent-a"));

    let _ = design_generation_model_selected(&mut app, "  gpt-test  ".to_string());
    assert_eq!(app.active_design_state().unwrap().design_generation_model, "gpt-test");

    let _ = design_generation_model_changed(&mut app, "   ".to_string());
    assert_eq!(app.active_design_state().unwrap().design_generation_model, TASK_MODEL_AUTO);

    let _ = design_generation_style_selected(&mut app, DesignStyle::Dark);
    let _ = design_generation_device_selected(&mut app, DesignGenerationDevice::MobileApp);
    let _ = design_generation_theme_selected(&mut app, DesignGenerationTheme::Halo);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_style, DesignStyle::Dark);
    assert_eq!(state.design_generation_device, DesignGenerationDevice::MobileApp);
    assert_eq!(state.design_generation_theme, DesignGenerationTheme::Halo);
    assert!(state.design_generation_summary.as_deref().unwrap().contains("Halo"));
}

#[test]
fn parallel_pages_changed_filters_clamps_and_reports_empty_input() {
    let mut app = app_with_design();

    let _ = design_generation_parallel_pages_changed(&mut app, "abc99".to_string());
    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_parallel_pages, 16);
    assert_eq!(state.design_generation_parallel_pages_input, "16");

    let _ = design_generation_parallel_pages_changed(&mut app, "abc".to_string());
    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_parallel_pages_input, "");
    assert_eq!(state.design_generation_summary.as_deref(), Some("请输入页面并行数，最小为 1。"));
}

#[test]
fn planner_controls_toggle_panel_menu_corner_tab_and_sessions() {
    let mut app = app_with_design();
    let initial_panel = app.show_design_planner_panel;

    let _ = toggle_design_planner_panel_collapsed(&mut app);
    assert_eq!(app.show_design_planner_panel, !initial_panel);

    let _ = design_planner_select_tab(&mut app, DesignPlannerTab::Tools);
    let _ = open_design_planner_quick_menu(&mut app);
    assert_eq!(
        app.active_design_state().unwrap().design_planner_active_tab,
        DesignPlannerTab::Tools
    );
    assert!(app.active_design_state().unwrap().design_planner_quick_menu_open);

    let _ = design_planner_set_corner(&mut app, DesignPlannerCorner::TopLeft);
    assert_eq!(app.design_planner_corner, DesignPlannerCorner::TopLeft);
    assert!(!app.active_design_state().unwrap().design_planner_quick_menu_open);

    let old_session_count = app.active_design_state().unwrap().design_chat_sessions.len();
    let _ = design_planner_new_chat_session(&mut app);
    assert_eq!(
        app.active_design_state().unwrap().design_chat_sessions.len(),
        old_session_count + 1
    );
    assert_eq!(
        app.active_design_state().unwrap().design_planner_active_tab,
        DesignPlannerTab::Chat
    );

    let _ = design_planner_select_chat_session(&mut app, 0);
    assert_eq!(app.active_design_state().unwrap().design_chat_active_session, 0);

    let _ = close_design_planner_quick_menu(&mut app);
    assert!(!app.active_design_state().unwrap().design_planner_quick_menu_open);
}

#[test]
fn cancel_without_loading_only_requests_snapshot_and_preserves_state() {
    let mut app = app_with_design();

    let _ = design_generation_cancel(&mut app);

    let state = app.active_design_state().unwrap();
    assert!(!state.design_generation_loading);
    assert!(state.design_generation_summary.is_none());
}

#[test]
fn cancel_loading_requeues_running_modules_and_updates_chat() {
    let mut state = DesignState::new(DesignDoc {
        children: vec![placeholder("target")],
        ..DesignDoc::default()
    });
    state.design_generation_loading = true;
    state.design_generation_anim_frame = 2;
    let mut running = module("hero", "target", DesignGenerationStatus::Running);
    running.is_generating = true;
    state.design_generation_pages = vec![DesignGenerationPage {
        frame_id: "page".to_string(),
        title: "Page".to_string(),
        objective: String::new(),
        status: DesignGenerationStatus::Running,
        modules: vec![running],
    }];
    let mut app = app_with_state(state);

    let _ = design_generation_cancel(&mut app);

    let state = app.active_design_state().unwrap();
    let module = &state.design_generation_pages[0].modules[0];
    assert!(!state.design_generation_loading);
    assert_eq!(state.design_generation_anim_frame, 0);
    assert_eq!(module.status, DesignGenerationStatus::Queued);
    assert!(!module.is_generating);
    assert_eq!(
        state.doc.find_element("target-badge-text").unwrap().content.as_deref(),
        Some("queued")
    );
    assert!(state.design_chat_messages.iter().any(|message| message.content.contains("已停止")));
}

#[test]
fn partial_regenerate_reports_busy_empty_and_sets_targets_loading() {
    let mut busy_app = app_with_design();
    busy_app.active_design_state_mut().unwrap().design_generation_loading = true;
    let _ = design_generation_apply_partial_regenerate(&mut busy_app);
    assert_eq!(
        busy_app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("当前正在生成中，请稍后重试。")
    );

    let mut empty_app = app_with_design();
    let _ = design_generation_apply_partial_regenerate(&mut empty_app);
    assert_eq!(
        empty_app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("暂无可重新生成的模块。")
    );

    let mut state = DesignState::new(DesignDoc::default());
    state.design_generation_pages = vec![DesignGenerationPage {
        frame_id: "page".to_string(),
        title: "Page".to_string(),
        objective: String::new(),
        status: DesignGenerationStatus::Queued,
        modules: vec![module("hero", "target", DesignGenerationStatus::Failed)],
    }];
    let mut app = app_with_state(state);
    let _ = design_generation_apply_partial_regenerate(&mut app);
    let state = app.active_design_state().unwrap();
    assert!(state.design_generation_loading);
    assert_eq!(state.design_generation_summary.as_deref(), Some("已触发重新生成：1 个页面任务。"));
}

#[test]
fn set_design_page_target_frame_normalizes_and_adds_option() {
    let mut state = DesignState::new(DesignDoc::default());
    state.design_generation_pages = vec![DesignGenerationPage {
        frame_id: "page".to_string(),
        title: "Page".to_string(),
        objective: String::new(),
        status: DesignGenerationStatus::Queued,
        modules: vec![module("hero", "old-target", DesignGenerationStatus::Queued)],
    }];
    let mut app = app_with_state(state);

    let _ = set_design_page_target_frame(
        &mut app,
        "page".to_string(),
        "hero".to_string(),
        r#"{ "id": "new-target" }"#.to_string(),
    );

    let module = &app.active_design_state().unwrap().design_generation_pages[0].modules[0];
    assert_eq!(module.target_frame_id, "new-target");
    assert!(module.target_frame_options.contains(&"new-target".to_string()));
}
