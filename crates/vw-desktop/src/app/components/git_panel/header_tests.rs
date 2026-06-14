use crate::app::state::GitWorktreeOption;

#[test]
fn task_718_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("header_tests.rs"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn worktree(directory: &str, label: &str, branch: Option<&str>) -> GitWorktreeOption {
    GitWorktreeOption {
        directory: directory.to_string(),
        label: label.to_string(),
        branch: branch.map(str::to_string),
    }
}

#[test]
fn view_builds_with_default_branch_and_no_worktree_picker() {
    let app = app();

    let _element = super::header::view(&app, vec!["src/lib.rs".to_string()]);
}

#[test]
fn view_builds_with_long_branch_fullscreen_and_open_worktree_menu() {
    let mut app = app();
    app.selected_branch = Some("feature/super-long-branch-name".to_string());
    app.git_diff_fullscreen = true;
    app.show_git_diff_summary = true;
    app.show_git_filter_options = true;
    app.git_worktree_menu_open = true;
    app.selected_git_worktree_directory = Some("/repo/wt-two".to_string());
    app.git_worktree_options = vec![
        worktree("/repo/main", "main worktree · extra metadata", Some("main")),
        worktree(
            "/repo/wt-two",
            "secondary worktree with very long label · detail",
            Some("feature/extremely-long-branch"),
        ),
    ];

    let _element =
        super::header::view(&app, vec!["src/lib.rs".to_string(), "src/main.rs".to_string()]);
}

#[test]
fn view_builds_worktree_picker_without_selected_directory_or_branch() {
    let mut app = app();
    app.git_worktree_options = vec![
        worktree("/repo/a", "alpha", None),
        worktree("/repo/b", "beta · ignored suffix", Some("")),
    ];

    let _element = super::header::view(&app, Vec::new());
}
