use serde_json::json;
use vw_api_types::file::{
    CopyFileRequest, DeleteFileRequest, LargeFileDeleteRequest, LargeFileScanCancelRequest,
    LargeFileScanRequest, LargeFileScanStartRequest, ListFilesRequest, MoveFileRequest,
    ReadFileRequest, SearchFilesRequest, StatFileRequest, WriteFileRequest,
};

use crate::client::test_support;

#[test]
fn normalize_entry_path_trims_after_backslash_conversion() {
    assert_eq!(super::normalize_entry_path(r"\src\main.rs\"), "src/main.rs");
    assert_eq!(super::normalize_requested_path(Some("  ")), ".");
}

#[tokio::test]
async fn file_api_routes_structured_legacy_and_large_file_calls() {
    let server = test_support::server(vec![
        (
            200,
            json!({
                "project": {
                    "id": "project",
                    "name": "Repo",
                    "directory": "/repo",
                    "display_path": "/repo",
                    "status": "ready",
                    "created_at_ms": 1,
                    "updated_at_ms": 2,
                    "git": {"is_repo": true, "has_uncommitted_changes": false}
                }
            }),
        ),
        (200, json!([{"path": "\\src\\", "name": "src", "type": "directory"}])),
        (200, json!([{"path": "\\src\\main.rs", "name": "main.rs", "type": "file"}])),
        (
            200,
            json!({"path": "src/main.rs", "content": "fn main() {}", "encoding": "utf-8", "offset_line": 0, "line_count": 1, "truncated": false}),
        ),
        (200, json!({"ok": true, "path": "src/main.rs", "bytes_written": 12})),
        (200, json!({"ok": true})),
        (200, json!({"ok": true})),
        (200, json!({"ok": true})),
        (
            200,
            json!({"matches": [{"path": "src/main.rs", "line": 1, "column": 4, "text": "main"}]}),
        ),
        (
            200,
            json!({"entry": {"path": "src/main.rs", "name": "main.rs", "kind": "file", "size_bytes": 12}}),
        ),
        (200, json!({"root": "/repo", "total_bytes": 0, "total_files": 0, "categories": []})),
        (200, json!({"job_id": "job"})),
        (
            200,
            json!({"job_id": "job", "progress": {"phase_label": "done", "current_path": "", "total_files": 1, "processed_files": 1, "matched_files": 0, "progress_value": 1.0}, "finished": true}),
        ),
        (200, json!({"ok": true})),
        (200, json!({"deleted_paths": ["/repo/big.bin"], "failed_paths": []})),
        (200, json!({"root_directory": "/repo", "path": "README.md", "content": "hi"})),
        (
            200,
            json!({"ok": true, "root_directory": "/repo", "path": "README.md", "bytes_written": 2}),
        ),
    ]);

    let root = server
        .client()
        .file_list(&ListFilesRequest {
            project_id: "project".into(),
            worktree_id: None,
            path: None,
            depth: Some(2),
        })
        .await
        .expect("list")
        .root;
    assert_eq!(root.children.unwrap()[0].path, "src");
    assert_eq!(
        server
            .client()
            .file_read(&ReadFileRequest {
                project_id: "project".into(),
                worktree_id: None,
                path: "src/main.rs".to_string(),
                offset_line: Some(0),
                limit_lines: Some(10),
            })
            .await
            .expect("read")
            .line_count,
        1
    );
    assert!(
        server
            .client()
            .file_write(&WriteFileRequest {
                project_id: "project".into(),
                worktree_id: None,
                path: "src/main.rs".to_string(),
                content: "fn main() {}".to_string(),
                create_if_missing: true,
            })
            .await
            .expect("write")
            .ok
    );
    server
        .client()
        .file_move(&MoveFileRequest {
            project_id: "project".into(),
            worktree_id: None,
            from_path: "a".to_string(),
            to_path: "b".to_string(),
            overwrite: true,
        })
        .await
        .expect("move");
    server
        .client()
        .file_copy(&CopyFileRequest {
            project_id: "project".into(),
            worktree_id: None,
            from_path: "a".to_string(),
            to_path: "b".to_string(),
            overwrite: true,
        })
        .await
        .expect("copy");
    server
        .client()
        .file_delete(&DeleteFileRequest {
            project_id: "project".into(),
            worktree_id: None,
            path: "b".to_string(),
            recursive: true,
        })
        .await
        .expect("delete");
    assert_eq!(
        server
            .client()
            .file_search(&SearchFilesRequest {
                project_id: "project".into(),
                worktree_id: None,
                pattern: "main".to_string(),
                include: None,
                regex: false,
                case_sensitive: false,
            })
            .await
            .expect("search")
            .matches[0]
            .line,
        1
    );
    assert_eq!(
        server
            .client()
            .file_stat(&StatFileRequest {
                project_id: "project".into(),
                worktree_id: None,
                path: "src/main.rs".to_string(),
            })
            .await
            .expect("stat")
            .entry
            .name,
        "main.rs"
    );
    assert_eq!(
        server
            .client()
            .large_file_scan(&LargeFileScanRequest { root: "/repo".to_string() })
            .await
            .expect("scan")
            .total_files,
        0
    );
    assert_eq!(
        server
            .client()
            .large_file_scan_start(&LargeFileScanStartRequest { root: "/repo".to_string() })
            .await
            .expect("start")
            .job_id,
        "job"
    );
    assert!(server.client().large_file_scan_status("job").await.expect("status").finished);
    server
        .client()
        .large_file_scan_cancel(&LargeFileScanCancelRequest { job_id: "job".to_string() })
        .await
        .expect("cancel");
    assert_eq!(
        server
            .client()
            .large_file_delete(&LargeFileDeleteRequest {
                root: "/repo".to_string(),
                paths: vec!["/repo/big.bin".to_string()],
            })
            .await
            .expect("delete large")
            .deleted_paths,
        vec!["/repo/big.bin".to_string()]
    );
    assert_eq!(
        server
            .client()
            .file_read_in_directory(Some("/repo"), Some("agent"), "README.md")
            .await
            .expect("legacy read")
            .content,
        "hi"
    );
    assert!(
        server
            .client()
            .file_write_in_directory(Some("/repo"), None, "README.md", "hi", true)
            .await
            .expect("legacy write")
            .ok
    );

    assert_eq!(server.take_request().path, "/v1/projects/project");
    assert!(server.take_request().path.contains("/v1/file?directory=%2Frepo&path="));
    assert!(server.take_request().path.contains("/v1/file?directory=%2Frepo&path=src"));
    assert_eq!(server.take_request().path, "/v1/files/read");
    assert_eq!(server.take_request().path, "/v1/files/write");
    assert_eq!(server.take_request().path, "/v1/files/move");
    assert_eq!(server.take_request().path, "/v1/files/copy");
    assert_eq!(server.take_request().path, "/v1/files/delete");
    assert_eq!(server.take_request().path, "/v1/files/search");
    assert_eq!(server.take_request().path, "/v1/files/stat");
    assert_eq!(server.take_request().path, "/v1/files/large/scan");
    assert_eq!(server.take_request().path, "/v1/files/large/scan/start");
    assert!(server.take_request().path.contains("/v1/files/large/scan/status?job_id=job"));
    assert_eq!(server.take_request().path, "/v1/files/large/scan/cancel");
    assert_eq!(server.take_request().path, "/v1/files/large/delete");
    assert_eq!(server.take_request().path, "/v1/file/read");
    assert_eq!(server.take_request().path, "/v1/file/write");
    server.join();
}

#[tokio::test]
async fn file_list_supports_worktree_directory_and_depth_zero() {
    let server = test_support::server(vec![
        (
            200,
            json!({
                "worktree": {
                    "id": "worktree",
                    "project_id": "project",
                    "name": "Feature",
                    "branch": "feature",
                    "directory": "/repo-feature",
                    "status": "ready",
                    "created_at_ms": 1,
                    "updated_at_ms": 2
                }
            }),
        ),
        (200, json!([{"path": "\\src\\", "name": "src", "type": "directory"}])),
    ]);

    let root = server
        .client()
        .file_list(&ListFilesRequest {
            project_id: "project".into(),
            worktree_id: Some("worktree".into()),
            path: Some("src".to_string()),
            depth: Some(0),
        })
        .await
        .expect("list")
        .root;
    assert_eq!(root.path, "src");
    assert!(root.children.is_none());

    assert_eq!(server.take_request().path, "/v1/worktrees/worktree");
    assert!(server.take_request().path.contains("/v1/file?directory=%2Frepo-feature&path=src"));
    server.join();
}
