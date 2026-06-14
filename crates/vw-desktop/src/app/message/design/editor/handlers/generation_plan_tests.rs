#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("generation_plan_tests"));
}

use super::generation_plan::{design_generation_completed, design_generation_submit};
use crate::app::App;
use crate::app::message::design::editor::DesignPlanExecutionResult;
use crate::app::task::TaskLogStream;
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::{
    DesignGenerationModule, DesignGenerationPage, DesignGenerationPlan, DesignGenerationStatus,
    DesignState,
};

fn module(module_id: &str, target_frame_id: &str) -> DesignGenerationModule {
    DesignGenerationModule {
        module_id: module_id.to_string(),
        title: module_id.to_string(),
        description: "module description".to_string(),
        status: DesignGenerationStatus::Planned,
        target_frame_id: target_frame_id.to_string(),
        target_frame_options: Vec::new(),
        generated_doc: None,
        is_generating: true,
        logs: vec!["stale".to_string()],
    }
}

fn plan() -> DesignGenerationPlan {
    DesignGenerationPlan {
        summary: Some("summary".to_string()),
        pages: vec![DesignGenerationPage {
            frame_id: "page-1".to_string(),
            title: "Home".to_string(),
            objective: "Sell".to_string(),
            status: DesignGenerationStatus::Planned,
            modules: vec![module("hero", "page-1-module-1")],
        }],
    }
}

fn app_with_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

fn app_with_design() -> App {
    app_with_state(DesignState::new(DesignDoc::default()))
}

#[test]
fn submit_without_active_design_state_is_noop() {
    let mut app = App::new().0;

    let _ = design_generation_submit(&mut app);

    assert!(app.active_design_state().is_none());
}

#[test]
fn submit_empty_prompt_reports_validation_message() {
    let mut app = app_with_design();
    app.active_design_state_mut().unwrap().design_chat_input =
        iced::widget::text_editor::Content::with_text("   ");

    let _ = design_generation_submit(&mut app);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_summary.as_deref(), Some("请先输入页面与模块需求。"));
    assert!(!state.design_generation_loading);
}

#[test]
fn submit_valid_prompt_prepares_loading_state_chat_and_stream_receiver() {
    let mut app = app_with_design();
    app.project_path = Some(std::env::temp_dir().display().to_string());
    app.active_design_state_mut().unwrap().design_chat_input =
        iced::widget::text_editor::Content::with_text("Build a product page");

    let _ = design_generation_submit(&mut app);

    let state = app.active_design_state().unwrap();
    assert!(state.design_generation_loading);
    assert_eq!(state.design_generation_brief, "Build a product page");
    assert!(state.design_generation_stream_rx.is_some());
    assert!(state.design_generation_logs.iter().any(|line| line.contains("[PLAN] start")));
    assert!(
        state.design_chat_messages.iter().any(|message| message.content == "Build a product page")
    );
    assert!(state.design_chat_input.text().is_empty());
}

#[test]
fn completed_while_not_loading_only_requests_snapshot() {
    let mut app = app_with_design();

    let _ = design_generation_completed(
        &mut app,
        Ok(DesignPlanExecutionResult { plan: plan(), logs: Vec::new() }),
    );

    assert!(app.active_design_state().unwrap().design_generation_pages.is_empty());
}

#[test]
fn completed_error_clears_loading_and_records_failure() {
    let mut app = app_with_design();
    {
        let state = app.active_design_state_mut().unwrap();
        state.design_generation_loading = true;
        let (_tx, rx) = std::sync::mpsc::channel::<TaskLogStream>();
        state.design_generation_stream_rx = Some(rx);
        state.design_generation_anim_frame = 7;
    }

    let _ = design_generation_completed(&mut app, Err("bad plan".to_string()));

    let state = app.active_design_state().unwrap();
    assert!(!state.design_generation_loading);
    assert!(state.design_generation_stream_rx.is_none());
    assert_eq!(state.design_generation_anim_frame, 0);
    assert_eq!(state.design_generation_summary.as_deref(), Some("bad plan"));
    assert!(
        state.design_generation_logs.iter().any(|line| line.contains("[PLAN] failed bad plan"))
    );
    assert!(state.design_chat_messages.iter().any(|message| message.content.contains("生成失败")));
}

#[test]
fn completed_success_builds_canvas_queues_modules_and_starts_page_generation() {
    let mut app = app_with_design();
    {
        let state = app.active_design_state_mut().unwrap();
        state.design_generation_loading = true;
        state.design_generation_brief = "brief".to_string();
        state.design_generation_stream_cursor = 1;
    }

    let _ = design_generation_completed(
        &mut app,
        Ok(DesignPlanExecutionResult {
            plan: plan(),
            logs: vec!["streamed".to_string(), "remaining".to_string()],
        }),
    );

    let state = app.active_design_state().unwrap();
    assert!(state.design_generation_loading);
    assert_eq!(state.design_generation_pages.len(), 1);
    assert_eq!(state.design_generation_pages[0].status, DesignGenerationStatus::Queued);
    let module = &state.design_generation_pages[0].modules[0];
    assert_eq!(module.status, DesignGenerationStatus::Queued);
    assert!(!module.is_generating);
    assert!(module.generated_doc.is_none());
    assert!(module.logs.is_empty());
    assert!(module.target_frame_options.contains(&"page-1-module-1".to_string()));
    assert!(state.doc.find_element("page-1-module-1").is_some());
    assert!(state.design_generation_summary.as_deref().unwrap().contains("已启动按页面并行生成"));
    assert!(
        state
            .design_generation_logs
            .iter()
            .any(|line| line.contains("[PLAN] parsed pages=1 modules=1"))
    );
    assert_eq!(state.design_generation_stream_cursor, 2);
}
