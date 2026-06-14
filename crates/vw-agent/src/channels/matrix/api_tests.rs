use super::*;
use std::collections::VecDeque;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn mock_server(
    responses: Vec<(&'static str, u16, &'static str)>,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut responses: VecDeque<_> = responses
        .into_iter()
        .map(|(path, status, body)| (path.to_string(), status, body.to_string()))
        .collect();

    let handle = tokio::spawn(async move {
        let mut seen = Vec::new();
        while let Some((expected_path, status, body)) = responses.pop_front() {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0; 4096];
            let n = stream.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("")
                .to_string();
            seen.push(path.clone());
            assert_eq!(path, expected_path);

            let status_text = match status {
                200 => "OK",
                404 => "Not Found",
                _ => "Error",
            };
            let response = format!(
                "HTTP/1.1 {status} {status_text}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        }
        seen
    });

    (format!("http://{addr}"), handle)
}

fn channel(homeserver: String, room_id: &str) -> MatrixChannel {
    MatrixChannel::new(
        homeserver,
        "token".to_string(),
        room_id.to_string(),
        vec!["@user:server".to_string()],
    )
}

#[tokio::test]
async fn target_room_id_returns_direct_room_ids_without_http() {
    let ch = channel("http://127.0.0.1:9".to_string(), "!room:server");

    assert_eq!(ch.target_room_id().await.unwrap(), "!room:server");
}

#[tokio::test]
async fn resolve_room_alias_fetches_directory_endpoint_and_caches_target_room() {
    let (homeserver, handle) = mock_server(vec![(
        "/_matrix/client/v3/directory/room/%23alias%3Aserver",
        200,
        r#"{"room_id":"!resolved:server"}"#,
    )])
    .await;
    let ch = channel(homeserver, "#alias:server");

    assert_eq!(ch.target_room_id().await.unwrap(), "!resolved:server");
    assert_eq!(ch.target_room_id().await.unwrap(), "!resolved:server");
    assert_eq!(handle.await.unwrap().len(), 1);
}

#[tokio::test]
async fn resolve_room_alias_reports_sanitized_error_body() {
    let (homeserver, _handle) = mock_server(vec![(
        "/_matrix/client/v3/directory/room/%23missing%3Aserver",
        500,
        r#"{"error":"token syt_secret_value failed"}"#,
    )])
    .await;
    let ch = channel(homeserver, "#missing:server");

    let err = ch.resolve_room_id().await.expect_err("alias failure should error");

    assert!(err.to_string().contains("Matrix room alias resolution failed"));
    assert!(!err.to_string().contains("syt_secret_value"));
}

#[tokio::test]
async fn resolve_room_id_rejects_invalid_room_reference() {
    let ch = channel("http://127.0.0.1:9".to_string(), "plain-room");

    let err = ch.resolve_room_id().await.expect_err("invalid room reference");

    assert!(err.to_string().contains("must start with"));
}

#[tokio::test]
async fn get_my_user_id_reads_whoami_response() {
    let (homeserver, _handle) = mock_server(vec![(
        "/_matrix/client/v3/account/whoami",
        200,
        r#"{"user_id":"@bot:server","device_id":"DEVICE"}"#,
    )])
    .await;
    let ch = channel(homeserver, "!room:server");

    assert_eq!(ch.get_my_user_id().await.unwrap(), "@bot:server");
}

#[tokio::test]
async fn ensure_room_supported_accepts_accessible_unencrypted_room() {
    let (homeserver, _handle) = mock_server(vec![
        ("/_matrix/client/v3/rooms/%21room%3Aserver/joined_members", 200, r#"{"joined":{}}"#),
        (
            "/_matrix/client/v3/rooms/%21room%3Aserver/state/m.room.encryption",
            404,
            r#"{"errcode":"M_NOT_FOUND"}"#,
        ),
    ])
    .await;
    let ch = channel(homeserver, "!room:server");

    ch.ensure_room_supported("!room:server").await.unwrap();
}

#[tokio::test]
async fn ensure_room_supported_errors_when_access_check_fails() {
    let (homeserver, _handle) = mock_server(vec![(
        "/_matrix/client/v3/rooms/%21room%3Aserver/joined_members",
        403,
        r#"{"error":"forbidden"}"#,
    )])
    .await;
    let ch = channel(homeserver, "!room:server");

    let err = ch.ensure_room_supported("!room:server").await.expect_err("access failure");

    assert!(err.to_string().contains("room access check failed"));
}
