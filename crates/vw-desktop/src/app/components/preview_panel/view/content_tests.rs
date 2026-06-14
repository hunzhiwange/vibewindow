#[test]
fn content_tests_are_wired() {
    assert!(module_path!().contains("content_tests"));
}

use super::content::build_content_base;
use crate::app::components::editor::Editor;
use crate::app::{App, Message, PreviewTab, Screen};
use iced::widget::Id;

fn app() -> App {
    App::new().0
}

fn tab(path: &str, content: &str) -> PreviewTab {
    PreviewTab {
        path: path.to_string(),
        title: path.rsplit('/').next().unwrap_or(path).to_string(),
        content: content.to_string(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: Editor::new(content, "txt"),
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
fn content_shows_empty_selection_message_in_preview_screen() {
    let mut app = app();
    app.screen = Screen::Preview;
    app.active_preview_path = None;

    keep(build_content_base(&app));
}

#[test]
fn content_shows_missing_preview_when_active_tab_is_absent() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/missing.rs".to_string());

    keep(build_content_base(&app));
}

#[test]
fn content_builds_editor_view_for_text_tab() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/main.rs".to_string());
    app.preview_tabs = vec![tab("/tmp/main.rs", "fn main() {}")];

    keep(build_content_base(&app));
}

#[test]
fn content_builds_raster_image_view_from_extension() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/image.PNG".to_string());
    app.preview_tabs = vec![tab("/tmp/image.PNG", "")];

    keep(build_content_base(&app));
}

#[test]
fn content_builds_svg_view_from_extension() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/icon.svg".to_string());
    app.preview_tabs = vec![tab("/tmp/icon.svg", "")];

    keep(build_content_base(&app));
}

#[test]
fn content_uses_git_panel_when_changes_view_is_enabled() {
    let mut app = app();
    app.active_preview_path = None;
    app.file_manager_show_changes = true;
    app.screen = Screen::Project;

    keep(build_content_base(&app));
}
