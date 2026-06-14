#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("page_generation_tests"));
}

use super::page_generation::{
    aggregate_design_page, design_page_generated, generate_design_page, regenerate_design_page,
};
use crate::app::App;
use crate::app::message::design::editor::DesignModuleExecutionResult;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{
    DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus, DesignState,
};

fn element(id: &str, kind: &str, children: Vec<DesignElement>) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "type": kind,
        "name": id,
        "width": 100,
        "height": 80,
        "children": children
    }))
    .unwrap()
}

fn text_element(id: &str, content: &str) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "type": "text",
        "content": content,
        "visible": true
    }))
    .unwrap()
}

fn placeholder(id: &str) -> DesignElement {
    element(
        id,
        "frame",
        vec![
            text_element(&format!("{id}-status"), ""),
            element(&format!("{id}-badge"), "rect", Vec::new()),
            text_element(&format!("{id}-badge-text"), ""),
            text_element(&format!("{id}-slot-hint"), ""),
            text_element(&format!("{id}-status-id"), ""),
        ],
    )
}

fn generated_doc(id: &str, child_id: &str) -> DesignDoc {
    DesignDoc {
        children: vec![
            serde_json::from_value(serde_json::json!({
                "id": id,
                "type": "frame",
                "name": "Generated",
                "width": 320,
                "height": 240,
                "children": [
                    { "id": child_id, "type": "text", "content": "Generated copy" }
                ]
            }))
            .unwrap(),
        ],
        ..DesignDoc::default()
    }
}

fn module(
    module_id: &str,
    target_frame_id: &str,
    status: DesignGenerationStatus,
) -> DesignGenerationModule {
    DesignGenerationModule {
        module_id: module_id.to_string(),
        title: module_id.to_string(),
        description: "description".to_string(),
        status,
        target_frame_id: target_frame_id.to_string(),
        target_frame_options: vec![target_frame_id.to_string()],
        generated_doc: None,
        is_generating: false,
        logs: Vec::new(),
    }
}

fn page(modules: Vec<DesignGenerationModule>) -> DesignGenerationPage {
    DesignGenerationPage {
        frame_id: "page-1".to_string(),
        title: "Home".to_string(),
        objective: "objective".to_string(),
        status: DesignGenerationStatus::Queued,
        modules,
    }
}

fn app_with_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

fn state_with_page() -> DesignState {
    let mut state = DesignState::new(DesignDoc {
        children: vec![element("page-1", "frame", vec![placeholder("target")])],
        ..DesignDoc::default()
    });
    state.design_generation_brief = "brief".to_string();
    state.design_generation_parallel_pages = 1;
    state.design_generation_pages =
        vec![page(vec![module("hero", "target", DesignGenerationStatus::Queued)])];
    state
}

#[test]
fn generate_page_reports_missing_page_and_noops_without_active_state() {
    let mut app = App::new().0;
    let _ = generate_design_page(&mut app, "missing".to_string(), "hero".to_string());
    assert!(app.active_design_state().is_none());

    let mut app = app_with_state(DesignState::new(DesignDoc::default()));
    let _ = generate_design_page(&mut app, "missing".to_string(), "hero".to_string());
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("未找到页面规划。")
    );
}

#[test]
fn generate_page_marks_page_modules_running_and_placeholder_running() {
    let mut app = app_with_state(state_with_page());

    let _ = generate_design_page(&mut app, "page-1".to_string(), "hero".to_string());

    let state = app.active_design_state().unwrap();
    let page = &state.design_generation_pages[0];
    let module = &page.modules[0];
    assert_eq!(page.status, DesignGenerationStatus::Running);
    assert_eq!(module.status, DesignGenerationStatus::Running);
    assert!(module.is_generating);
    assert!(state.design_generation_loading);
    assert!(state.design_generation_summary.as_deref().unwrap().contains("正在生成页面"));
    assert_eq!(
        state.doc.find_element("target-badge-text").unwrap().content.as_deref(),
        Some("running")
    );
    assert!(module.logs.iter().any(|line| line.contains("[PAGE:Home] start")));
}

#[test]
fn regenerate_page_uses_same_generation_path() {
    let mut app = app_with_state(state_with_page());

    let _ = regenerate_design_page(&mut app, "page-1".to_string(), "hero".to_string());

    assert_eq!(
        app.active_design_state().unwrap().design_generation_pages[0].modules[0].status,
        DesignGenerationStatus::Running
    );
}

#[test]
fn generate_page_respects_existing_running_page_limit() {
    let mut state = state_with_page();
    state.design_generation_pages.push(DesignGenerationPage {
        frame_id: "page-2".to_string(),
        title: "Other".to_string(),
        objective: String::new(),
        status: DesignGenerationStatus::Running,
        modules: vec![{
            let mut running = module("other", "other-target", DesignGenerationStatus::Running);
            running.is_generating = true;
            running
        }],
    });
    let mut app = app_with_state(state);

    let _ = generate_design_page(&mut app, "page-1".to_string(), "hero".to_string());

    let page = &app.active_design_state().unwrap().design_generation_pages[0];
    assert_eq!(page.status, DesignGenerationStatus::Queued);
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("页面任务并行数已满，当前页面已排队。")
    );
}

#[test]
fn page_generated_success_fills_page_and_completes_generation() {
    let mut state = state_with_page();
    state.design_generation_loading = true;
    state.design_generation_pages[0].status = DesignGenerationStatus::Running;
    state.design_generation_pages[0].modules[0].status = DesignGenerationStatus::Running;
    state.design_generation_pages[0].modules[0].is_generating = true;
    let mut app = app_with_state(state);

    let _ = design_page_generated(
        &mut app,
        "page-1".to_string(),
        "hero".to_string(),
        Ok(DesignModuleExecutionResult {
            doc: generated_doc("generated-page", "generated-child"),
            logs: vec!["log one".to_string()],
        }),
    );

    let state = app.active_design_state().unwrap();
    assert!(!state.design_generation_loading);
    assert_eq!(state.design_generation_pages[0].status, DesignGenerationStatus::Filled);
    assert_eq!(state.design_generation_pages[0].modules[0].status, DesignGenerationStatus::Filled);
    assert!(!state.design_generation_pages[0].modules[0].is_generating);
    assert!(state.doc.find_element("generated-child").is_some());
    assert!(state.design_generation_summary.as_deref().unwrap().contains("页面并行生成完成"));
    assert!(
        state.design_chat_messages.iter().any(|message| message.content.contains("Designed brief"))
    );
}

#[test]
fn page_generated_error_marks_page_failed_and_records_logs() {
    let mut state = state_with_page();
    state.design_generation_loading = true;
    state.design_generation_pages[0].status = DesignGenerationStatus::Running;
    state.design_generation_pages[0].modules[0].status = DesignGenerationStatus::Running;
    state.design_generation_pages[0].modules[0].is_generating = true;
    let mut app = app_with_state(state);

    let _ = design_page_generated(
        &mut app,
        "page-1".to_string(),
        "hero".to_string(),
        Err("boom".to_string()),
    );

    let state = app.active_design_state().unwrap();
    assert!(!state.design_generation_loading);
    assert_eq!(state.design_generation_pages[0].status, DesignGenerationStatus::Failed);
    assert_eq!(state.design_generation_pages[0].modules[0].status, DesignGenerationStatus::Failed);
    assert_eq!(
        state.design_generation_summary.as_deref(),
        Some("页面并行生成完成：0 个模块已回填，1 个失败，0 个未完成。")
    );
    assert!(
        state.design_generation_pages[0].modules[0].logs.iter().any(|line| line.contains("boom"))
    );
    assert_eq!(
        state.doc.find_element("target-badge-text").unwrap().content.as_deref(),
        Some("failed")
    );
}

#[test]
fn page_generated_import_failure_marks_failed_when_canvas_target_missing() {
    let mut state = state_with_page();
    state.doc = DesignDoc::default();
    state.design_generation_loading = true;
    state.design_generation_pages[0].status = DesignGenerationStatus::Running;
    state.design_generation_pages[0].modules[0].status = DesignGenerationStatus::Running;
    state.design_generation_pages[0].modules[0].is_generating = true;
    let mut app = app_with_state(state);

    let _ = design_page_generated(
        &mut app,
        "page-1".to_string(),
        "hero".to_string(),
        Ok(DesignModuleExecutionResult {
            doc: generated_doc("generated-page", "generated-child"),
            logs: Vec::new(),
        }),
    );

    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_pages[0].status, DesignGenerationStatus::Failed);
    assert!(state.design_generation_summary.as_deref().is_some());
}

#[test]
fn aggregate_page_imports_generated_module_or_reports_missing_doc() {
    let mut state = state_with_page();
    state.design_generation_pages[0].modules[0].generated_doc =
        Some(generated_doc("generated-module", "module-child"));
    let mut app = app_with_state(state);

    let _ = aggregate_design_page(&mut app, "page-1".to_string(), "hero".to_string());

    let state = app.active_design_state().unwrap();
    assert_eq!(
        state.design_generation_pages[0].modules[0].status,
        DesignGenerationStatus::Aggregated
    );
    assert!(state.doc.find_element("module-child").is_some());
    assert!(state.design_generation_summary.as_deref().unwrap().contains("已导入到指定画布位置"));

    let mut state = state_with_page();
    state.design_generation_pages[0].modules[0].generated_doc = None;
    let mut app = app_with_state(state);
    let _ = aggregate_design_page(&mut app, "page-1".to_string(), "hero".to_string());
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("模块“hero”还没有可导入的生成结果。")
    );
}

#[test]
fn aggregate_page_reports_missing_module() {
    let mut app = app_with_state(state_with_page());

    let _ = aggregate_design_page(&mut app, "missing".to_string(), "missing".to_string());

    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("未找到模块规划。")
    );
}
