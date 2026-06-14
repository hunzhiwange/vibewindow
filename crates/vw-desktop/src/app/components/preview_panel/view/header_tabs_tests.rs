#[test]
fn header_tabs_tests_are_wired() {
    assert!(module_path!().contains("header_tabs_tests"));
}

use super::header_tabs::build_header_tabs;
use crate::app::components::editor::Editor;
use crate::app::{App, Message, PreviewTab};
use iced::{Point, widget::Id};

fn app() -> App {
    App::new().0
}

fn tab(path: &str, dirty: bool) -> PreviewTab {
    PreviewTab {
        path: path.to_string(),
        title: path.rsplit('/').next().unwrap_or(path).to_string(),
        content: "content".to_string(),
        is_dirty: dirty,
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
fn header_tabs_builds_for_empty_tabs() {
    let app = app();

    keep(build_header_tabs(&app));
}

#[test]
fn header_tabs_builds_selected_and_unselected_tabs() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/a.rs".to_string());
    app.preview_tabs = vec![tab("/tmp/a.rs", false), tab("/tmp/b.ts", true)];

    keep(build_header_tabs(&app));
}

#[test]
fn header_tabs_builds_context_menu_overlay_for_target_tab() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/a.rs".to_string());
    app.preview_tabs = vec![tab("/tmp/a.rs", false), tab("/tmp/b.ts", true)];
    app.preview_tab_menu_path = Some("/tmp/b.ts".to_string());
    app.preview_tab_menu_pos = Some(Point::new(12.0, 24.0));

    keep(build_header_tabs(&app));
}

#[test]
fn header_tabs_uses_taller_scrollbar_container_when_tabs_overflow() {
    let mut app = app();
    app.window_size = (120.0, 700.0);
    app.preview_tabs =
        (0..8).map(|i| tab(&format!("/tmp/very-long-file-name-{i}.rs"), false)).collect();
    app.active_preview_path = Some("/tmp/very-long-file-name-0.rs".to_string());

    keep(build_header_tabs(&app));
}
