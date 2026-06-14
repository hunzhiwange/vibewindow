#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("file_tree_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn preview_tab(path: &str) -> crate::app::PreviewTab {
    crate::app::PreviewTab {
        path: path.to_string(),
        title: path.to_string(),
        content: String::new(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: crate::app::components::editor::Editor::new("", "txt"),
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
fn file_tree_right_click_and_close_manage_menu_state() {
    let mut app = app();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeRightClicked(
            "/tmp/a.txt".to_string(),
            "tree".to_string(),
            10.0,
            20.0,
        ),
    );

    assert_eq!(app.file_tree_menu_path.as_deref(), Some("/tmp/a.txt"));
    assert_eq!(app.file_tree_menu_source.as_deref(), Some("tree"));
    assert_eq!(app.file_tree_menu_anchor, Some(iced::Point::new(10.0, 20.0)));

    let _ =
        super::handle(&mut app, crate::app::message::project::ProjectMessage::FileTreeMenuClose);

    assert!(app.file_tree_menu_path.is_none());
    assert!(app.file_tree_menu_source.is_none());
    assert!(app.file_tree_menu_anchor.is_none());
}

#[test]
fn rename_changed_and_cancel_manage_rename_state() {
    let mut app = app();
    app.file_tree_rename_path = Some("/tmp/a.txt".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeRenameChanged("b.txt".to_string()),
    );
    assert_eq!(app.file_tree_rename_value, "b.txt");

    let _ =
        super::handle(&mut app, crate::app::message::project::ProjectMessage::FileTreeRenameCancel);

    assert!(app.file_tree_rename_path.is_none());
    assert!(app.file_tree_rename_value.is_empty());
}

#[test]
fn drag_start_and_end_transfer_pending_drop_state() {
    let mut app = app();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeDragStart(
            "/tmp/a.txt".to_string(),
            Some((3, 4)),
        ),
    );
    assert_eq!(app.dragging_file_paths, vec!["/tmp/a.txt".to_string()]);
    assert_eq!(app.dragging_file_position, Some((3, 4)));

    let _ = super::handle(&mut app, crate::app::message::project::ProjectMessage::FileTreeDragEnd);

    assert!(app.dragging_file_paths.is_empty());
    assert_eq!(app.pending_drop_file_paths, vec!["/tmp/a.txt".to_string()]);
    assert_eq!(app.pending_drop_file_position, Some((3, 4)));
}

#[test]
fn rename_save_rejects_invalid_names() {
    let mut app = app();
    app.file_tree_rename_path = Some("/tmp/a.txt".to_string());
    app.file_tree_rename_value = "../b".to_string();

    let _ =
        super::handle(&mut app, crate::app::message::project::ProjectMessage::FileTreeRenameSave);

    assert_eq!(app.error_message.as_deref(), Some("文件名不合法"));
    assert_eq!(app.file_tree_rename_path.as_deref(), Some("/tmp/a.txt"));
}

#[test]
fn rename_completed_updates_preview_paths_on_success() {
    let mut app = app();
    app.project_path = Some("/tmp/project".to_string());
    app.preview_tabs.push(preview_tab("/tmp/project/src/old.rs"));
    app.preview_tabs.push(preview_tab("/tmp/project/src/old.rs/nested.txt"));
    app.active_preview_path = Some("/tmp/project/src/old.rs".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeRenameCompleted {
            old_path: "/tmp/project/src/old.rs".to_string(),
            result: Ok("/tmp/project/src/new.rs".to_string()),
        },
    );

    assert_eq!(app.preview_tabs[0].path, "/tmp/project/src/new.rs");
    assert_eq!(app.preview_tabs[1].path, "/tmp/project/src/new.rs/nested.txt");
    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/project/src/new.rs"));
}

#[test]
fn file_tree_action_without_menu_path_is_ignored() {
    let mut app = app();
    app.file_tree_menu_path = None;

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeAction(
            crate::app::message::project::FileTreeAction::Rename,
        ),
    );

    assert!(app.file_tree_rename_path.is_none());
}

#[test]
fn copy_and_cut_actions_update_file_tree_clipboard() {
    let mut app = app();
    app.file_tree_menu_path = Some("/tmp/a.txt".to_string());
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeAction(
            crate::app::message::project::FileTreeAction::Copy,
        ),
    );
    let clipboard = app.file_tree_clipboard.as_ref().expect("clipboard");
    assert!(matches!(clipboard.mode, crate::app::state::FileTreeClipboardMode::Copy));
    assert_eq!(clipboard.src_path, "/tmp/a.txt");

    app.file_tree_menu_path = Some("/tmp/b.txt".to_string());
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeAction(
            crate::app::message::project::FileTreeAction::Cut,
        ),
    );
    let clipboard = app.file_tree_clipboard.as_ref().expect("clipboard");
    assert!(matches!(clipboard.mode, crate::app::state::FileTreeClipboardMode::Cut));
    assert_eq!(clipboard.src_path, "/tmp/b.txt");
}

#[test]
fn find_in_folder_action_creates_active_find_tab_and_open_tab() {
    let mut app = app();
    app.project_path = Some("/tmp/project".to_string());
    app.file_tree_menu_path = Some("/tmp/project/src".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeAction(
            crate::app::message::project::FileTreeAction::FindInFolder,
        ),
    );

    assert_eq!(app.find_results_tabs.len(), 1);
    assert_eq!(app.find_results_tabs[0].scope_path, "/tmp/project/src");
    assert_eq!(app.active_find_results_tab_id, Some(app.find_results_tabs[0].id.clone()));
    assert!(app.show_file_manager);
    assert!(app.active_tab_id.as_deref().is_some_and(|id| id.starts_with("find:")));
}

#[test]
fn rename_action_primes_rename_fields() {
    let mut app = app();
    app.file_tree_menu_path = Some("/tmp/project/src/main.rs".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeAction(
            crate::app::message::project::FileTreeAction::Rename,
        ),
    );

    assert_eq!(app.file_tree_rename_path.as_deref(), Some("/tmp/project/src/main.rs"));
    assert_eq!(app.file_tree_rename_value, "main.rs");
}

#[test]
fn paste_completed_cut_success_clears_clipboard() {
    let mut app = app();
    app.file_tree_clipboard = Some(crate::app::state::FileTreeClipboard {
        mode: crate::app::state::FileTreeClipboardMode::Cut,
        src_path: "/tmp/a.txt".to_string(),
    });

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreePasteCompleted {
            clear_clipboard: true,
            result: Ok(()),
        },
    );

    assert!(app.file_tree_clipboard.is_none());
}

#[test]
fn paste_and_delete_failures_set_error_message() {
    let mut app = app();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreePasteCompleted {
            clear_clipboard: false,
            result: Err("denied".to_string()),
        },
    );
    assert_eq!(app.error_message.as_deref(), Some("粘贴失败: denied"));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeDeleteCompleted(Err(
            "missing".to_string()
        )),
    );
    assert_eq!(app.error_message.as_deref(), Some("删除失败: missing"));
}
