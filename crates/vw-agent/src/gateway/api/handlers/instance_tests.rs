use super::*;
use axum::extract::Query;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

fn query_for(path: &std::path::Path) -> InstanceQuery {
    InstanceQuery { directory: Some(path.to_string_lossy().to_string()) }
}

fn headers_for(path: &std::path::Path) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-vibewindow-directory",
        HeaderValue::from_str(&path.to_string_lossy()).expect("valid directory header"),
    );
    headers
}

#[tokio::test]
async fn path_get_reports_global_and_instance_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let Json(info) = path_get(Query(query_for(temp.path())), HeaderMap::new())
        .await
        .expect("path_get should succeed");

    assert!(!info.home.is_empty());
    assert!(!info.state.is_empty());
    assert!(!info.config.is_empty());
    assert_eq!(info.directory, temp.path().to_string_lossy());
    assert!(!info.worktree.trim().is_empty());
}

#[tokio::test]
async fn path_get_uses_directory_header_when_query_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let Json(info) = path_get(Query(InstanceQuery { directory: None }), headers_for(temp.path()))
        .await
        .expect("path_get should succeed");

    assert_eq!(info.directory, temp.path().to_string_lossy());
}

#[tokio::test]
async fn vcs_get_returns_empty_branch_for_non_git_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    let Json(info) = vcs_get(Query(query_for(temp.path())), HeaderMap::new())
        .await
        .expect("vcs_get should succeed");

    assert_eq!(info.branch, "");
}

#[tokio::test]
async fn vcs_get_reports_current_git_branch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let init = std::process::Command::new("git")
        .args(["init", "-b", "unit-test-branch"])
        .current_dir(temp.path())
        .output()
        .expect("git init should run");
    assert!(init.status.success());
    std::fs::write(temp.path().join("README.md"), "test").expect("readme should be written");
    let add = std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp.path())
        .output()
        .expect("git add should run");
    assert!(add.status.success());
    let commit = std::process::Command::new("git")
        .args([
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "user.name=Test User",
            "commit",
            "-m",
            "initial",
        ])
        .current_dir(temp.path())
        .output()
        .expect("git commit should run");
    assert!(commit.status.success());

    let Json(info) = vcs_get(Query(query_for(temp.path())), HeaderMap::new())
        .await
        .expect("vcs_get should succeed");

    assert_eq!(info.branch, "unit-test-branch");
}

#[tokio::test]
async fn instance_event_sse_builds_event_stream_response() {
    let temp = tempfile::tempdir().expect("tempdir");
    let response = instance_event_sse(Query(query_for(temp.path())), HeaderMap::new())
        .await
        .expect("sse should be created")
        .into_response();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert_eq!(
        response.headers().get(axum::http::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
        Some("text/event-stream")
    );
}

#[tokio::test]
async fn instance_dispose_returns_true_and_clears_cached_instance() {
    let temp = tempfile::tempdir().expect("tempdir");

    let Json(disposed) = instance_dispose(Query(query_for(temp.path())), HeaderMap::new())
        .await
        .expect("dispose should succeed");

    assert!(disposed);
}
