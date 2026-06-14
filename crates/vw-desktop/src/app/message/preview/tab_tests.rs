#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("tab_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn preview_tab(path: &str, content: &str) -> crate::app::PreviewTab {
    crate::app::PreviewTab {
        path: path.to_string(),
        title: std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_string(),
        content: content.to_string(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: crate::app::components::editor::Editor::new(content, "txt"),
        scroll_id: iced::widget::Id::unique(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_server_key: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_uri: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_language_id: None,
    }
}

#[test]
fn select_empty_path_clears_active_preview() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/a.txt".to_string());

    let _ = super::tab::update(&mut app, super::PreviewMessage::Select(String::new()));

    assert!(app.active_preview_path.is_none());
}

#[test]
fn select_path_sets_active_preview_and_focus() {
    let mut app = app();

    let _ = super::tab::update(&mut app, super::PreviewMessage::Select("/tmp/a.txt".to_string()));

    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/a.txt"));
    assert!(matches!(app.focus_area, crate::app::FocusArea::Preview));
}

#[test]
fn close_active_preview_selects_last_remaining_tab() {
    let mut app = app();
    app.preview_tabs.push(preview_tab("/tmp/a.txt", "a"));
    app.preview_tabs.push(preview_tab("/tmp/b.txt", "b"));
    app.active_preview_path = Some("/tmp/a.txt".to_string());

    let _ = super::tab::update(&mut app, super::PreviewMessage::Close("/tmp/a.txt".to_string()));

    assert_eq!(app.preview_tabs.len(), 1);
    assert_eq!(app.preview_tabs[0].path, "/tmp/b.txt");
    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/b.txt"));
}

#[test]
fn tab_menu_close_left_removes_tabs_before_target() {
    let mut app = app();
    app.preview_tabs.push(preview_tab("/tmp/a.txt", "a"));
    app.preview_tabs.push(preview_tab("/tmp/b.txt", "b"));
    app.preview_tabs.push(preview_tab("/tmp/c.txt", "c"));
    app.active_preview_path = Some("/tmp/a.txt".to_string());
    app.preview_tab_menu_path = Some("/tmp/b.txt".to_string());
    app.preview_tab_menu_pos = Some(iced::Point::new(1.0, 2.0));

    let _ = super::tab::update(
        &mut app,
        super::PreviewMessage::TabMenuCloseLeft("/tmp/b.txt".to_string()),
    );

    assert_eq!(
        app.preview_tabs.iter().map(|tab| tab.path.as_str()).collect::<Vec<_>>(),
        vec!["/tmp/b.txt", "/tmp/c.txt"]
    );
    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/b.txt"));
    assert!(app.preview_tab_menu_path.is_none());
    assert!(app.preview_tab_menu_pos.is_none());
}

#[test]
fn tab_menu_close_right_removes_tabs_after_target() {
    let mut app = app();
    app.preview_tabs.push(preview_tab("/tmp/a.txt", "a"));
    app.preview_tabs.push(preview_tab("/tmp/b.txt", "b"));
    app.preview_tabs.push(preview_tab("/tmp/c.txt", "c"));
    app.active_preview_path = Some("/tmp/c.txt".to_string());

    let _ = super::tab::update(
        &mut app,
        super::PreviewMessage::TabMenuCloseRight("/tmp/b.txt".to_string()),
    );

    assert_eq!(
        app.preview_tabs.iter().map(|tab| tab.path.as_str()).collect::<Vec<_>>(),
        vec!["/tmp/a.txt", "/tmp/b.txt"]
    );
    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/b.txt"));
}

#[test]
fn tab_menu_close_all_clears_tabs_and_active_path() {
    let mut app = app();
    app.preview_tabs.push(preview_tab("/tmp/a.txt", "a"));
    app.preview_tabs.push(preview_tab("/tmp/b.txt", "b"));
    app.active_preview_path = Some("/tmp/a.txt".to_string());
    app.preview_tab_menu_path = Some("/tmp/a.txt".to_string());
    app.preview_tab_menu_pos = Some(iced::Point::new(1.0, 2.0));

    let _ = super::tab::update(&mut app, super::PreviewMessage::TabMenuCloseAll);

    assert!(app.preview_tabs.is_empty());
    assert!(app.active_preview_path.is_none());
    assert!(app.preview_tab_menu_path.is_none());
    assert!(app.preview_tab_menu_pos.is_none());
}

#[test]
fn open_loaded_updates_existing_tab_content() {
    let mut app = app();
    app.preview_tabs.push(preview_tab("/tmp/a.rs", "old"));

    let _ = super::tab::update(
        &mut app,
        super::PreviewMessage::OpenLoaded {
            path: "/tmp/a.rs".to_string(),
            content: "new".to_string(),
            truncated: true,
        },
    );

    let tab = &app.preview_tabs[0];
    assert_eq!(tab.content, "new");
    assert!(!tab.is_dirty);
    assert!(tab.truncated);
}

#[test]
fn save_file_without_active_preview_returns_without_state_change() {
    let mut app = app();
    app.active_preview_path = None;

    let _ = super::tab::update(&mut app, super::PreviewMessage::SaveFile);

    assert!(app.active_preview_path.is_none());
}

#[test]
fn save_file_path_for_clean_tab_closes_context_menu() {
    let mut app = app();
    app.preview_tabs.push(preview_tab("/tmp/a.txt", "a"));
    app.show_preview_context_menu = true;

    let _ = super::tab::update(
        &mut app,
        super::PreviewMessage::SaveFilePath { path: "/tmp/a.txt".to_string(), notify: false },
    );

    assert!(!app.show_preview_context_menu);
    assert!(!app.preview_tabs[0].is_dirty);
}
