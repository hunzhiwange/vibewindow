use super::*;
use crate::app::agent::channels::traits::{Channel, SendMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn one_shot_http_server(status_line: &'static str, body: &'static str) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buffer = vec![0; 4096];
        let _ = stream.read(&mut buffer).await.unwrap();
        let response = format!(
            "{status_line}\r\ncontent-length: {}\r\ncontent-type: application/json\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    format!("http://{addr}")
}

#[test]
fn constructor_trims_base_url_and_actor_ids_are_canonicalized() {
    let channel = NextcloudTalkChannel::new(
        "https://cloud.example.com///".to_string(),
        "app-token".to_string(),
        vec!["users/Alice".to_string()],
    );

    assert_eq!(channel.base_url, "https://cloud.example.com");
    assert_eq!(NextcloudTalkChannel::canonical_actor_id(" users/Alice "), "Alice");
    assert!(channel.is_user_allowed("alice"));
    assert!(channel.is_user_allowed("https://cloud.example.com/users/Alice"));
    assert!(!channel.is_user_allowed(""));
}

#[test]
fn parse_webhook_payload_handles_activity_content_object_and_timestamp_string() {
    let channel = NextcloudTalkChannel::new(
        "https://cloud.example.com".to_string(),
        "app-token".to_string(),
        vec!["alice".to_string()],
    );
    let payload = serde_json::json!({
        "type": "create",
        "timestamp": "1735701200123",
        "actor": {
            "type": "Person",
            "id": "https://cloud.example.com/users/alice"
        },
        "object": {
            "type": "note",
            "id": 99,
            "token": "room-token",
            "content": {
                "message": "  hello from object  "
            }
        }
    });

    let messages = channel.parse_webhook_payload(&payload);

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, "99");
    assert_eq!(messages[0].sender, "alice");
    assert_eq!(messages[0].reply_target, "room-token");
    assert_eq!(messages[0].content, "hello from object");
    assert_eq!(messages[0].timestamp, 1_735_701_200);
}

#[test]
fn signature_verification_rejects_empty_random_and_bad_hex() {
    assert!(!verify_nextcloud_talk_signature("secret", "", "{}", "deadbeef"));
    assert!(!verify_nextcloud_talk_signature("secret", "nonce", "{}", "not-hex"));
}

#[tokio::test]
async fn send_posts_to_encoded_room_token() {
    let base_url = one_shot_http_server("HTTP/1.1 200 OK", "{}").await;
    let channel =
        NextcloudTalkChannel::new(base_url, "app-token".to_string(), vec!["*".to_string()]);

    channel.send(&SendMessage::new("hello", "room token/with slash")).await.unwrap();
}

#[tokio::test]
async fn send_reports_non_success_status() {
    let base_url =
        one_shot_http_server("HTTP/1.1 403 Forbidden", r#"{"ocs":{"meta":{"status":"failure"}}}"#)
            .await;
    let channel =
        NextcloudTalkChannel::new(base_url, "app-token".to_string(), vec!["*".to_string()]);

    let error =
        channel.send(&SendMessage::new("hello", "room-token")).await.unwrap_err().to_string();

    assert!(error.contains("Nextcloud Talk API error"));
    assert!(error.contains("403 Forbidden"));
}
