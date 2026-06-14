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
fn constructor_trims_base_url_and_keeps_runtime_flags() {
    let channel = MattermostChannel::new(
        "https://mattermost.example.com///".to_string(),
        "token".to_string(),
        Some("chan-1".to_string()),
        vec!["alice".to_string()],
        false,
        true,
    );

    assert_eq!(channel.base_url, "https://mattermost.example.com");
    assert_eq!(channel.channel_id.as_deref(), Some("chan-1"));
    assert_eq!(channel.allowed_users, vec!["alice"]);
    assert!(!channel.thread_replies);
    assert!(channel.mention_only);
}

#[test]
fn group_reply_sender_override_rejects_empty_and_honors_wildcard() {
    let specific = MattermostChannel::new(
        "https://mattermost.example.com".to_string(),
        "token".to_string(),
        None,
        vec!["*".to_string()],
        true,
        true,
    )
    .with_group_reply_allowed_senders(vec![
        " user-2 ".to_string(),
        String::new(),
        "user-1".to_string(),
        "user-1".to_string(),
    ]);

    assert_eq!(specific.group_reply_allowed_sender_ids, vec!["user-1", "user-2"]);
    assert!(specific.is_group_sender_trigger_enabled(" user-1 "));
    assert!(!specific.is_group_sender_trigger_enabled(" "));

    let wildcard = specific.with_group_reply_allowed_senders(vec!["*".to_string()]);
    assert!(wildcard.is_group_sender_trigger_enabled("anyone"));
}

#[tokio::test]
async fn send_posts_root_level_message() {
    let base_url = one_shot_http_server("HTTP/1.1 201 Created", "{}").await;
    let channel = MattermostChannel::new(
        base_url,
        "token".to_string(),
        None,
        vec!["*".to_string()],
        true,
        false,
    );

    channel.send(&SendMessage::new("hello", "channel-1")).await.unwrap();
}

#[tokio::test]
async fn send_reports_sanitized_error_status() {
    let base_url = one_shot_http_server(
        "HTTP/1.1 500 Internal Server Error",
        r#"{"message":"token sk-secret should be hidden"}"#,
    )
    .await;
    let channel = MattermostChannel::new(
        base_url,
        "token".to_string(),
        None,
        vec!["*".to_string()],
        true,
        false,
    );

    let error =
        channel.send(&SendMessage::new("hello", "channel-1:root-1")).await.unwrap_err().to_string();

    assert!(error.contains("Mattermost post failed"));
    assert!(error.contains("500 Internal Server Error"));
}
