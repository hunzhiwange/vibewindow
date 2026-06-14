use super::*;
use crate::app::agent::gateway::instance::InstanceQuery;
use crate::app::agent::storage;
use axum::http::StatusCode;
use axum::http::{HeaderMap, HeaderValue};
use vw_api_types::session::GatewaySessionCreateBody;

#[test]
fn session_api_error_maps_storage_not_found_to_not_found() {
    let error = session_api_error(agent_session::session::Error::Storage(
        storage::Error::NotFound(storage::NotFoundError { message: "missing session".to_string() }),
    ));

    assert_eq!(error.status, StatusCode::NOT_FOUND);
    assert_eq!(error.to_string(), "missing session");
}

#[test]
fn session_api_error_maps_other_errors_to_bad_request() {
    let error = session_api_error(agent_session::session::Error::NoProjectContext);

    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert_eq!(error.to_string(), "no active project context");
}

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
fn has_explicit_directory_rejects_blank_query_and_header() {
    let query = InstanceQuery { directory: Some("  ".to_string()) };
    let mut headers = HeaderMap::new();
    headers.insert("x-vibewindow-directory", HeaderValue::from_static("  "));

    assert!(!has_explicit_directory(&query, &headers));
}

#[test]
fn has_explicit_directory_rejects_absent_query_and_header() {
    let query = InstanceQuery { directory: None };
    let headers = HeaderMap::new();

    assert!(!has_explicit_directory(&query, &headers));
}

#[test]
fn has_explicit_directory_ignores_invalid_header_value() {
    let query = InstanceQuery { directory: None };
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_bytes(b"\xff").expect("header bytes");
    headers.insert("x-vibewindow-directory", value);

    assert!(!has_explicit_directory(&query, &headers));
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

#[test]
fn ui_session_list_query_deserializes_all_filters() {
    let query: UiSessionListQuery = serde_json::from_value(serde_json::json!({
        "directory": "/tmp/project",
        "roots": false,
        "start": 42,
        "search": "topic",
        "limit": 10
    }))
    .expect("valid query");

    assert_eq!(query.directory.as_deref(), Some("/tmp/project"));
    assert_eq!(query.roots, Some(false));
    assert_eq!(query.start, Some(42));
    assert_eq!(query.search.as_deref(), Some("topic"));
    assert_eq!(query.limit, Some(10));
}

#[test]
fn ui_session_create_body_deserializes_flattened_session_and_permission() {
    let body: UiSessionCreateBody = serde_json::from_value(serde_json::json!({
        "parentID": "parent-1",
        "title": "Child",
        "permission": [
            {
                "permission": "shell.exec",
                "pattern": "cargo test",
                "action": "allow"
            }
        ]
    }))
    .expect("valid create body");

    assert_eq!(
        body.session,
        GatewaySessionCreateBody {
            parent_id: Some("parent-1".to_string()),
            title: Some("Child".to_string()),
        }
    );
    let permission = body.permission.expect("permission rules");
    assert_eq!(permission.len(), 1);
    assert_eq!(permission[0].permission, "shell.exec");
}

#[test]
fn ui_session_create_body_allows_permission_to_be_absent() {
    let body: UiSessionCreateBody = serde_json::from_value(serde_json::json!({
        "title": "Standalone"
    }))
    .expect("valid create body");

    assert_eq!(
        body.session,
        GatewaySessionCreateBody { parent_id: None, title: Some("Standalone".to_string()) }
    );
    assert!(body.permission.is_none());
}

#[test]
fn resolve_scope_from_query_returns_scope_for_directory_query() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let query = InstanceQuery { directory: Some(temp_dir.path().to_string_lossy().to_string()) };
    let headers = HeaderMap::new();

    assert!(resolve_scope_from_query(&query, &headers).is_some());
}

#[test]
fn resolve_scope_from_query_returns_scope_for_directory_header() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let query = InstanceQuery { directory: None };
    let mut headers = HeaderMap::new();
    let path = temp_dir.path().to_string_lossy();
    let value = HeaderValue::from_str(path.as_ref()).expect("header value");
    headers.insert("x-vibewindow-directory", value);

    assert!(resolve_scope_from_query(&query, &headers).is_some());
}
