use crate::app::state::{
    ConventionalCommitType, GitDiffContextMenuState, GitDiffLineRange, GitDiffSelectedLine,
};

#[test]
fn refresh_after_repo_mutation_clears_stale_loading_guards() {
    let (mut app, _task) = crate::app::App::new();

    app.git_changed_files_loading = true;
    app.git_changed_files_repo_path = Some("/tmp/repo".to_string());
    app.git_diff_file_metas_loading = true;
    app.git_diff_file_metas_repo_path = Some("/tmp/repo".to_string());
    app.git_diff_contents_loading.insert("src/main.rs".to_string());

    let _task = super::shared::refresh_git_panel_data_after_repo_mutation_task(&mut app);

    assert!(!app.git_changed_files_loading);
    assert_eq!(app.git_changed_files_repo_path, None);
    assert!(!app.git_diff_file_metas_loading);
    assert_eq!(app.git_diff_file_metas_repo_path, None);
    assert!(app.git_diff_contents_loading.is_empty());
}

#[test]
fn build_selected_commit_request_formats_conventional_subject_and_body() {
    let (mut app, _task) = crate::app::App::new();
    app.git_commit_type = Some(ConventionalCommitType::Fix);
    app.git_commit_scope = "desktop".to_string();
    app.git_commit_message = " refresh git state ".to_string();
    app.git_commit_description = " body line ".to_string();
    app.staged_files_selected = vec!["src/main.rs".to_string()];
    app.staged_hunks_selected = vec![("src/lib.rs".to_string(), 2)];
    app.staged_lines_selected = vec![("src/lib.rs".to_string(), 4)];
    app.staged_old_lines_selected = vec![("src/lib.rs".to_string(), 3)];

    let request = super::shared::build_selected_commit_request(&app).expect("message is valid");

    assert_eq!(request.message, "fix(desktop): refresh git state\n\nbody line");
    assert_eq!(request.selected_files, vec!["src/main.rs"]);
    assert_eq!(request.selected_hunks, vec![("src/lib.rs".to_string(), 2)]);
    assert_eq!(request.selected_lines, vec![("src/lib.rs".to_string(), 4)]);
    assert_eq!(request.selected_old_lines, vec![("src/lib.rs".to_string(), 3)]);
}

#[test]
fn build_selected_commit_request_rejects_blank_message() {
    let (mut app, _task) = crate::app::App::new();
    app.git_commit_type = None;
    app.git_commit_message = "   ".to_string();
    app.git_commit_description = "\n\t".to_string();

    let error = super::shared::build_selected_commit_request(&app).expect_err("blank is invalid");

    assert_eq!(error, "提交消息不能为空");
}

#[test]
fn reset_commit_form_state_clears_selection_and_restores_defaults() {
    let (mut app, _task) = crate::app::App::new();
    app.staged_files_selected = vec!["a.rs".to_string()];
    app.staged_hunks_selected = vec![("a.rs".to_string(), 1)];
    app.staged_lines_selected = vec![("a.rs".to_string(), 2)];
    app.staged_old_lines_selected = vec![("a.rs".to_string(), 3)];
    app.expanded_hunks.push(("a.rs".to_string(), 1));
    app.context_expansions.insert(("a.rs".to_string(), 0), (1, 2));
    app.git_commit_message = "msg".to_string();
    app.git_commit_type = Some(ConventionalCommitType::Docs);
    app.git_commit_scope = "scope".to_string();
    app.git_commit_description = "body".to_string();
    app.git_commit_description_editor = iced::widget::text_editor::Content::with_text("body");

    super::shared::reset_commit_form_state(&mut app);

    assert!(app.staged_files_selected.is_empty());
    assert!(app.staged_hunks_selected.is_empty());
    assert!(app.staged_lines_selected.is_empty());
    assert!(app.staged_old_lines_selected.is_empty());
    assert!(app.expanded_files.is_empty());
    assert!(app.expanded_hunks.is_empty());
    assert!(app.context_expansions.is_empty());
    assert!(app.git_commit_message.is_empty());
    assert_eq!(app.git_commit_type, Some(ConventionalCommitType::Feat));
    assert!(app.git_commit_scope.is_empty());
    assert!(app.git_commit_description.is_empty());
    assert!(app.git_commit_description_editor.text().is_empty());
}

#[test]
fn text_too_large_for_code_editor_detects_limits_and_nul() {
    assert!(!super::shared::text_too_large_for_code_editor("small\ntext"));
    assert!(super::shared::text_too_large_for_code_editor(&"a".repeat(1_000_001)));
    assert!(super::shared::text_too_large_for_code_editor(&format!("{}\n", "a".repeat(20_001))));
    assert!(super::shared::text_too_large_for_code_editor("abc\0def"));
}

#[test]
fn normalize_range_orders_start_and_end() {
    let range = super::shared::normalize_range(GitDiffLineRange {
        file: "src/lib.rs".to_string(),
        start: 9,
        end: 3,
        is_old: true,
    });

    assert_eq!(range.start, 3);
    assert_eq!(range.end, 9);
    assert!(range.is_old);
}

#[test]
fn diff_context_target_range_prefers_menu_inside_selected_range() {
    let (mut app, _task) = crate::app::App::new();
    app.git_diff_selected_range =
        Some(GitDiffLineRange { file: "src/lib.rs".to_string(), start: 8, end: 4, is_old: false });
    app.git_diff_context_menu = Some(GitDiffContextMenuState {
        file: "src/lib.rs".to_string(),
        line: 6,
        is_old: false,
        x: 1.0,
        y: 2.0,
    });

    let range = super::shared::diff_context_target_range(&app).expect("range should resolve");

    assert_eq!(range.file, "src/lib.rs");
    assert_eq!(range.start, 4);
    assert_eq!(range.end, 8);
    assert!(!range.is_old);
}

#[test]
fn diff_context_target_range_falls_back_to_menu_line() {
    let (mut app, _task) = crate::app::App::new();
    app.git_diff_selected_range =
        Some(GitDiffLineRange { file: "other.rs".to_string(), start: 1, end: 3, is_old: false });
    app.git_diff_context_menu = Some(GitDiffContextMenuState {
        file: "src/lib.rs".to_string(),
        line: 6,
        is_old: true,
        x: 1.0,
        y: 2.0,
    });

    let range = super::shared::diff_context_target_range(&app).expect("range should resolve");

    assert_eq!(range.file, "src/lib.rs");
    assert_eq!(range.start, 6);
    assert_eq!(range.end, 6);
    assert!(range.is_old);
}

#[test]
fn diff_context_target_stage_lines_deduplicates_selected_lines() {
    let (mut app, _task) = crate::app::App::new();
    app.git_diff_selected_lines = vec![
        GitDiffSelectedLine {
            file: "src/lib.rs".to_string(),
            line: 5,
            is_old: false,
            text: "a".to_string(),
        },
        GitDiffSelectedLine {
            file: "src/lib.rs".to_string(),
            line: 5,
            is_old: false,
            text: "a".to_string(),
        },
        GitDiffSelectedLine {
            file: "src/lib.rs".to_string(),
            line: 2,
            is_old: true,
            text: "b".to_string(),
        },
    ];

    let lines = super::shared::diff_context_target_stage_lines(&app);

    assert_eq!(
        lines,
        vec![("src/lib.rs".to_string(), 2, true), ("src/lib.rs".to_string(), 5, false)]
    );
}

#[test]
fn extend_and_clear_stage_selection_for_lines_are_idempotent() {
    let (mut app, _task) = crate::app::App::new();
    let lines = vec![
        ("src/lib.rs".to_string(), 2, false),
        ("src/lib.rs".to_string(), 2, false),
        ("src/lib.rs".to_string(), 1, true),
    ];

    super::shared::extend_stage_selection_for_lines(&mut app, &lines);

    assert_eq!(app.staged_lines_selected, vec![("src/lib.rs".to_string(), 2)]);
    assert_eq!(app.staged_old_lines_selected, vec![("src/lib.rs".to_string(), 1)]);

    super::shared::clear_stage_selection_for_lines(
        &mut app,
        &[("src/lib.rs".to_string(), 2, false)],
    );

    assert!(app.staged_lines_selected.is_empty());
    assert_eq!(app.staged_old_lines_selected, vec![("src/lib.rs".to_string(), 1)]);
}
