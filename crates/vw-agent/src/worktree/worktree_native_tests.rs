#![cfg(not(target_arch = "wasm32"))]

use super::native::{CmdResult, parse_worktree_list, path_exists};

#[test]
fn parse_worktree_list_preserves_paths_and_branches() {
    let entries = parse_worktree_list(
        "worktree /repo\nbranch refs/heads/main\n\nworktree /repo-two\nbare\n",
    );

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
}
