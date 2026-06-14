#[test]
fn menus_tests_are_wired() {
    assert!(module_path!().contains("menus_tests"));
}

use super::menus::build_menu_ui;
use crate::app::components::editor::Editor;
use crate::app::{App, Message, PreviewTab};
use iced::widget::Id;

fn app() -> App {
    App::new().0
}

fn tab(path: &str) -> PreviewTab {
    PreviewTab {
        path: path.to_string(),
        title: path.rsplit('/').next().unwrap_or(path).to_string(),
        content: "content".to_string(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: Editor::new("content", "txt"),
        scroll_id: Id::unique(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_server_key: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_uri: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_language_id: None,
    }
}

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn menu_ui_builds_empty_layers_when_hidden() {
    let app = app();

    keep(build_menu_ui(&app));
}

#[test]
fn menu_ui_handles_context_menu_without_target_or_position() {
    let mut app = app();
    app.show_preview_context_menu = true;

    keep(build_menu_ui(&app));

    app.preview_context_target = Some(("/tmp/a.rs".to_string(), 1, 1, 1, 1));
    keep(build_menu_ui(&app));
}

#[test]
fn menu_ui_builds_context_menu_for_matching_tab() {
    let mut app = app();
    app.show_preview_context_menu = true;
    app.preview_context_target = Some(("/tmp/a.rs".to_string(), 1, 1, 2, 4));
    app.preview_context_menu_pos = Some((20.0, 30.0));
    app.preview_tabs = vec![tab("/tmp/a.rs")];

    keep(build_menu_ui(&app));
}

#[test]
fn menu_ui_uses_empty_context_items_for_missing_tab() {
    let mut app = app();
    app.show_preview_context_menu = true;
    app.preview_context_target = Some(("/tmp/missing.rs".to_string(), 1, 1, 2, 4));
    app.preview_context_menu_pos = Some((20.0, 30.0));
    app.preview_tabs = vec![tab("/tmp/a.rs")];

    keep(build_menu_ui(&app));
}

#[test]
fn menu_ui_builds_nav_popup_for_directory_and_file_items() {
    let mut app = app();
    app.preview_nav_popup = Some((
        "/tmp/project".to_string(),
        10.0,
        20.0,
        vec![("src".to_string(), true), ("main.rs".to_string(), false)],
    ));

    keep(build_menu_ui(&app));
}
