use super::*;
use axum::http::{HeaderMap, HeaderValue};

#[test]
fn resolve_directory_prefers_query_over_header() {
    let query = InstanceQuery { directory: Some("/query".to_string()) };
    let mut headers = HeaderMap::new();
    headers.insert("x-vibewindow-directory", HeaderValue::from_static("/header"));

    assert_eq!(resolve_directory(&query, &headers), "/query");
}

#[test]
fn resolve_directory_uses_header_when_query_is_blank() {
    let query = InstanceQuery { directory: Some("   ".to_string()) };
    let mut headers = HeaderMap::new();
    headers.insert("x-vibewindow-directory", HeaderValue::from_static("/header"));

    assert_eq!(resolve_directory(&query, &headers), "/header");
}

#[test]
fn resolve_directory_ignores_blank_header() {
    let query = InstanceQuery { directory: None };
    let mut headers = HeaderMap::new();
    headers.insert("x-vibewindow-directory", HeaderValue::from_static("   "));

    assert_eq!(
        resolve_directory(&query, &headers),
        std::env::current_dir().expect("current dir").to_string_lossy().to_string()
    );
}

#[test]
fn resolve_directory_ignores_invalid_header_value() {
    let query = InstanceQuery { directory: None };
    let mut headers = HeaderMap::new();
    let invalid = HeaderValue::from_bytes(&[0xff]).expect("invalid utf8 header bytes");
    headers.insert("x-vibewindow-directory", invalid);

    assert_eq!(
        resolve_directory(&query, &headers),
        std::env::current_dir().expect("current dir").to_string_lossy().to_string()
    );
}

#[tokio::test]
async fn with_instance_runs_callback_inside_requested_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    let directory = temp.path().to_string_lossy().to_string();
    let observed = with_instance(directory.clone(), || {
        Box::pin(async move { Ok(crate::app::agent::project::instance::directory()) })
    })
    .await
    .expect("with_instance should run callback");

    assert_eq!(observed, directory);
}

#[tokio::test]
async fn with_instance_returns_callback_error() {
    let temp = tempfile::tempdir().expect("tempdir");
    let err = with_instance(temp.path().to_string_lossy().to_string(), || {
        Box::pin(async move { Err::<(), _>(ApiError::bad_request("callback failed")) })
    })
    .await
    .expect_err("callback error should be returned");

    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn with_instance_maps_instance_binding_error_to_bad_request() {
    let temp = tempfile::tempdir().expect("tempdir");
    let git_dir = temp.path().join(".git");
    std::fs::create_dir(&git_dir).expect("git dir");
    let project_id = format!("bad-instance-json-{}", std::process::id());
    std::fs::write(git_dir.join("vibewindow"), &project_id).expect("cached project id");

    let storage_path = crate::app::agent::global::paths()
        .data
        .join("storage")
        .join("project")
        .join(format!("{project_id}.json"));
    std::fs::create_dir_all(storage_path.parent().expect("project storage parent"))
        .expect("project storage dir");
    std::fs::write(&storage_path, "{").expect("invalid project json");

    let err = with_instance(temp.path().to_string_lossy().to_string(), || {
        Box::pin(async move { Ok::<_, ApiError>(()) })
    })
    .await
    .expect_err("instance binding error should be mapped");

    let _ = std::fs::remove_file(storage_path);
    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn instance_init_noops_without_project_context() {
    (instance_init())().await;
}

#[test]
fn normalize_rel_path_handles_empty_root_and_dot() {
    let root = std::path::PathBuf::from("/tmp/root");

    assert_eq!(normalize_rel_path(&root, ""), Some(String::new()));
    assert_eq!(normalize_rel_path(&root, "/"), Some(String::new()));
    assert_eq!(normalize_rel_path(&root, "."), Some(String::new()));
}

#[test]
fn normalize_rel_path_strips_root_from_absolute_path() {
    let root = std::path::PathBuf::from("/tmp/root");

    assert_eq!(
        normalize_rel_path(&root, "/tmp/root/nested/file.txt"),
        Some("nested/file.txt".to_string())
    );
}

#[test]
fn normalize_rel_path_keeps_relative_path_without_leading_slashes() {
    let root = std::path::PathBuf::from("/tmp/root");

    assert_eq!(normalize_rel_path(&root, "nested/file.txt"), Some("nested/file.txt".to_string()));
}

#[test]
fn normalize_rel_path_rejects_absolute_path_outside_root() {
    let root = std::path::PathBuf::from("/tmp/root");

    assert_eq!(normalize_rel_path(&root, "/etc/passwd"), None);
}
