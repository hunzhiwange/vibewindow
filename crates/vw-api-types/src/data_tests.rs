use super::data::{AiDataConnectionDto, AiDataConnectionKind, AiDataCountMode, AiDataQueryRequest, AiDataReportSourceDto, AiDataSettings};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn settings_and_connection_defaults_are_applied() {
    let settings: AiDataSettings = serde_json::from_value(json!({})).expect("settings");
    assert_eq!(settings.schema_version, 1);
    assert_eq!(settings.default_limit, 100);
    assert_eq!(settings.default_timeout_secs, 30);

    let connection: AiDataConnectionDto = serde_json::from_value(json!({
        "id": "c1",
        "name": "local",
        "kind": "sqlite"
    }))
    .expect("connection");
    assert_eq!(connection.kind, AiDataConnectionKind::Sqlite);
    assert!(connection.enabled);
    assert!(!connection.read_only);
}

#[test]
fn query_request_accepts_string_encoded_compat_fields() {
    let query: AiDataQueryRequest = serde_json::from_value(json!({
        "count": "count_only",
        "searchFields": " name, email ",
        "searchCondition": "{\"name\":\"Ada\"}"
    }))
    .expect("query");

    assert_eq!(query.count, Some(AiDataCountMode::Only));
    assert_eq!(query.search_fields, Some(vec!["name".into(), "email".into()]));
    assert_eq!(query.search_condition, Some(json!({"name": "Ada"})));
}

#[test]
fn report_source_defaults_http_fields() {
    let source: AiDataReportSourceDto = serde_json::from_value(json!({
        "source_key": "main",
        "connection_id": "c1",
        "query_kind": "http"
    }))
    .expect("source");

    assert_eq!(source.http_method, "GET");
    assert!(source.append_pagination);
    assert_eq!(source.cube_query, None);
    assert_eq!(BTreeMap::<String, String>::new().len(), 0);
}
