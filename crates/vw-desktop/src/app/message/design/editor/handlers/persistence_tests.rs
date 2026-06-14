#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("persistence_tests"));
}

use super::persistence;
use crate::app::message::design::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{
    DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus, DesignState,
};
use crate::app::{App, Message};

fn app_with_design_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

fn module_doc(id: &str) -> DesignDoc {
    DesignDoc {
        children: vec![DesignElement {
            kind: "frame".to_string(),
            id: id.to_string(),
            width: Some(serde_json::json!(320)),
            height: Some(serde_json::json!(180)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn page_with_module(generated_doc: Option<DesignDoc>) -> DesignGenerationPage {
    DesignGenerationPage {
        frame_id: "design-page-0".to_string(),
        title: "首页".to_string(),
        objective: "展示核心入口".to_string(),
        status: DesignGenerationStatus::Queued,
        modules: vec![DesignGenerationModule {
            module_id: "page-0-module-0".to_string(),
            title: "Hero / Main".to_string(),
            description: "展示主视觉".to_string(),
            status: DesignGenerationStatus::Queued,
            target_frame_id: "target-frame".to_string(),
            target_frame_options: vec!["target-frame".to_string()],
            generated_doc,
            is_generating: false,
            logs: Vec::new(),
        }],
    }
}

#[test]
fn saved_project_updates_path_and_summary() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));
    let path = std::path::PathBuf::from("/tmp/example.pen");

    let _ = persistence::design_project_pen_saved(&mut app, Ok(Some(path.clone())));

    let state = app.active_design_state().unwrap();
    assert_eq!(state.file_path.as_ref(), Some(&path));
    assert_eq!(
        state.design_generation_summary.as_deref(),
        Some("项目 .json 已保存: /tmp/example.pen")
    );
}

#[test]
fn saved_project_reports_cancel_and_error() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    let _ = persistence::design_project_pen_saved(&mut app, Ok(None));
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("已取消保存项目 .pen。")
    );

    let _ = persistence::design_project_pen_saved(&mut app, Err("disk full".to_string()));
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("disk full")
    );
}

#[test]
fn generated_page_save_result_updates_summary() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    let _ = persistence::generated_page_pen_saved(
        &mut app,
        Ok(Some(std::path::PathBuf::from("/tmp/module.pen"))),
    );
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("模块子文档已保存: /tmp/module.pen")
    );

    let _ = persistence::generated_page_pen_saved(&mut app, Ok(None));
    assert_eq!(
        app.active_design_state().unwrap().design_generation_summary.as_deref(),
        Some("已取消保存模块子文档。")
    );
}

#[test]
fn generated_page_import_success_sets_module_and_canvas() {
    let mut state = DesignState::new(DesignDoc {
        children: vec![DesignElement {
            kind: "frame".to_string(),
            id: "target-frame".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    state.design_generation_pages = vec![page_with_module(None)];
    let mut app = app_with_design_state(state);
    let imported = module_doc("imported-root");

    let task = persistence::generated_page_pen_imported(
        &mut app,
        "design-page-0".to_string(),
        "page-0-module-0".to_string(),
        Ok(imported),
    );

    let state = app.active_design_state().unwrap();
    let module = &state.design_generation_pages[0].modules[0];
    assert_eq!(module.status, DesignGenerationStatus::Generated);
    assert!(module.generated_doc.is_some());
    assert_eq!(
        state.design_generation_summary.as_deref(),
        Some("已导入 .json 子文档到模块“Hero / Main”。")
    );
    assert!(!state.doc.children.is_empty());
    let _ = task.map(|message| {
        assert!(matches!(message, Message::Design(DesignMessage::Snapshot)));
    });
}

#[test]
fn generated_page_import_error_keeps_module_queued() {
    let mut state = DesignState::new(DesignDoc::default());
    state.design_generation_pages = vec![page_with_module(None)];
    let mut app = app_with_design_state(state);

    let _ = persistence::generated_page_pen_imported(
        &mut app,
        "design-page-0".to_string(),
        "page-0-module-0".to_string(),
        Err("bad json".to_string()),
    );

    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_pages[0].modules[0].status, DesignGenerationStatus::Queued);
    assert_eq!(state.design_generation_summary.as_deref(), Some("bad json"));
}
