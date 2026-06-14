use crate::app::message::design::{DesignMessage, io};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;
use crate::app::{App, AppTab, Screen};
use std::path::PathBuf;

fn element(id: &str) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

fn app_with_design_state(tab_id: &str, doc: DesignDoc) -> App {
    let mut app = App::new().0;
    app.active_tab_id = Some(tab_id.to_string());
    app.open_tabs.push(AppTab {
        id: tab_id.to_string(),
        title: "Existing".to_string(),
        screen: Screen::Design,
        project_path: Some("project".to_string()),
    });
    app.design_states.insert(tab_id.to_string(), DesignState::new(doc));
    app
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("io_tests"));
}

#[test]
fn new_reuses_empty_active_design_tab() {
    let mut app = App::new().0;
    app.active_tab_id = Some("empty-design".to_string());
    app.open_tabs.push(AppTab {
        id: "empty-design".to_string(),
        title: "Old".to_string(),
        screen: Screen::Design,
        project_path: Some("project".to_string()),
    });

    let _ = io::update(&mut app, DesignMessage::New);

    assert_eq!(app.active_tab_id.as_deref(), Some("empty-design"));
    assert!(app.design_states.contains_key("empty-design"));
    let tab = app.open_tabs.iter().find(|tab| tab.id == "empty-design").unwrap();
    assert_eq!(tab.title, "设计");
    assert_eq!(tab.screen, Screen::Design);
    assert!(tab.project_path.is_none());
}

#[test]
fn new_creates_default_design_tab_when_active_tab_unusable() {
    let mut app = app_with_design_state("busy", DesignDoc::default());

    let _ = io::update(&mut app, DesignMessage::New);

    assert_eq!(app.active_tab_id.as_deref(), Some("design"));
    assert!(app.design_states.contains_key("design"));
    assert!(app.open_tabs.iter().any(|tab| tab.id == "design" && tab.title == "设计"));
}

#[test]
fn file_opened_uses_empty_active_tab_and_sets_title_from_path() {
    let mut app = App::new().0;
    app.active_tab_id = Some("slot".to_string());
    app.open_tabs.push(AppTab {
        id: "slot".to_string(),
        title: "Slot".to_string(),
        screen: Screen::Design,
        project_path: Some("project".to_string()),
    });
    let path = PathBuf::from("/tmp/sample.pen");
    let doc = DesignDoc {
        version: "1.0".to_string(),
        children: vec![element("shape")],
        ..Default::default()
    };

    let _ = io::update(&mut app, DesignMessage::FileOpened(Ok((doc, Some(path.clone())))));

    assert_eq!(app.active_tab_id.as_deref(), Some("slot"));
    let state = app.active_design_state().unwrap();
    assert_eq!(state.file_path.as_ref(), Some(&path));
    assert_eq!(state.doc.children[0].id, "shape");
    assert_eq!(app.open_tabs.iter().find(|tab| tab.id == "slot").unwrap().title, "sample.pen");
}

#[test]
fn file_opened_creates_unique_tab_when_active_slot_is_busy() {
    let mut app = app_with_design_state("Design 1", DesignDoc::default());
    app.design_states.insert("Design 1 (1)".to_string(), DesignState::new(DesignDoc::default()));

    let doc = DesignDoc { version: "1.0".to_string(), ..Default::default() };
    let _ = io::update(&mut app, DesignMessage::FileOpened(Ok((doc, None))));

    assert_eq!(app.active_tab_id.as_deref(), Some("Design 3"));
    assert!(app.design_states.contains_key("Design 3"));
    assert!(app.open_tabs.iter().any(|tab| tab.id == "Design 3"));
}

#[test]
fn file_opened_error_sets_error_message_except_cancelled() {
    let mut app = App::new().0;

    let _ = io::update(&mut app, DesignMessage::FileOpened(Err("parse".to_string())));
    assert_eq!(app.error_message.as_deref(), Some("打开设计文件失败：parse"));

    app.error_message = None;
    let _ = io::update(&mut app, DesignMessage::FileOpened(Err("Cancelled".to_string())));
    assert!(app.error_message.is_none());
}

#[test]
fn file_saved_updates_active_state_path() {
    let mut app = app_with_design_state("design", DesignDoc::default());
    let path = PathBuf::from("/tmp/out.json");

    let _ = io::update(&mut app, DesignMessage::FileSaved(Some(path.clone())));

    assert_eq!(app.active_design_state().unwrap().file_path.as_ref(), Some(&path));
}
