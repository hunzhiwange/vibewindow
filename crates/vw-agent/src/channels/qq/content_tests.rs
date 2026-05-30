use super::*;

#[test]
fn parse_outgoing_content_extracts_only_remote_image_markers() {
    let (text, images) =
        parse_outgoing_content("hello\n[IMAGE:https://example.com/a.png]\n[IMAGE:/tmp/a.png]");

    assert_eq!(text, "hello\n[IMAGE:/tmp/a.png]");
    assert_eq!(images, vec!["https://example.com/a.png".to_string()]);
}

#[test]
fn compose_message_content_combines_text_and_image_attachments() {
    let payload = serde_json::json!({
        "content": "hello",
        "attachments": [{
            "url": "https://example.com/a.png",
            "content_type": "image/png"
        }]
    });

    assert_eq!(
        compose_message_content(&payload),
        Some("hello\n\n[IMAGE:https://example.com/a.png]".to_string())
    );
}

#[test]
fn build_text_message_body_trims_and_adds_passive_reply_fields() {
    let body = build_text_message_body(" hi ", Some("msg-1"), 7).expect("text body");

    assert_eq!(body["content"], "hi");
    assert_eq!(body["msg_type"], 0);
    assert_eq!(body["msg_id"], "msg-1");
    assert_eq!(body["msg_seq"], 7);
    assert!(build_text_message_body("   ", None, 0).is_none());
}
