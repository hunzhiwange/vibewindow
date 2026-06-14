use crate::app::message::git::GitMessage;
use crate::app::state::GitWorktreeOption;

#[test]
fn refresh_worktree_options_without_project_clears_state() {
    let (mut app, _task) = crate::app::App::new();
    app.project_path = None;
    app.git_worktree_options = vec![GitWorktreeOption {
        directory: "/repo".to_string(),
        label: "repo".to_string(),
        branch: Some("main".to_string()),
    }];
    app.selected_git_worktree_directory = Some("/repo".to_string());
    app.git_worktree_options_loading = true;
    app.git_worktree_options_project_path = Some("/repo".to_string());
    app.git_worktree_menu_open = true;

    let _task = super::refresh::update(&mut app, GitMessage::RefreshWorktreeOptions);

    assert!(app.git_worktree_options.is_empty());
    assert_eq!(app.selected_git_worktree_directory, None);
    assert!(!app.git_worktree_options_loading);
    assert_eq!(app.git_worktree_options_project_path, None);
    assert!(!app.git_worktree_menu_open);
}

#[test]
fn worktree_options_ready_ignores_stale_project_path() {
    let (mut app, _task) = crate::app::App::new();
    app.git_worktree_options_project_path = Some("/current".to_string());
    app.git_worktree_options_loading = true;

    let _task = super::refresh::update(
        &mut app,
        GitMessage::WorktreeOptionsReady {
            project_path: "/stale".to_string(),
            result: Ok(vec![GitWorktreeOption {
                directory: "/stale".to_string(),
                label: "stale".to_string(),
                branch: Some("old".to_string()),
            }]),
        },
    );

    assert!(app.git_worktree_options.is_empty());
    assert!(app.git_worktree_options_loading);
}

#[test]
fn worktree_options_ready_selects_matching_directory_and_branch() {
    let (mut app, _task) = crate::app::App::new();
    app.git_worktree_options_project_path = Some("/project".to_string());
    app.git_worktree_options_loading = true;
    app.selected_git_worktree_directory = Some("/worktree/".to_string());

    let _task = super::refresh::update(
        &mut app,
        GitMessage::WorktreeOptionsReady {
            project_path: "/project".to_string(),
            result: Ok(vec![
                GitWorktreeOption {
                    directory: "/project".to_string(),
                    label: "main".to_string(),
                    branch: Some("main".to_string()),
                },
                GitWorktreeOption {
                    directory: "/worktree".to_string(),
                    label: "feature".to_string(),
                    branch: Some("feature".to_string()),
                },
            ]),
        },
    );

    assert!(!app.git_worktree_options_loading);
    assert_eq!(app.selected_git_worktree_directory.as_deref(), Some("/worktree"));
    assert_eq!(app.selected_branch.as_deref(), Some("feature"));
}

#[test]
fn worktree_options_ready_error_resets_options_and_reports_error() {
    let (mut app, _task) = crate::app::App::new();
    app.git_worktree_options_project_path = Some("/project".to_string());
    app.git_worktree_options_loading = true;
    app.git_worktree_menu_open = true;
    app.git_worktree_options = vec![GitWorktreeOption {
        directory: "/old".to_string(),
        label: "old".to_string(),
        branch: None,
    }];

    let _task = super::refresh::update(
        &mut app,
        GitMessage::WorktreeOptionsReady {
            project_path: "/project".to_string(),
            result: Err("boom".to_string()),
        },
    );

    assert!(app.git_worktree_options.is_empty());
    assert_eq!(app.selected_git_worktree_directory, None);
    assert!(!app.git_worktree_menu_open);
    assert_eq!(app.error_message.as_deref(), Some("读取 Git worktree 失败: boom"));
}

#[test]
fn select_git_worktree_resets_diff_state_and_updates_branch() {
    let (mut app, _task) = crate::app::App::new();
    app.git_worktree_menu_open = true;
    app.git_changed_files = vec!["src/lib.rs".to_string()];
    app.git_changed_files_loading = true;
    app.git_changed_files_repo_path = Some("/old".to_string());
    app.git_diff_file_metas_loading = true;
    app.git_diff_file_metas_repo_path = Some("/old".to_string());
    app.git_diff_contents.insert("src/lib.rs".to_string(), ("old".to_string(), "new".to_string()));
    app.git_diff_contents_loading.insert("src/lib.rs".to_string());
    app.git_diff_selected_lines.push(crate::app::state::GitDiffSelectedLine {
        file: "src/lib.rs".to_string(),
        line: 1,
        is_old: false,
        text: "new".to_string(),
    });
    app.git_diff_selected_range = Some(crate::app::state::GitDiffLineRange {
        file: "src/lib.rs".to_string(),
        start: 1,
        end: 2,
        is_old: false,
    });
    app.git_diff_comment_draft = Some(crate::app::state::GitDiffCommentDraft {
        range: crate::app::state::GitDiffLineRange {
            file: "src/lib.rs".to_string(),
            start: 1,
            end: 1,
            is_old: false,
        },
        editor: iced::widget::text_editor::Content::with_text("note"),
    });

    let _task = super::refresh::update(
        &mut app,
        GitMessage::SelectGitWorktree(GitWorktreeOption {
            directory: "/next".to_string(),
            label: "next".to_string(),
            branch: Some("feat".to_string()),
        }),
    );

    assert!(!app.git_worktree_menu_open);
    assert_eq!(app.selected_git_worktree_directory.as_deref(), Some("/next"));
    assert_eq!(app.selected_branch.as_deref(), Some("feat"));
    assert!(app.git_changed_files.is_empty());
    assert!(!app.git_changed_files_loading);
    assert_eq!(app.git_changed_files_repo_path, None);
    assert!(app.git_diff_file_metas.is_empty());
    assert!(!app.git_diff_file_metas_loading);
    assert_eq!(app.git_diff_file_metas_repo_path, None);
    assert!(app.git_diff_contents.is_empty());
    assert!(app.git_diff_contents_loading.is_empty());
    assert!(app.git_diff_selected_lines.is_empty());
    assert!(app.git_diff_selected_range.is_none());
    assert!(app.git_diff_comment_draft.is_none());
}

#[test]
fn changed_files_ready_ignores_stale_repo_path() {
    let (mut app, _task) = crate::app::App::new();
    app.git_changed_files_repo_path = Some("/current".to_string());
    app.git_changed_files_loading = true;

    let _task = super::refresh::update(
        &mut app,
        GitMessage::ChangedFilesReady {
            repo_path: Some("/old".to_string()),
            files: vec!["a.rs".to_string()],
        },
    );

    assert!(app.git_changed_files.is_empty());
    assert!(app.git_changed_files_loading);
}

#[test]
fn changed_files_ready_updates_files_and_clears_selection() {
    let (mut app, _task) = crate::app::App::new();
    app.git_changed_files_repo_path = Some("/repo".to_string());
    app.git_changed_files_loading = true;
    app.file_manager_changes_refreshing = true;
    app.git_diff_selected_lines.push(crate::app::state::GitDiffSelectedLine {
        file: "old.rs".to_string(),
        line: 1,
        is_old: false,
        text: "x".to_string(),
    });
    app.git_diff_selected_range = Some(crate::app::state::GitDiffLineRange {
        file: "old.rs".to_string(),
        start: 2,
        end: 1,
        is_old: false,
    });
    app.git_diff_comment_draft = Some(crate::app::state::GitDiffCommentDraft {
        range: crate::app::state::GitDiffLineRange {
            file: "old.rs".to_string(),
            start: 1,
            end: 1,
            is_old: false,
        },
        editor: iced::widget::text_editor::Content::with_text("draft"),
    });

    let _task = super::refresh::update(
        &mut app,
        GitMessage::ChangedFilesReady {
            repo_path: Some("/repo".to_string()),
            files: vec!["src/main.rs".to_string()],
        },
    );

    assert_eq!(app.git_changed_files, vec!["src/main.rs"]);
    assert!(!app.git_changed_files_loading);
    assert!(!app.file_manager_changes_refreshing);
    assert!(app.git_diff_selected_lines.is_empty());
    assert!(app.git_diff_selected_range.is_none());
    assert!(app.git_diff_comment_draft.is_none());
}

#[test]
fn diff_file_metas_ready_ignores_stale_repo_path() {
    let (mut app, _task) = crate::app::App::new();
    app.git_diff_file_metas_repo_path = Some("/current".to_string());
    app.git_diff_file_metas_loading = true;

    let _task = super::refresh::update(
        &mut app,
        GitMessage::DiffFileMetasReady { repo_path: Some("/old".to_string()), metas: Vec::new() },
    );

    assert!(app.git_diff_file_metas.is_empty());
    assert!(app.git_diff_file_metas_loading);
}

#[test]
fn diff_content_ready_ignores_unknown_file_but_clears_loading() {
    let (mut app, _task) = crate::app::App::new();
    app.git_diff_file_metas_repo_path = Some("/repo".to_string());
    app.git_diff_contents_loading.insert("src/lib.rs".to_string());

    let _task = super::refresh::update(
        &mut app,
        GitMessage::DiffContentReady {
            repo_path: Some("/repo".to_string()),
            file: "src/lib.rs".to_string(),
            old_content: "old".to_string(),
            new_content: "new".to_string(),
        },
    );

    assert!(!app.git_diff_contents_loading.contains("src/lib.rs"));
    assert!(!app.git_diff_contents.contains_key("src/lib.rs"));
}
