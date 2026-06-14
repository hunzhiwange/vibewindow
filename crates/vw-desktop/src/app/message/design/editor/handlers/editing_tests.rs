#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("editing_tests"));
}

use super::editing::{edit_cancel, edit_content_changed, edit_start, edit_submit};
use crate::app::App;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;

fn text_element(id: &str, content: &str) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "type": "text",
        "content": content
    }))
    .unwrap()
}

fn app_with_design_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

#[test]
fn edit_start_selects_element_and_loads_editor_content() {
    let mut app = app_with_design_doc(DesignDoc::default());

    let _ = edit_start(&mut app, "title".to_string(), "Hello".to_string());

    let state = app.active_design_state().unwrap();
    assert_eq!(state.editing_id.as_deref(), Some("title"));
    assert_eq!(state.editing_content, "Hello");
    assert_eq!(state.editing_editor.text(), "Hello");
    assert_eq!(state.selected_element_id.as_deref(), Some("title"));
    assert!(state.selected_element_ids.contains("title"));
}

#[test]
fn edit_content_changed_replaces_buffer_without_changing_selection() {
    let mut app = app_with_design_doc(DesignDoc::default());
    let _ = edit_start(&mut app, "title".to_string(), "Old".to_string());

    let _ = edit_content_changed(&mut app, "New".to_string());

    let state = app.active_design_state().unwrap();
    assert_eq!(state.editing_id.as_deref(), Some("title"));
    assert_eq!(state.editing_content, "New");
    assert_eq!(state.editing_editor.text(), "New");
    assert_eq!(state.selected_element_id.as_deref(), Some("title"));
}

#[test]
fn edit_submit_updates_document_content_and_clears_editing_state() {
    let doc = DesignDoc { children: vec![text_element("title", "Old")], ..DesignDoc::default() };
    let mut app = app_with_design_doc(doc);
    let _ = edit_start(&mut app, "title".to_string(), "Old".to_string());
    let _ = edit_content_changed(&mut app, "New copy".to_string());

    let _ = edit_submit(&mut app);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.find_element("title").unwrap().content.as_deref(), Some("New copy"));
    assert_eq!(state.editing_id, None);
    assert!(state.editing_content.is_empty());
    assert!(state.editing_editor.text().is_empty());
    assert_eq!(state.selected_element_id.as_deref(), Some("title"));
}

#[test]
fn edit_cancel_keeps_document_content_and_clears_editor() {
    let doc =
        DesignDoc { children: vec![text_element("title", "Original")], ..DesignDoc::default() };
    let mut app = app_with_design_doc(doc);
    let _ = edit_start(&mut app, "title".to_string(), "Original".to_string());
    let _ = edit_content_changed(&mut app, "Discarded".to_string());

    let _ = edit_cancel(&mut app);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.find_element("title").unwrap().content.as_deref(), Some("Original"));
    assert_eq!(state.editing_id, None);
    assert!(state.editing_content.is_empty());
    assert!(state.editing_editor.text().is_empty());
    assert_eq!(state.selected_element_id.as_deref(), Some("title"));
}

#[test]
fn editing_handlers_are_noops_without_active_design_state() {
    let mut app = App::new().0;

    let _ = edit_start(&mut app, "title".to_string(), "Hello".to_string());
    let _ = edit_content_changed(&mut app, "Changed".to_string());
    let _ = edit_submit(&mut app);
    let _ = edit_cancel(&mut app);

    assert!(app.active_design_state().is_none());
}
