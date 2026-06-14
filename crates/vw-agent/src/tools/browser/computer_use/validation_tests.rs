use super::*;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::browser::computer_use::ComputerUseConfig;
use std::sync::Arc;
use tempfile::TempDir;

fn client(config: ComputerUseConfig, allowed_domains: Vec<String>) -> ComputerUseClient {
    ComputerUseClient::new(Arc::new(SecurityPolicy::default()), allowed_domains, None, config)
}

fn client_with_policy(
    security: SecurityPolicy,
    config: ComputerUseConfig,
    allowed_domains: Vec<String>,
) -> ComputerUseClient {
    ComputerUseClient::new(Arc::new(security), allowed_domains, None, config)
}

#[test]
fn endpoint_url_keeps_remote_public_hosts_https_only() {
    let local = client(ComputerUseConfig::default(), vec!["example.com".into()]);
    assert!(local.endpoint_url().is_ok());

    let remote_http = client(
        ComputerUseConfig {
            endpoint: "http://example.com/actions".into(),
            allow_remote_endpoint: true,
            ..Default::default()
        },
        vec!["example.com".into()],
    );
    assert!(remote_http.endpoint_url().is_err());

    let remote_https = client(
        ComputerUseConfig {
            endpoint: "https://example.com/actions".into(),
            allow_remote_endpoint: true,
            ..Default::default()
        },
        vec!["example.com".into()],
    );
    assert!(remote_https.endpoint_url().is_ok());
}

#[test]
fn endpoint_url_rejects_empty_bad_scheme_public_and_zero_timeout() {
    for (config, expected) in [
        (
            ComputerUseConfig { endpoint: "   ".into(), ..Default::default() },
            "endpoint cannot be empty",
        ),
        (
            ComputerUseConfig { endpoint: "ftp://127.0.0.1/actions".into(), ..Default::default() },
            "must use http:// or https://",
        ),
        (
            ComputerUseConfig { endpoint: "not a url".into(), ..Default::default() },
            "Invalid browser.computer_use.endpoint",
        ),
        (
            ComputerUseConfig {
                endpoint: "https://example.com/actions".into(),
                allow_remote_endpoint: false,
                ..Default::default()
            },
            "host 'example.com' is public",
        ),
        (ComputerUseConfig { timeout_ms: 0, ..Default::default() }, "timeout_ms must be > 0"),
    ] {
        let c = client(config, vec!["example.com".into()]);
        let err = c.endpoint_url().expect_err("endpoint should be rejected");
        assert!(err.to_string().contains(expected), "{err}");
    }
}

#[test]
fn available_returns_boolean_for_valid_local_endpoint() {
    let c = client(ComputerUseConfig::default(), vec!["example.com".into()]);

    assert!(matches!(c.available(), Ok(true) | Ok(false)));
}

#[test]
fn validate_url_rejects_unsafe_shapes_and_accepts_allowlisted_hosts() {
    let c = client(ComputerUseConfig::default(), vec!["example.com".into()]);

    assert!(c.validate_url("https://example.com/path").is_ok());
    assert!(c.validate_url("http://www.example.com/path").is_ok());

    for (url, expected) in [
        ("   ", "URL cannot be empty"),
        ("file:///tmp/a.txt", "file:// URLs are not allowed"),
        ("ssh://example.com", "Only http:// and https:// URLs are allowed"),
        ("http://127.0.0.1:3000", "Blocked local/private host"),
        ("https://example.net", "not in browser.allowed_domains"),
    ] {
        let err = c.validate_url(url).expect_err("URL should be rejected");
        assert!(err.to_string().contains(expected), "{url}: {err}");
    }

    let no_domains = client(ComputerUseConfig::default(), vec![]);
    let err = no_domains.validate_url("https://example.com").unwrap_err();
    assert!(err.to_string().contains("no allowed_domains configured"));
}

#[test]
fn validate_coordinate_enforces_lower_and_upper_bounds() {
    let c = client(ComputerUseConfig::default(), vec![]);
    assert!(c.validate_coordinate("x", 0, Some(10)).is_ok());
    assert!(c.validate_coordinate("x", -1, Some(10)).is_err());
    assert!(c.validate_coordinate("x", 11, Some(10)).is_err());
    assert!(c.validate_coordinate("x", 1, Some(-1)).is_err());
}

#[test]
fn read_required_i64_rejects_missing_or_wrong_type() {
    let params = serde_json::json!({"x": 3, "y": "4"}).as_object().unwrap().clone();
    assert_eq!(read_required_i64(&params, "x").unwrap(), 3);
    assert!(read_required_i64(&params, "y").is_err());
    assert!(read_required_i64(&params, "z").is_err());
}

#[test]
fn validate_output_path_rejects_escape_empty_and_null_paths() {
    let c = client(ComputerUseConfig::default(), vec![]);

    assert!(c.validate_output_path("path", "screens/out.png").is_ok());

    for (path, expected) in [
        (" ", "path cannot be empty"),
        ("bad\0path", "invalid null byte"),
        ("../escape.png", "must stay within the workspace"),
        ("/tmp/escape.png", "must stay within the workspace"),
    ] {
        let err = c.validate_output_path("path", path).expect_err("path should be rejected");
        assert!(err.to_string().contains(expected), "{path:?}: {err}");
    }
}

#[tokio::test]
async fn resolve_output_path_creates_parent_and_rejects_non_files() {
    let tmp = TempDir::new().expect("tempdir should create");
    let mut security = SecurityPolicy::default();
    security.workspace_dir = tmp.path().to_path_buf();
    let c = client_with_policy(security, ComputerUseConfig::default(), vec![]);

    let resolved = c
        .resolve_output_path_for_write("path", "screens/out.png")
        .await
        .expect("relative output should resolve");
    assert!(resolved.ends_with("screens/out.png"));
    assert!(resolved.parent().unwrap().is_dir());

    tokio::fs::create_dir_all(tmp.path().join("dir-output")).await.unwrap();
    let err = c
        .resolve_output_path_for_write("path", "dir-output")
        .await
        .expect_err("directory output should fail");
    assert!(err.to_string().contains("not a regular file"));
}

#[test]
fn validate_action_covers_computer_use_specific_rules() {
    let c = client(
        ComputerUseConfig {
            max_coordinate_x: Some(10),
            max_coordinate_y: Some(20),
            ..Default::default()
        },
        vec!["example.com".into()],
    );

    let open = serde_json::json!({"url": "https://example.com"}).as_object().unwrap().clone();
    assert!(c.validate_action("open", &open).is_ok());

    let mouse = serde_json::json!({"x": 10, "y": 20}).as_object().unwrap().clone();
    assert!(c.validate_action("mouse_move", &mouse).is_ok());
    assert!(c.validate_action("mouse_click", &mouse).is_ok());

    let drag = serde_json::json!({"from_x": 0, "from_y": 1, "to_x": 10, "to_y": 20})
        .as_object()
        .unwrap()
        .clone();
    assert!(c.validate_action("mouse_drag", &drag).is_ok());

    let key_type = serde_json::json!({"text": "hello"}).as_object().unwrap().clone();
    assert!(c.validate_action("key_type", &key_type).is_ok());

    let key_press = serde_json::json!({"key": "Control+C"}).as_object().unwrap().clone();
    assert!(c.validate_action("key_press", &key_press).is_ok());

    let capture = serde_json::json!({"path": "screens/out.png"}).as_object().unwrap().clone();
    assert!(c.validate_action("screen_capture", &capture).is_ok());
    assert!(c.validate_action("unknown", &serde_json::Map::new()).is_ok());
}

#[test]
fn validate_action_reports_missing_or_invalid_parameters() {
    let c = client(
        ComputerUseConfig {
            max_coordinate_x: Some(10),
            max_coordinate_y: Some(20),
            ..Default::default()
        },
        vec!["example.com".into()],
    );

    for (action, params, expected) in [
        ("open", serde_json::json!({}), "Missing 'url'"),
        ("mouse_move", serde_json::json!({"x": 1}), "Missing or invalid 'y'"),
        ("mouse_click", serde_json::json!({"x": 11, "y": 1}), "exceeds configured limit"),
        (
            "mouse_drag",
            serde_json::json!({"from_x": 0, "from_y": 0, "to_x": 0}),
            "Missing or invalid 'to_y'",
        ),
        ("key_type", serde_json::json!({"text": "   "}), "must not be empty"),
        ("key_type", serde_json::json!({"text": "x".repeat(4097)}), "exceeds maximum length"),
        ("key_press", serde_json::json!({"key": "bad key!"}), "must be 1-32 chars"),
        (
            "screen_capture",
            serde_json::json!({"path": "../out.png"}),
            "must stay within the workspace",
        ),
    ] {
        let params = params.as_object().unwrap().clone();
        let err = c.validate_action(action, &params).expect_err("action should fail");
        assert!(err.to_string().contains(expected), "{action}: {err}");
    }
}
