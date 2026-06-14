use crate::app::state::ConventionalCommitType;

#[test]
fn task_719_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("left_panel_tests.rs"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

#[test]
fn help_copy_lists_commit_format_and_common_types() {
    assert_eq!(super::left_panel::COMMIT_HELP_TITLE, "约定式提交");

    for expected in ["<类型>[可选作用域]: <摘要>", "feat:", "fix:", "docs:", "locale:"] {
        assert!(super::left_panel::COMMIT_HELP_TEXT.contains(expected), "{expected}");
    }
}

#[test]
fn view_builds_disabled_state_when_nothing_selected() {
    let mut app = app();
    app.git_commit_message = "add panel tests".to_string();
    app.git_commit_type = Some(ConventionalCommitType::Feat);

    let _element = super::left_panel::view(&app);
}

#[test]
fn view_builds_summary_mode_with_missing_summary_and_type() {
    let mut app = app();
    app.show_git_diff_summary = true;
    app.git_commit_type = None;
    app.git_commit_message = "   ".to_string();
    app.staged_files_selected.push("src/lib.rs".to_string());

    let _element = super::left_panel::view(&app);
}

#[test]
fn view_builds_ready_and_in_progress_commit_states() {
    let mut app = app();
    app.show_git_diff_summary = true;
    app.git_commit_type = Some(ConventionalCommitType::Fix);
    app.git_commit_scope = "git-panel".to_string();
    app.git_commit_message = "cover left panel".to_string();
    app.staged_hunks_selected.push(("src/lib.rs".to_string(), 1));

    let ready = super::left_panel::view(&app);
    drop(ready);

    app.git_commit_in_progress = true;
    app.file_manager_refresh_frame = 2;
    let _busy = super::left_panel::view(&app);
}
