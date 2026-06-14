use serde_json::json;
use vw_api_types::git::{GitCommandRequest, GitCommitRequest, GitMergeRequest};

use crate::client::test_support;

#[tokio::test]
async fn git_api_routes_commit_command_and_merge() {
    let server = test_support::server(vec![
        (200, json!({"ok": true, "commit": {"sha": "abc123", "message": "ship it"}})),
        (200, json!({"success": true, "code": 0, "stdout": "main", "stderr": ""})),
        (
            200,
            json!({
                "ok": true,
                "source_branch": "feature",
                "target_branch": "main",
                "workspace": "/repo",
                "already_merged": false,
                "message": "merged"
            }),
        ),
    ]);

    let commit = server
        .client()
        .git_commit(&GitCommitRequest {
            project_id: "project".into(),
            worktree_id: None,
            message: "ship it".to_string(),
            stage_all: true,
            selected_files: Vec::new(),
            selected_hunks: Vec::new(),
            selected_lines: Vec::new(),
            selected_old_lines: Vec::new(),
        })
        .await
        .expect("commit");
    assert_eq!(commit.commit.sha, "abc123");
    let command = server
        .client()
        .git_command(&GitCommandRequest {
            directory: "/repo".to_string(),
            args: vec!["branch".to_string()],
            timeout_secs: Some(5),
        })
        .await
        .expect("command");
    assert!(command.success);
    let merge = server
        .client()
        .git_merge(&GitMergeRequest {
            project_id: "project".into(),
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
        })
        .await
        .expect("merge");
    assert!(merge.ok);

    let request = server.take_request();
    assert_eq!(request.path, "/v1/git/commit");
    assert_eq!(request.body["stage_all"], true);
    let request = server.take_request();
    assert_eq!(request.path, "/v1/git/command");
    assert_eq!(request.body["args"], json!(["branch"]));
    let request = server.take_request();
    assert_eq!(request.path, "/v1/git/merge");
    assert_eq!(request.body["source_branch"], "feature");
    assert_eq!(request.body["target_branch"], "main");
    server.join();
}
