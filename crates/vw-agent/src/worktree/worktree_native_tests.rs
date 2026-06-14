#![cfg(not(target_arch = "wasm32"))]

use super::Info;
use super::native::{
    CmdResult, canonical, find_entry, parse_worktree_list, path_exists, resolve_reset_target,
    run_git,
};
use std::path::Path;
use std::process::Command;

#[test]
fn parse_worktree_list_preserves_paths_and_branches() {
    let entries =
        parse_worktree_list("worktree /repo\nbranch refs/heads/main\n\nworktree /repo-two\nbare\n");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, "/repo");
    assert_eq!(entries[0].branch.as_deref(), Some("refs/heads/main"));
    assert_eq!(entries[1].branch, None);
}

#[test]
fn cmd_result_prefers_stderr_then_stdout_for_error_text() {
    let result = CmdResult { success: false, stdout: "out\n".into(), stderr: "err\n".into() };
    assert_eq!(result.error_text("fallback"), "err\nout");

    let empty = CmdResult { success: false, stdout: String::new(), stderr: String::new() };
    assert_eq!(empty.error_text("fallback"), "fallback");
}

#[tokio::test]
async fn path_exists_reports_temp_directory() {
    let dir = tempfile::tempdir().expect("temp dir");

    assert!(path_exists(dir.path()).await);
    assert!(!path_exists(dir.path().join("missing").as_path()).await);
}

#[tokio::test]
async fn canonical_absolutizes_missing_relative_paths() {
    let path = canonical("definitely-missing-worktree-test-path").await;

    assert!(path.is_absolute());
    assert!(path.ends_with("definitely-missing-worktree-test-path"));
}

#[tokio::test]
async fn find_entry_matches_canonical_target_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let entries = vec![super::native::WorktreeEntry {
        path: dir.path().to_string_lossy().to_string(),
        branch: Some("refs/heads/demo".to_string()),
    }];

    let found = find_entry(&entries, dir.path()).await.expect("entry");

    assert_eq!(found.branch.as_deref(), Some("refs/heads/demo"));
}

#[tokio::test]
async fn run_git_and_resolve_reset_target_work_in_temp_repo() {
    let repo = tempfile::tempdir().expect("temp repo");
    init_repo(repo.path());

    let status =
        run_git(&["status", "--porcelain"], &repo.path().to_string_lossy()).await.expect("git");
    assert!(status.success);

    let target = resolve_reset_target(repo.path(), Some("main")).await.expect("main target");
    assert_eq!(target.target, "main");
    assert_eq!(target.remote, None);
    assert_eq!(target.remote_branch, None);

    let invalid = match resolve_reset_target(repo.path(), Some("missing-branch")).await {
        Ok(_) => panic!("missing branch should be rejected"),
        Err(error) => error,
    };
    assert!(
        invalid.to_string().contains("missing-branch")
            || invalid.to_string().contains("Needed a single revision")
    );
}

#[test]
fn info_is_cloneable_for_native_callers() {
    let info = Info { name: "n".into(), branch: "b".into(), directory: "d".into() };

    assert_eq!(info.clone().directory, "d");
}

fn init_repo(path: &Path) {
    run(Command::new("git").arg("init").arg("-b").arg("main").arg(path));
    run(Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("config")
        .arg("user.email")
        .arg("test@example.com"));
    run(Command::new("git").arg("-C").arg(path).arg("config").arg("user.name").arg("Test User"));
    std::fs::write(path.join("README.md"), "demo").expect("write readme");
    run(Command::new("git").arg("-C").arg(path).arg("add").arg("README.md"));
    run(Command::new("git").arg("-C").arg(path).arg("commit").arg("-m").arg("init"));
}

fn run(command: &mut Command) {
    let output = command.output().expect("run command");
    assert!(
        output.status.success(),
        "command failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
