use super::export_ops::{export_finished, export_jpeg, export_png, export_svg};
use crate::app::App;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::MindMapTab;

fn app() -> App {
    App::new().0
}

fn app_with_tab() -> App {
    let mut app = app();
    app.mindmap_tabs.push(MindMapTab::new(
        "tab-1".to_string(),
        "Tab".to_string(),
        None,
        model::default_doc(),
    ));
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn export_finished_records_error_and_ignores_success() {
    let mut app = app();

    let _ = export_finished(&mut app, Ok(()));
    assert_eq!(app.error_message, None);

    let _ = export_finished(&mut app, Err("failed".to_string()));
    assert_eq!(app.error_message.as_deref(), Some("failed"));
}

#[test]
fn export_entry_points_noop_without_active_tab() {
    let mut app = app();

    let _ = export_svg(&mut app);
    let _ = export_png(&mut app);
    let _ = export_jpeg(&mut app);

    assert!(app.mindmap_tabs.is_empty());
    assert_eq!(app.error_message, None);
}

#[test]
fn export_svg_prepares_active_tab_before_async_save() {
    let mut app = app_with_tab();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_action_menu = true;
    tab.show_export_menu = true;

    let _ = export_svg(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_action_menu);
    assert!(!tab.show_export_menu);
}

#[test]
fn export_png_and_jpeg_prepare_active_tab_before_async_save() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().show_export_menu = true;

    let _ = export_png(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().show_export_menu);

    app.active_mindmap_tab_mut().unwrap().show_export_menu = true;
    let _ = export_jpeg(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().show_export_menu);
}
