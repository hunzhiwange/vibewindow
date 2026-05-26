use super::*;
use crate::app::agent::gateway::instance::InstanceQuery;
use axum::http::{HeaderMap, HeaderValue};

#[test]
fn has_explicit_directory_checks_query_before_headers() {
    let query = InstanceQuery { directory: Some("/tmp/project".to_string()) };
    let headers = HeaderMap::new();

    assert!(has_explicit_directory(&query, &headers));
}

#[test]
fn has_explicit_directory_accepts_directory_header() {
    let query = InstanceQuery { directory: None };
    let mut headers = HeaderMap::new();
    headers.insert("x-vibewindow-directory", HeaderValue::from_static("/tmp/project"));

    assert!(has_explicit_directory(&query, &headers));
}

#[test]
fn ui_session_list_query_deserializes_defaults() {
    let query: UiSessionListQuery = serde_json::from_value(serde_json::json!({
        "roots": true,
        "limit": 20
    }))
    .expect("valid query");

    assert_eq!(query.directory, None);
    assert_eq!(query.roots, Some(true));
    assert_eq!(query.limit, Some(20));
}
