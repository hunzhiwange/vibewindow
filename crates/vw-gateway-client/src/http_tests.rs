use std::net::TcpListener;

use serde_json::{Value, json};

use crate::client::test_support;
use crate::endpoint::{GatewayAuth, GatewayEndpoint};

#[test]
fn directory_query_and_format_query_skip_blank_values() {
    assert!(super::directory_query(None).is_empty());
    assert!(super::directory_query(Some("   ")).is_empty());
    assert_eq!(
        super::directory_query(Some("/tmp/project")),
        vec![("directory".to_string(), "/tmp/project".to_string())]
    );
    assert_eq!(super::format_query(&[]), "");
    assert_eq!(
        super::format_query(&[
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "two".to_string())
        ]),
        "a=1&b=two"
    );
}

#[test]
fn success_logging_is_quiet_only_for_git_command() {
    assert!(super::is_quiet_success_path("/v1/git/command"));
    assert!(!super::is_quiet_success_path("/v1/git/commit"));
    assert!(!super::is_quiet_success_path("/v1/project/worktrees"));
}

#[test]
fn apply_auth_prefers_skey_as_bearer_token() {
    let endpoint = GatewayEndpoint::new("127.0.0.1", 1234).with_auth(GatewayAuth {
        skey: Some("skey-1".to_string()),
    });
    let request = super::apply_auth(reqwest::Client::new().get("http://example.test"), &endpoint)
        .build()
        .expect("request");

    assert_eq!(request.headers()["authorization"].to_str().unwrap(), "Bearer skey-1");
    assert!(!request.headers().contains_key("x-skey"));
}

#[test]
fn apply_auth_ignores_blank_skey() {
    let endpoint = GatewayEndpoint::new("127.0.0.1", 1234).with_auth(GatewayAuth {
        skey: Some(" ".to_string()),
    });
    let request = super::apply_auth(reqwest::Client::new().get("http://example.test"), &endpoint)
        .build()
        .expect("request");

    assert!(!request.headers().contains_key("authorization"));
    assert!(!request.headers().contains_key("x-skey"));
}

#[test]
fn apply_auth_without_credentials_leaves_auth_headers_absent() {
    let anonymous = GatewayEndpoint::new("127.0.0.1", 1234);
    let request = super::apply_auth(reqwest::Client::new().get("http://example.test"), &anonymous)
        .build()
        .expect("request");
    assert!(!request.headers().contains_key("authorization"));
    assert!(!request.headers().contains_key("x-skey"));

    let empty_auth = GatewayEndpoint::new("127.0.0.1", 1234).with_auth(GatewayAuth::default());
    let request = super::apply_auth(reqwest::Client::new().get("http://example.test"), &empty_auth)
        .build()
        .expect("request");

    assert!(!request.headers().contains_key("authorization"));
    assert!(!request.headers().contains_key("x-skey"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_blocking_auth_matches_async_auth_behavior() {
    let endpoint = GatewayEndpoint::new("127.0.0.1", 1234).with_auth(GatewayAuth {
        skey: Some("skey-2".to_string()),
    });
    let request = super::apply_blocking_auth(
        reqwest::blocking::Client::new().get("http://example.test"),
        &endpoint,
    )
    .build()
    .expect("request");

    assert_eq!(request.headers()["authorization"].to_str().unwrap(), "Bearer skey-2");
    assert!(!request.headers().contains_key("x-skey"));
}

#[tokio::test]
async fn parse_json_response_decodes_success_and_reports_decode_failures() {
    let server = test_support::server_raw(vec![
        (200, r#"{"ok":true}"#.to_string()),
        (200, "not-json".to_string()),
    ]);
    let endpoint = GatewayEndpoint::new("127.0.0.1", 0);
    let client = reqwest::Client::new();

    let response = client.get(format!("{}/ok", server.base_url())).send().await.unwrap();
    let value: Value = super::parse_json_response("GET", &endpoint, "/ok", response).await.unwrap();
    assert_eq!(value, json!({"ok": true}));
    let response = client.get(format!("{}/bad-json", server.base_url())).send().await.unwrap();
    let error = super::parse_json_response::<Value>("GET", &endpoint, "/bad-json", response)
        .await
        .unwrap_err();
    assert!(!error.trim().is_empty());

    assert_eq!(server.take_request().path, "/ok");
    assert_eq!(server.take_request().path, "/bad-json");
    server.join();
}

#[tokio::test]
async fn response_error_includes_status_and_body_when_present() {
    let server = test_support::server(vec![(500, json!({"error": "bad"}))]);
    let endpoint = GatewayEndpoint::new("127.0.0.1", 0);
    let response =
        reqwest::Client::new().get(format!("{}/failure", server.base_url())).send().await.unwrap();

    let error = super::response_error("GET", &endpoint, "/failure", response).await;

    assert!(error.contains("500 Internal Server Error"));
    assert!(error.contains("\"bad\""));
    assert_eq!(server.take_request().path, "/failure");
    server.join();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn response_error_blocking_includes_status_and_body_when_present() {
    let server = test_support::server(vec![(404, json!({"error": "missing"}))]);
    let endpoint = GatewayEndpoint::new("127.0.0.1", 0);
    let response = reqwest::blocking::Client::new()
        .get(format!("{}/missing", server.base_url()))
        .send()
        .unwrap();

    let error = super::response_error_blocking("GET", &endpoint, "/missing", response);

    assert!(error.contains("404 Not Found"));
    assert!(error.contains("\"missing\""));
    assert_eq!(server.take_request().path, "/missing");
    server.join();
}

#[tokio::test]
async fn transport_error_returns_original_reqwest_message() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind unused port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    let endpoint = GatewayEndpoint::new("127.0.0.1", port);
    let error = reqwest::Client::new()
        .get(format!("{}/closed", endpoint.base_url()))
        .send()
        .await
        .expect_err("closed port should fail");

    let message = super::transport_error("GET", &endpoint, "/closed", error);

    assert!(!message.trim().is_empty());
}

#[test]
fn log_helpers_accept_empty_and_non_empty_request_metadata() {
    let endpoint = GatewayEndpoint::new(" ", 8080);

    super::log_request::<Value>("GET", &endpoint, "/v1/plain", &[], None);
    super::log_request(
        "POST",
        &endpoint,
        "/v1/json",
        &[("q".to_string(), "1".to_string())],
        Some(&json!({"x": 1})),
    );
    super::log_request_succeeded("GET", &endpoint, "/v1/git/command");
    super::log_request_succeeded("GET", &endpoint, "/v1/other");
}
