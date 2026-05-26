use super::*;

#[test]
fn extract_lark_token_ttl_seconds_accepts_expire_variants_and_defaults() {
    assert_eq!(extract_lark_token_ttl_seconds(&serde_json::json!({"expire": 30})), 30);
    assert_eq!(extract_lark_token_ttl_seconds(&serde_json::json!({"expires_in": 45})), 45);
    assert_eq!(extract_lark_token_ttl_seconds(&serde_json::json!({"expire": 0})), 1);
    assert_eq!(
        extract_lark_token_ttl_seconds(&serde_json::json!({})),
        LARK_DEFAULT_TOKEN_TTL.as_secs()
    );
}

#[test]
fn ensure_lark_send_success_rejects_status_and_business_errors() {
    assert!(ensure_lark_send_success(reqwest::StatusCode::OK, &serde_json::json!({"code": 0}), "ctx").is_ok());
    assert!(ensure_lark_send_success(reqwest::StatusCode::BAD_REQUEST, &serde_json::json!({"msg": "bad"}), "ctx").is_err());
    assert!(ensure_lark_send_success(reqwest::StatusCode::OK, &serde_json::json!({"code": 1}), "ctx").is_err());
}
