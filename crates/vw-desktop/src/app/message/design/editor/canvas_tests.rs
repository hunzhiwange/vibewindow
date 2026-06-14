#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("canvas_tests"));
}

use super::canvas::{
    apply_module_doc_to_canvas, apply_page_doc_to_canvas, build_target_frame_options,
    collect_retry_error_context, find_generation_module_index, find_generation_page_index,
    find_generation_page_mut, normalize_target_frame_id, sync_module_placeholder_status,
};
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

fn state_with_placeholder(target_id: &str) -> DesignState {
    let frame = element(
        target_id,
        "frame",
        vec![
            text_element(&format!("{target_id}-status"), "old status"),
            element(&format!("{target_id}-badge"), "rect", Vec::new()),
            text_element(&format!("{target_id}-badge-text"), "old badge"),
            text_element(&format!("{target_id}-slot-hint"), "old hint"),
            text_element(&format!("{target_id}-status-id"), "old id"),
        ],
    );
    DesignState::new(DesignDoc { children: vec![frame], ..DesignDoc::default() })
}

fn generated_frame(id: &str) -> DesignDoc {
    DesignDoc {
        children: vec![
            serde_json::from_value(serde_json::json!({
                "id": id,
                "type": "frame",
                "name": "Generated",
                "width": 320,
                "height": 240,
                "fill": "#FFFFFF",
                "class": "flex gap-2",
                "children": [
                    { "id": "generated-child", "type": "text", "content": "Generated copy" }
                ]
            }))
            .unwrap(),
        ],
        ..DesignDoc::default()
    }
}

fn module(module_id: &str, target_frame_id: &str) -> DesignGenerationModule {
    DesignGenerationModule {
        module_id: module_id.to_string(),
        title: module_id.to_string(),
        description: String::new(),
        status: DesignGenerationStatus::Queued,
        target_frame_id: target_frame_id.to_string(),
        target_frame_options: vec![target_frame_id.to_string()],
        generated_doc: None,
        is_generating: false,
        logs: Vec::new(),
    }
}

#[test]
fn normalize_target_frame_id_trims_json_and_plain_values() {
    assert_eq!(normalize_target_frame_id(r#" { "id": "Frame-42" } "#), "Frame-42");
    assert_eq!(normalize_target_frame_id(r#" "module-a", "#), "module-a");
    assert_eq!(normalize_target_frame_id("  PlainId  "), "PlainId");
}

#[test]
fn find_generation_page_and_module_accept_normalized_case_insensitive_ids() {
    let mut pages = vec![DesignGenerationPage {
        frame_id: "Page-1".to_string(),
        title: "Page".to_string(),
        objective: String::new(),
        status: DesignGenerationStatus::Planned,
        modules: vec![module("Hero", "hero-frame")],
    }];

    assert_eq!(find_generation_page_index(&pages, r#"{ "id": "page-1" }"#), Some(0));
    assert_eq!(find_generation_module_index(&pages[0], "HERO-FRAME"), Some(0));
    assert!(find_generation_page_mut(&mut pages, "Page-1").is_some());
}

#[test]
fn collect_retry_error_context_only_includes_failed_modules() {
    let mut failed = module("hero", "hero-frame");
    failed.title = "Hero".to_string();
    failed.status = DesignGenerationStatus::Failed;
    failed.logs = vec!["start".to_string(), "render failed because bad json".to_string()];
    let mut queued = module("footer", "footer-frame");
    queued.title = "Footer".to_string();
    queued.logs = vec!["error but not failed status".to_string()];
    let page = DesignGenerationPage {
        frame_id: "page".to_string(),
        title: String::new(),
        objective: String::new(),
        status: DesignGenerationStatus::Planned,
        modules: vec![failed, queued],
    };

    let context = collect_retry_error_context(&page);

    assert!(context.contains("模块 Hero"));
    assert!(context.contains("render failed"));
    assert!(!context.contains("Footer"));
}

#[test]
fn build_target_frame_options_merges_defaults_canvas_ids_and_current_target() {
    let doc = DesignDoc {
        children: vec![element("page", "frame", vec![element("nested", "frame", Vec::new())])],
        ..DesignDoc::default()
    };

    let options = build_target_frame_options(&doc, 0, 1, Some(r#"{ "id": "custom" }"#));

    assert!(options.contains(&"page-0-module-1".to_string()));
    assert!(options.contains(&"page".to_string()));
    assert!(options.contains(&"nested".to_string()));
    assert!(options.contains(&"custom".to_string()));
}

#[test]
fn sync_module_placeholder_status_updates_badge_and_status_text() {
    let mut state = state_with_placeholder("target");
    state.design_generation_anim_frame = 3;

    sync_module_placeholder_status(&mut state, "TARGET", DesignGenerationStatus::Running);

    let status = state.doc.find_element("target-status").unwrap();
    let badge = state.doc.find_element("target-badge").unwrap();
    let badge_text = state.doc.find_element("target-badge-text").unwrap();
    assert_eq!(status.content.as_deref(), Some("生成中.. · 实时预览占位"));
    assert_eq!(badge_text.content.as_deref(), Some("running"));
    assert_eq!(badge.visible, Some(true));
    assert_eq!(badge.width, Some(serde_json::json!(92.0)));
}

#[test]
fn apply_module_doc_to_canvas_replaces_frame_and_hides_placeholder_text() {
    let mut state = state_with_placeholder("target");

    apply_module_doc_to_canvas(&mut state, r#"{ "id": "TARGET" }"#, &generated_frame("new"))
        .unwrap();

    let target = state.doc.find_element("target").unwrap();
    assert_eq!(target.name.as_deref(), Some("Generated"));
    assert_eq!(target.class.as_deref(), Some("flex gap-2"));
    assert_eq!(target.clip, Some(true));
    assert_eq!(target.children.len(), 1);
    assert!(state.doc.find_element("target-status").is_none());
}

#[test]
fn apply_module_doc_to_canvas_reports_nearby_candidates_when_missing() {
    let mut state = state_with_placeholder("hero-card");

    let error =
        apply_module_doc_to_canvas(&mut state, "hero", &generated_frame("new")).unwrap_err();

    assert!(error.contains("未找到目标模块占位 frame"));
    assert!(error.contains("hero-card"));
}

#[test]
fn apply_page_doc_to_canvas_replaces_existing_page_frame() {
    let mut state = state_with_placeholder("page-frame");

    apply_page_doc_to_canvas(&mut state, "page-frame", &generated_frame("generated-page")).unwrap();

    let page = state.doc.find_element("page-frame").unwrap();
    assert_eq!(page.name.as_deref(), Some("Generated"));
    assert_eq!(page.clip, Some(true));
    assert_eq!(page.children[0].id, "generated-child");
}
