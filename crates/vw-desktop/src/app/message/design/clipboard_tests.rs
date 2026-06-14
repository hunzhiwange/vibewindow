use crate::app::App;
use crate::app::message::design::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;
use iced::{Point, Vector};

fn new_app_with_design_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

fn element_with_child() -> DesignElement {
    DesignElement {
        id: "source".to_string(),
        kind: "rect".to_string(),
        x: 10.0,
        y: 20.0,
        children: vec![DesignElement { id: "child".to_string(), ..Default::default() }],
        ..Default::default()
    }
}

#[test]
fn copy_and_cut_noop_without_selection() {
    let mut app = App::new().0;

    let _copy_task = super::update(&mut app, DesignMessage::Copy);
    let _cut_task = super::update(&mut app, DesignMessage::Cut);

    assert!(app.active_design_state().is_none());
}

#[test]
fn copy_and_cut_return_tasks_when_selection_exists() {
    let mut doc = DesignDoc::default();
    doc.children.push(element_with_child());
    let mut state = DesignState::new(doc);
    state.selected_element_id = Some("source".to_string());
    let mut app = new_app_with_design_state(state);

    let _copy_task = super::update(&mut app, DesignMessage::Copy);
    let _cut_task = super::update(&mut app, DesignMessage::Cut);

    assert_eq!(app.active_design_state().unwrap().selected_element_id.as_deref(), Some("source"));
}

#[test]
fn paste_requests_clipboard_read() {
    let mut app = App::new().0;

    let _task = super::update(&mut app, DesignMessage::Paste);

    assert!(app.active_design_state().is_none());
}

#[test]
fn clipboard_content_received_adds_element_with_offset_and_new_ids() {
    let mut app = new_app_with_design_state(DesignState::new(DesignDoc::default()));
    let json = serde_json::to_string(&element_with_child()).unwrap();

    let _task = super::update(&mut app, DesignMessage::ClipboardContentReceived(Some(json)));

    let state = app.active_design_state().unwrap();
    let pasted = state.doc.children.last().unwrap();
    assert_eq!(pasted.x, 30.0);
    assert_eq!(pasted.y, 40.0);
    assert!(pasted.id.starts_with("paste_"));
    assert!(pasted.children[0].id.starts_with("paste_"));
    assert_ne!(pasted.id, pasted.children[0].id);
}

#[test]
fn clipboard_content_received_uses_paste_anchor_before_cursor() {
    let mut state = DesignState::new(DesignDoc::default());
    state.pan = Vector::new(10.0, 20.0);
    state.zoom = 2.0;
    state.paste_anchor = Some(Point::new(50.0, 80.0));
    let mut app = new_app_with_design_state(state);
    app.cursor_position = Point::new(200.0, 200.0);
    let json = serde_json::to_string(&element_with_child()).unwrap();

    let _task = super::update(&mut app, DesignMessage::ClipboardContentReceived(Some(json)));

    let state = app.active_design_state().unwrap();
    let pasted = state.doc.children.last().unwrap();
    assert_eq!(Point::new(pasted.x, pasted.y), Point::new(20.0, 30.0));
    assert_eq!(state.paste_anchor, None);
}

#[test]
fn clipboard_content_received_uses_cursor_when_anchor_is_missing() {
    let mut state = DesignState::new(DesignDoc::default());
    state.pan = Vector::new(5.0, 10.0);
    state.zoom = 5.0;
    let mut app = new_app_with_design_state(state);
    app.cursor_position = Point::new(55.0, 60.0);
    let json = serde_json::to_string(&element_with_child()).unwrap();

    let _task = super::update(&mut app, DesignMessage::ClipboardContentReceived(Some(json)));

    let pasted = app.active_design_state().unwrap().doc.children.last().unwrap();
    assert_eq!(Point::new(pasted.x, pasted.y), Point::new(10.0, 10.0));
}

#[test]
fn clipboard_content_received_ignores_empty_invalid_or_inactive_state() {
    let mut app = new_app_with_design_state(DesignState::new(DesignDoc::default()));

    let _none_task = super::update(&mut app, DesignMessage::ClipboardContentReceived(None));
    let _invalid_task =
        super::update(&mut app, DesignMessage::ClipboardContentReceived(Some("bad".to_string())));
    let _other_task = super::update(&mut app, DesignMessage::ZoomIn);

    assert!(app.active_design_state().unwrap().doc.children.is_empty());

    let mut inactive = App::new().0;
    let json = serde_json::to_string(&element_with_child()).unwrap();
    let _task = super::update(&mut inactive, DesignMessage::ClipboardContentReceived(Some(json)));
    assert!(inactive.active_design_state().is_none());
}
