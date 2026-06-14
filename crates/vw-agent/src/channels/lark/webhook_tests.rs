use super::*;

#[tokio::test]
async fn listen_http_requires_port_before_binding_server() {
    let channel = LarkChannel::new(
        "app-id".to_string(),
        "secret".to_string(),
        "verify".to_string(),
        None,
        vec!["*".to_string()],
        false,
    );
    let (tx, _rx) = tokio::sync::mpsc::channel(1);

    let err = channel.listen_http(tx).await.expect_err("missing port should fail");

    assert!(err.to_string().contains("requires `port`"));
}
