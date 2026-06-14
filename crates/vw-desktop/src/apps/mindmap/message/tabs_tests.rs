use super::tabs::{close_tab, ensure_top_tab, mindmap_app_tab_id, new_blank_tab, sync_top_tabs};
use crate::app::{App, AppTab, Screen};
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::MindMapTab;

fn app() -> App {
    App::new().0
}

fn tab(id: &str, title: &str) -> MindMapTab {
    MindMapTab::new(id.to_string(), title.to_string(), None, model::default_doc())
}

#[test]
fn mindmap_app_tab_id_keeps_stable_prefix() {
    assert_eq!(mindmap_app_tab_id("mindmap-3"), "mindmap:mindmap-3");
}

#[test]
fn ensure_top_tab_removes_apps_tab_and_activates_mindmap_screen() {
    let mut app = app();
    app.open_tabs.push(AppTab {
        id: "apps".to_string(),
        title: "Apps".to_string(),
        screen: Screen::Apps,
        project_path: Some("/tmp/project".to_string()),
    });

    ensure_top_tab(&mut app, "mindmap-1", "Map 1");

    assert!(app.open_tabs.iter().all(|tab| tab.id != "apps"));
    assert_eq!(app.active_tab_id.as_deref(), Some("mindmap:mindmap-1"));
    assert!(matches!(app.screen, Screen::MindMapTool));
    let top = app.open_tabs.iter().find(|tab| tab.id == "mindmap:mindmap-1").unwrap();
    assert_eq!(top.title, "Map 1");
    assert_eq!(top.project_path, None);
}

#[test]
fn ensure_top_tab_updates_existing_top_tab() {
    let mut app = app();
    app.open_tabs.push(AppTab {
        id: "mindmap:mindmap-1".to_string(),
        title: "Old".to_string(),
        screen: Screen::Apps,
        project_path: Some("/tmp/project".to_string()),
    });

    ensure_top_tab(&mut app, "mindmap-1", "Updated");

    assert_eq!(app.open_tabs.iter().filter(|tab| tab.id == "mindmap:mindmap-1").count(), 1);
    let top = app.open_tabs.iter().find(|tab| tab.id == "mindmap:mindmap-1").unwrap();
    assert_eq!(top.title, "Updated");
    assert!(matches!(top.screen, Screen::MindMapTool));
    assert_eq!(top.project_path, None);
}

#[test]
fn sync_top_tabs_adds_all_tabs_and_prefers_active_id() {
    let mut app = app();
    app.mindmap_tabs = vec![tab("mindmap-1", "One"), tab("mindmap-2", "Two")];
    app.mindmap_active_tab_id = Some("mindmap-2".to_string());

    sync_top_tabs(&mut app);

    assert!(app.open_tabs.iter().any(|tab| tab.id == "mindmap:mindmap-1"));
    assert!(app.open_tabs.iter().any(|tab| tab.id == "mindmap:mindmap-2"));
    assert_eq!(app.active_tab_id.as_deref(), Some("mindmap:mindmap-2"));
    assert!(matches!(app.screen, Screen::MindMapTool));
}

#[test]
fn new_blank_tab_skips_existing_numeric_id() {
    let mut app = app();
    app.mindmap_tabs.push(tab("mindmap-2", "Existing"));

    let _ = new_blank_tab(&mut app);

    assert_eq!(app.mindmap_tabs.len(), 2);
    assert_eq!(app.mindmap_tabs[1].id, "mindmap-3");
    assert_eq!(app.mindmap_active_tab_id.as_deref(), Some("mindmap-3"));
    assert_eq!(app.active_tab_id.as_deref(), Some("mindmap:mindmap-3"));
}

#[test]
fn close_tab_removes_closed_tab_and_selects_last_remaining_tab() {
    let mut app = app();
    app.mindmap_tabs = vec![tab("mindmap-1", "One"), tab("mindmap-2", "Two")];
    app.mindmap_active_tab_id = Some("mindmap-1".to_string());
    ensure_top_tab(&mut app, "mindmap-1", "One");
    ensure_top_tab(&mut app, "mindmap-2", "Two");

    let _ = close_tab(&mut app, "mindmap-1");

    assert_eq!(app.mindmap_tabs.len(), 1);
    assert_eq!(app.mindmap_active_tab_id.as_deref(), Some("mindmap-2"));
    assert!(app.open_tabs.iter().all(|tab| tab.id != "mindmap:mindmap-1"));
}

#[test]
fn close_tab_clears_active_id_when_last_tab_is_removed() {
    let mut app = app();
    app.mindmap_tabs = vec![tab("mindmap-1", "One")];
    app.mindmap_active_tab_id = Some("mindmap-1".to_string());
    ensure_top_tab(&mut app, "mindmap-1", "One");

    let _ = close_tab(&mut app, "mindmap-1");

    assert!(app.mindmap_tabs.is_empty());
    assert_eq!(app.mindmap_active_tab_id, None);
    assert!(app.open_tabs.iter().all(|tab| tab.id != "mindmap:mindmap-1"));
}
