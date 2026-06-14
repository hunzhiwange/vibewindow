use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn normalize_path_trims_slashes_and_current_directory() {
    assert_eq!(normalize_path("./src/main.rs"), "src/main.rs");
    assert_eq!(normalize_path("/src/main.rs"), "src/main.rs");
    assert_eq!(normalize_path("src\\main.rs\\"), "src/main.rs");
}

#[test]
fn validate_repo_relative_path_rejects_escape_segments() {
    assert!(validate_repo_relative_path("src/lib.rs").is_ok());
    assert!(validate_repo_relative_path("  ").is_err());
    assert!(validate_repo_relative_path("../secret").is_err());
    assert!(validate_repo_relative_path("/absolute").is_err());
}

#[test]
fn validate_git_command_request_checks_required_fields() {
    let temp = tempfile::tempdir().expect("temp dir");
    let valid = GitCommandRequest {
        directory: temp.path().to_string_lossy().to_string(),
        args: vec!["status".to_string()],
        timeout_secs: Some(1),
    };
    assert!(validate_git_command_request(&valid).is_ok());

    assert!(
        validate_git_command_request(&GitCommandRequest {
            directory: String::new(),
            ..valid.clone()
        })
        .is_err()
    );
    assert!(
        validate_git_command_request(&GitCommandRequest {
            directory: "/definitely/missing/vw-git".to_string(),
            ..valid.clone()
        })
        .is_err()
    );
    assert!(
        validate_git_command_request(&GitCommandRequest { args: Vec::new(), ..valid.clone() })
            .is_err()
    );
    assert!(
        validate_git_command_request(&GitCommandRequest {
            args: vec!["status\0bad".to_string()],
            ..valid.clone()
        })
        .is_err()
    );
    assert!(
        validate_git_command_request(&GitCommandRequest { timeout_secs: Some(0), ..valid })
            .is_err()
    );
}

#[test]
fn collect_file_selections_merges_files_hunks_and_lines() {
    let request = GitCommitRequest {
        project_id: vw_api_types::id::ProjectId("project".to_string()),
        worktree_id: None,
        message: "commit".to_string(),
        stage_all: false,
        selected_files: vec!["src/lib.rs".to_string()],
        selected_hunks: vec![vw_api_types::git::GitHunkSelectionDto {
            path: "src/lib.rs".to_string(),
            index: 2,
        }],
        selected_lines: vec![vw_api_types::git::GitLineSelectionDto {
            path: "src/lib.rs".to_string(),
            line: 4,
        }],
        selected_old_lines: vec![vw_api_types::git::GitLineSelectionDto {
            path: "old.rs".to_string(),
            line: 1,
        }],
    };

    let selections = collect_file_selections(&request).expect("selections");

    let src = selections.get("src/lib.rs").expect("src selection");
    assert!(src.stage_file);
    assert!(src.hunks.contains(&2));
    assert!(src.new_lines.contains(&4));
    assert!(selections.get("old.rs").expect("old selection").old_lines.contains(&1));
}

#[test]
fn collect_file_selections_rejects_escaping_paths() {
    let request = GitCommitRequest {
        project_id: vw_api_types::id::ProjectId("project".to_string()),
        worktree_id: None,
        message: "commit".to_string(),
        stage_all: false,
        selected_files: vec!["../secret".to_string()],
        selected_hunks: Vec::new(),
        selected_lines: Vec::new(),
        selected_old_lines: Vec::new(),
    };

    assert!(collect_file_selections(&request).is_err());
}

#[test]
fn validate_merge_branch_pair_rejects_empty_head_and_same_branch() {
    assert!(validate_merge_branch_pair("feature", "main").is_ok());
    assert!(validate_merge_branch_pair("", "main").is_err());
    assert!(validate_merge_branch_pair("HEAD", "main").is_err());
    assert!(validate_merge_branch_pair("feature", "").is_err());
    assert!(validate_merge_branch_pair("feature", "HEAD").is_err());
    assert!(validate_merge_branch_pair("main", "main").is_err());
}

#[test]
fn final_line_ending_only_change_matches_missing_newline() {
    assert!(is_final_line_ending_only_change("123\n", "123"));
    assert!(is_final_line_ending_only_change("123", "123\n"));
    assert!(is_final_line_ending_only_change("123\r\n", "123"));
}

#[test]
fn final_line_ending_only_change_rejects_content_changes() {
    assert!(!is_final_line_ending_only_change("123\n", "124"));
    assert!(!is_final_line_ending_only_change("123\n", "123\n"));
    assert!(!is_final_line_ending_only_change("123\n456\n", "123\n457"));
}

#[test]
fn selected_file_content_replaces_selected_hunk() {
    let mut selection = GitFileSelection::default();
    selection.hunks.insert(0);

    let content =
        build_selected_file_content("alpha\nold\nomega\n", "alpha\nnew\nomega\n", &selection)
            .expect("hunk should build selected content");

    assert_eq!(content, "alpha\nnew\nomega\n");
}

#[test]
fn selected_file_content_keeps_unselected_delete_side() {
    let mut selection = GitFileSelection::default();
    selection.new_lines.insert(1);

    let content =
        build_selected_file_content("alpha\nold\nomega\n", "alpha\nnew\nomega\n", &selection)
            .expect("line selection should build selected content");

    assert_eq!(content, "alpha\nnew\nold\nomega\n");
}

#[test]
fn selected_file_content_keeps_unselected_insert_side() {
    let mut selection = GitFileSelection::default();
    selection.old_lines.insert(1);

    let content =
        build_selected_file_content("alpha\nold\nomega\n", "alpha\nnew\nomega\n", &selection)
            .expect("line selection should build selected content");

    assert_eq!(content, "alpha\nomega\n");
}

#[test]
fn selected_file_content_can_select_insert_only() {
    let mut selection = GitFileSelection::default();
    selection.new_lines.insert(1);

    let content = build_selected_file_content("alpha\nomega\n", "alpha\nnew\nomega\n", &selection)
        .expect("line selection should build selected content");

    assert_eq!(content, "alpha\nnew\nomega\n");
}

#[test]
fn selected_file_content_can_select_delete_only() {
    let mut selection = GitFileSelection::default();
    selection.old_lines.insert(1);

    let content = build_selected_file_content("alpha\nold\nomega\n", "alpha\nomega\n", &selection)
        .expect("line selection should build selected content");

    assert_eq!(content, "alpha\nomega\n");
}

#[test]
fn selected_file_content_rejects_bad_hunk_index() {
    let mut selection = GitFileSelection::default();
    selection.hunks.insert(99);

    let error = build_selected_file_content("old\n", "new\n", &selection)
        .expect_err("bad hunk should fail");

    assert_eq!(error, "bad hunk index");
}

#[test]
fn split_lines_preserves_endings_and_empty_input() {
    assert_eq!(split_lines_preserve_ending(""), Vec::<&str>::new());
    assert_eq!(split_lines_preserve_ending("a\nb"), vec!["a\n", "b"]);
}

#[test]
fn strip_final_line_ending_handles_lf_and_crlf() {
    assert_eq!(strip_final_line_ending("a\n"), Some("a"));
    assert_eq!(strip_final_line_ending("a\r\n"), Some("a"));
    assert_eq!(strip_final_line_ending("a"), None);
}

#[test]
fn normalize_git_output_bytes_normalizes_crlf_and_cr() {
    assert_eq!(normalize_git_output_bytes(b"a\r\nb\rc"), "a\nb\nc");
}

#[test]
fn git_file_mode_defaults_for_untracked_files() {
    let repo = init_repo();
    std::fs::write(repo.path().join("new.txt"), "new\n").expect("new file");

    assert_eq!(git_file_mode(repo.path_str(), "new.txt").expect("mode"), "100644");
}

#[test]
fn run_git_command_blocking_returns_stdout_and_status() {
    let repo = init_repo();

    let response = run_git_command_blocking(&GitCommandRequest {
        directory: repo.path_str().to_string(),
        args: vec!["status".to_string(), "--short".to_string()],
        timeout_secs: None,
    })
    .expect("git status");

    assert!(response.success);
    assert_eq!(response.code, Some(0));
    assert_eq!(response.stdout, "");
}

#[test]
fn commit_selected_blocking_stage_all_creates_commit() {
    let repo = init_repo();
    std::fs::write(repo.path().join("created.txt"), "created\n").expect("created file");
    let request = GitCommitRequest {
        project_id: vw_api_types::id::ProjectId("project".to_string()),
        worktree_id: None,
        message: "add created".to_string(),
        stage_all: true,
        selected_files: Vec::new(),
        selected_hunks: Vec::new(),
        selected_lines: Vec::new(),
        selected_old_lines: Vec::new(),
    };

    let response = commit_selected_blocking(repo.path_str(), &request).expect("commit");

    assert!(response.ok);
    assert_eq!(response.commit.message, "add created");
    assert_eq!(response.commit.sha.len(), 40);
}

#[test]
fn commit_selected_blocking_rejects_empty_selection() {
    let repo = init_repo();
    let request = GitCommitRequest {
        project_id: vw_api_types::id::ProjectId("project".to_string()),
        worktree_id: None,
        message: "nothing".to_string(),
        stage_all: false,
        selected_files: Vec::new(),
        selected_hunks: Vec::new(),
        selected_lines: Vec::new(),
        selected_old_lines: Vec::new(),
    };

    assert_eq!(
        commit_selected_blocking(repo.path_str(), &request).expect_err("empty selection"),
        "no changes selected"
    );
}

#[test]
fn git_stage_partial_file_stages_selected_new_line() {
    let repo = init_repo();
    std::fs::write(repo.path().join("tracked.txt"), "alpha\nold\nomega\n").expect("tracked");
    run_git(repo.path_str(), &["add", "tracked.txt"]).expect("add");
    run_git(repo.path_str(), &["commit", "-m", "initial"]).expect("initial commit");
    std::fs::write(repo.path().join("tracked.txt"), "alpha\nnew\nomega\n").expect("modified");
    let mut selection = GitFileSelection::default();
    selection.new_lines.insert(1);

    git_stage_partial_file(repo.path_str(), "tracked.txt", &selection).expect("partial stage");

    let cached =
        run_git(repo.path_str(), &["diff", "--cached", "--", "tracked.txt"]).expect("cached diff");
    assert!(cached.contains("+new"));
    assert!(!cached.contains("-old"));
}

struct TestRepo {
    temp: tempfile::TempDir,
}

impl TestRepo {
    fn path(&self) -> &std::path::Path {
        self.temp.path()
    }

    fn path_str(&self) -> &str {
        self.temp.path().to_str().expect("utf8 path")
    }
}

fn init_repo() -> TestRepo {
    let temp = tempfile::tempdir().expect("repo dir");
    let repo = TestRepo { temp };
    run_git(repo.path_str(), &["init"]).expect("git init");
    run_git(repo.path_str(), &["config", "user.email", "test@example.com"]).expect("email");
    run_git(repo.path_str(), &["config", "user.name", "Test User"]).expect("name");
    repo
}
