use super::{MindMapMessage, ensure_initialized, update, view};
use crate::app::App;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapTab;

#[test]
fn mindmap_module_reexports_message_type() {
    assert!(format!("{:?}", MindMapMessage::New).contains("New"));
}

fn tab(id: &str) -> MindMapTab {
    MindMapTab::new(
        id.to_string(),
        format!("Tab {id}"),
        None,
        MindNode { text: id.to_string(), children: Vec::new() },
    )
}

#[test]
fn ensure_initialized_keeps_valid_active_tab() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs = vec![tab("first"), tab("second")];
    app.mindmap_active_tab_id = Some("second".to_string());

    let _task = ensure_initialized(&mut app);

    assert_eq!(app.mindmap_active_tab_id.as_deref(), Some("second"));
}

#[test]
fn ensure_initialized_repairs_missing_active_tab_to_first_tab() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs = vec![tab("first"), tab("second")];
    app.mindmap_active_tab_id = Some("missing".to_string());

    let _task = ensure_initialized(&mut app);

    assert_eq!(app.mindmap_active_tab_id.as_deref(), Some("first"));
}

#[test]
fn view_wrapper_renders_empty_and_active_states() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;
    let empty = view(&app);
    drop(empty);

    app.mindmap_tabs.push(tab("first"));
    app.mindmap_active_tab_id = Some("first".to_string());
    let _active = view(&app);
}

#[test]
fn update_wrapper_delegates_message_handling() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs = vec![tab("first")];
    app.mindmap_active_tab_id = Some("first".to_string());

    let _task = update(&mut app, MindMapMessage::ToggleZoomMenu);

    assert!(app.mindmap_tabs[0].show_zoom_menu);
}
