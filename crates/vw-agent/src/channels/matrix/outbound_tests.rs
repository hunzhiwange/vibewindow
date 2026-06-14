use super::*;

#[tokio::test]
async fn send_impl_fails_fast_when_otk_conflict_is_already_detected() {
    let channel = MatrixChannel::new(
        "https://matrix.example.com".to_string(),
        "token".to_string(),
        "!room:matrix.example.com".to_string(),
        vec!["*".to_string()],
    );
    channel.otk_conflict_detected.store(true, Ordering::SeqCst);
    let message = SendMessage::new("hello", "!room:matrix.example.com");

    let error = channel.send_impl(&message).await.unwrap_err();

    assert!(error.to_string().contains("one-time key upload conflict"));
}

#[test]
fn outbound_markdown_payload_preserves_plain_body_and_html_format() {
    let content = RoomMessageEventContent::text_markdown("**hello**");
    let value = serde_json::to_value(content).unwrap();

    assert_eq!(value["msgtype"], "m.text");
    assert_eq!(value["body"], "**hello**");
    assert_eq!(value["format"], "org.matrix.custom.html");
    assert!(
        value["formatted_body"].as_str().unwrap_or_default().contains("<strong>hello</strong>")
    );
}
