use super::*;

#[test]
fn whatsapp_signature_rejects_wrong_signature() {
    assert!(!verify_whatsapp_signature("secret", b"body", "sha256=not-valid"));
}

#[test]
fn whatsapp_verify_query_deserializes_fields() {
    let query: WhatsAppVerifyQuery = serde_json::from_value(serde_json::json!({
        "hub.mode": "subscribe",
        "hub.verify_token": "token",
        "hub.challenge": "challenge"
    }))
    .expect("valid query");

    assert_eq!(query.mode.as_deref(), Some("subscribe"));
    assert_eq!(query.challenge.as_deref(), Some("challenge"));
}
