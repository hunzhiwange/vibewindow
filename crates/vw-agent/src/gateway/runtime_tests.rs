use super::*;

use std::time::Duration;

#[test]
fn runtime_entrypoints_are_available() {
    let _ = serve;
    let _ = start;
}

#[tokio::test]
async fn bind_prefer_uses_requested_available_port() {
    let reserved =
        tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.expect("reserve free port");
    let port = reserved.local_addr().expect("read reserved address").port();
    drop(reserved);

    let listener = bind_prefer("127.0.0.1", port).await.expect("bind requested port");

    assert_eq!(listener.local_addr().expect("read bound address").port(), port);
}

#[tokio::test]
async fn bind_prefer_rejects_occupied_requested_port() {
    let occupied = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.expect("occupy free port");
    let port = occupied.local_addr().expect("read occupied address").port();

    let error = bind_prefer("127.0.0.1", port).await.expect_err("occupied port should fail");

    assert!(error.to_string().contains("Address already in use"));
}

#[tokio::test]
async fn bind_prefer_rejects_invalid_host() {
    let result = tokio::time::timeout(Duration::from_secs(5), bind_prefer("256.256.256.256", 0))
        .await
        .expect("invalid host lookup should finish quickly");

    assert!(result.is_err());
}

#[tokio::test]
async fn bind_prefer_zero_port_uses_preferred_port_or_system_fallback() {
    let preferred = tokio::net::TcpListener::bind(("127.0.0.1", 4099)).await.ok();

    let listener = bind_prefer("127.0.0.1", 0).await.expect("bind automatic port");
    let port = listener.local_addr().expect("read automatic address").port();

    if preferred.is_some() {
        assert_ne!(port, 4099);
    } else {
        assert_ne!(port, 0);
    }
}

#[tokio::test]
async fn start_binds_service_and_returns_actual_address() {
    let opts = ServeOptions {
        hostname: "127.0.0.1".to_string(),
        port: 0,
        cors: vec!["https://example.test".to_string()],
    };

    let (addr, handle) = start(opts).await.expect("start gateway");

    assert!(addr.ip().is_loopback());
    assert_ne!(addr.port(), 0);

    handle.abort();
}

#[tokio::test]
async fn start_until_uses_custom_shutdown_future() {
    let opts = ServeOptions { hostname: "127.0.0.1".to_string(), port: 0, cors: Vec::new() };

    let (addr, handle) = start_until(opts, async {}).await.expect("start gateway");
    let result = tokio::time::timeout(Duration::from_secs(5), handle)
        .await
        .expect("service task should stop after shutdown")
        .expect("service task should not panic");

    assert!(addr.ip().is_loopback());
    assert_ne!(addr.port(), 0);
    assert!(result.is_ok());
}

#[tokio::test]
async fn serve_returns_actual_address_after_graceful_shutdown() {
    let opts = ServeOptions { hostname: "127.0.0.1".to_string(), port: 0, cors: Vec::new() };

    let addr = serve_until(opts, async {}).await.expect("serve should stop cleanly");

    assert!(addr.ip().is_loopback());
    assert_ne!(addr.port(), 0);
}

#[tokio::test]
async fn serve_returns_bind_error_for_occupied_port() {
    let occupied = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.expect("occupy free port");
    let port = occupied.local_addr().expect("read occupied address").port();
    let opts = ServeOptions { hostname: "127.0.0.1".to_string(), port, cors: Vec::new() };

    let error = serve(opts).await.expect_err("serve should fail before waiting on task");

    assert!(error.to_string().contains("Address already in use"));
}
