use crate::app::App;
use crate::app::message::design::{DesignMessage, ImageImportPayload, images};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{DesignState, ImageImportTarget};
use iced::widget::image::Handle;

fn element(id: &str) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        fill: Some(serde_json::json!([
            {"type": "image", "url": " first.png ", "enabled": true},
            {"type": "image", "url": "disabled.png", "enabled": false},
            [{"type": "image", "url": "nested.png"}]
        ])),
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("images_tests"));
}

#[test]
fn load_image_tasks_collects_enabled_unique_sources() {
    let fill = serde_json::json!([
        {"type": "image", "url": "a.png"},
        {"type": "image", "url": "a.png"},
        {"type": "image", "url": "b.png", "enabled": true},
        {"type": "image", "url": "c.png", "enabled": false}
    ]);

    let tasks = images::load_image_tasks_from_fill_value(&fill);

    assert_eq!(tasks.len(), 2);
}

#[test]
fn load_image_tasks_from_document_walks_children() {
    let child = element("child");
    let mut parent = element("parent");
    parent.children.push(child);
    let doc =
        DesignDoc { version: "1.0".to_string(), children: vec![parent], ..Default::default() };

    let tasks = images::load_image_tasks_from_document(&doc);

    assert_eq!(tasks.len(), 2);
}

#[test]
fn image_loaded_success_stores_handle_and_size() {
    let mut app = app_with_doc(DesignDoc::default());
    let result = Ok((Handle::from_bytes(vec![1, 2, 3]), Some((12, 34))));

    let task =
        images::update(&mut app, DesignMessage::ImageLoaded("asset.png".to_string(), result));

    assert!(task.is_some());
    let state = app.active_design_state().unwrap();
    assert!(state.doc.images.contains_key("asset.png"));
    assert_eq!(state.doc.image_sizes.get("asset.png"), Some(&(12, 34)));
}

#[test]
fn image_import_dialog_messages_update_state() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = images::update(&mut app, DesignMessage::ImportImageElement);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.image_import_target, Some(ImageImportTarget::Element));
    assert!(state.image_import_input.is_empty());
    assert!(!state.image_import_loading);

    let _ = images::update(&mut app, DesignMessage::ImportFillImage("shape".to_string(), 1));
    let state = app.active_design_state().unwrap();
    assert_eq!(
        state.image_import_target,
        Some(ImageImportTarget::Fill { element_id: "shape".to_string(), fill_index: 1 })
    );

    let _ = images::update(&mut app, DesignMessage::ImageImportInputChanged("  url  ".to_string()));
    assert_eq!(app.active_design_state().unwrap().image_import_input, "  url  ");

    let _ = images::update(
        &mut app,
        DesignMessage::ImageImportClipboardReceived(Some(" pasted ".to_string())),
    );
    let state = app.active_design_state().unwrap();
    assert_eq!(state.image_import_input, "pasted");
    assert!(state.image_import_error.is_none());

    let _ = images::update(&mut app, DesignMessage::ImageImportClipboardReceived(None));
    assert!(app.active_design_state().unwrap().image_import_error.is_some());

    let _ = images::update(&mut app, DesignMessage::CloseImageImportDialog);
    let state = app.active_design_state().unwrap();
    assert!(state.image_import_target.is_none());
    assert!(state.image_import_input.is_empty());
}

#[test]
fn image_import_resolution_applies_fill_payload_or_error() {
    let doc = DesignDoc {
        version: "1.0".to_string(),
        children: vec![DesignElement {
            id: "shape".to_string(),
            kind: "rect".to_string(),
            fill: Some(serde_json::json!([
                {"type": "image", "url": "old.png", "mode": "stretch"}
            ])),
            ..serde_json::from_value(serde_json::json!({})).unwrap()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    app.active_design_state_mut().unwrap().image_import_loading = true;

    let payload = ImageImportPayload {
        target: ImageImportTarget::Fill { element_id: "shape".to_string(), fill_index: 0 },
        source: "new.png".to_string(),
        bytes: vec![1, 2, 3],
        size_opt: Some((10, 20)),
    };
    let _ = images::update(&mut app, DesignMessage::ImageImportResolved(Ok(payload)));

    let state = app.active_design_state().unwrap();
    assert!(!state.image_import_loading);
    assert!(state.image_import_target.is_none());
    assert!(state.doc.images.contains_key("new.png"));
    assert_eq!(state.doc.image_sizes.get("new.png"), Some(&(10, 20)));
    assert!(
        state
            .doc
            .find_element("shape")
            .unwrap()
            .fill
            .as_ref()
            .unwrap()
            .to_string()
            .contains("new.png")
    );

    let _ = images::update(&mut app, DesignMessage::ImageImportResolved(Err("bad".to_string())));
    assert_eq!(app.active_design_state().unwrap().image_import_error.as_deref(), Some("bad"));
}

#[test]
fn sticky_note_messages_toggle_dialog_and_create_task() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = images::update(&mut app, DesignMessage::OpenStickyNoteDialog);
    assert!(app.active_design_state().unwrap().sticky_note_dialog_open);

    let _ = images::update(
        &mut app,
        DesignMessage::CreateStickyNote(crate::app::views::design::models::StickyNoteKind::Prompt),
    );
    let state = app.active_design_state().unwrap();
    assert!(!state.sticky_note_dialog_open);
    assert_eq!(
        state.sticky_note_dialog_default_kind,
        crate::app::views::design::models::StickyNoteKind::Prompt
    );
}
