use super::*;
use vw_api_types::tool::GatewayRedisConnectionUpsertBody;

fn body(value: serde_json::Value) -> GatewayRedisConnectionUpsertBody {
    serde_json::from_value(value).expect("body")
}

#[test]
fn normalize_upsert_body_trims_names_and_defaults_pattern() {
    let normalized = normalize_upsert_body(&body(serde_json::json!({
        "name": "  local  ",
        "host": "  127.0.0.1  ",
        "port": 6379,
        "db": 0,
        "key_pattern": "  "
    })))
    .expect("valid config");

    assert_eq!(normalized.name, "local");
    assert_eq!(normalized.host, "127.0.0.1");
    assert_eq!(normalized.key_pattern, "*");
}

#[test]
fn normalize_upsert_body_rejects_tls_material_without_tls() {
    let error = normalize_upsert_body(&body(serde_json::json!({
        "name": "local",
        "host": "127.0.0.1",
        "port": 6379,
        "db": 0,
        "use_tls": false,
        "tls_cert": {"ca_path": "/tmp/ca.pem"}
    })))
    .expect_err("tls material requires tls");

    assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
}
