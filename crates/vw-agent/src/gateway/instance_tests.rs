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
fn normalize_rel_path_rejects_absolute_path_outside_root() {
    let root = std::path::PathBuf::from("/tmp/root");

    assert_eq!(normalize_rel_path(&root, "/etc/passwd"), None);
}
