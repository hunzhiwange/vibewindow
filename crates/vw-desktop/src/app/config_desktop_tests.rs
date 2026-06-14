use std::ffi::OsStr;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::thread::JoinHandle;

use tempfile::TempDir;
use vw_config_types::ui::{GatewayClientServerConfig, GatewayClientSystemSettingsConfig};

const SOURCE: &str = include_str!("config_desktop.rs");
static CONFIG_DESKTOP_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Clone)]
struct RecordedRequest {
    method: String,
    path: String,
    body: String,
}

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set_os(key: &'static str, value: &OsStr) -> Self {
        let previous = std::env::var_os(key);
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

struct TestServer {
    port: u16,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    handle: Option<JoinHandle<()>>,
}

impl TestServer {
    fn start(
        expected_requests: usize,
        handler: impl Fn(&RecordedRequest) -> (u16, &'static str) + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let port = listener.local_addr().expect("test server address should be available").port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = Arc::clone(&requests);
        let handler = Arc::new(handler);
        let handle = std::thread::spawn(move || {
            for stream in listener.incoming().take(expected_requests) {
                let mut stream = stream.expect("request stream should open");
                let request = read_http_request(&mut stream);
                let (status, body) = handler(&request);
                server_requests.lock().expect("requests lock should not be poisoned").push(request);
                write_http_response(&mut stream, status, body);
            }
        });

        Self { port, requests, handle: Some(handle) }
    }

    fn take_requests(mut self) -> Vec<RecordedRequest> {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("test server should finish");
        }
        self.requests.lock().expect("requests lock should not be poisoned").clone()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.handle.take();
    }
}

fn read_http_request(stream: &mut TcpStream) -> RecordedRequest {
    let mut buffer = Vec::new();
    let mut chunk = [0; 1024];
    loop {
        let read = stream.read(&mut chunk).expect("request should be readable");
        assert!(read > 0, "request should contain headers");
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let header_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("request header terminator should exist")
        + 4;
    let header_text =
        String::from_utf8(buffer[..header_end].to_vec()).expect("headers should be utf8");
    let mut lines = header_text.lines();
    let request_line = lines.next().expect("request line should exist");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().expect("method should exist").to_string();
    let path = request_parts.next().expect("path should exist").to_string();
    let content_length = header_text
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().expect("content length should parse"))
        })
        .unwrap_or(0);

    while buffer.len() < header_end + content_length {
        let read = stream.read(&mut chunk).expect("body should be readable");
        assert!(read > 0, "request body ended early");
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body = String::from_utf8(buffer[header_end..header_end + content_length].to_vec())
        .expect("body should be utf8");
    RecordedRequest { method, path, body }
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) {
    let reason = if status == 200 { "OK" } else { "ERROR" };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).expect("response should write");
}

fn env_lock() -> MutexGuard<'static, ()> {
    CONFIG_DESKTOP_ENV_LOCK.lock().expect("env lock should not be poisoned")
}

fn temp_home() -> (MutexGuard<'static, ()>, TempDir, EnvGuard) {
    let lock = env_lock();
    let home = tempfile::tempdir().expect("temp home should be created");
    let guard = EnvGuard::set_os("HOME", home.path().as_os_str());
    (lock, home, guard)
}

fn gateway_config(port: u16) -> GatewayClientSystemSettingsConfig {
    let mut config = GatewayClientSystemSettingsConfig::default();
    config.set_servers(
        vec![GatewayClientServerConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            host: "127.0.0.1".to_string(),
            port,
            bearer_token: "test-token".to_string(),
            ..GatewayClientServerConfig::default()
        }],
        "test".to_string(),
    );
    config
}

fn configure_gateway(port: u16) -> (MutexGuard<'static, ()>, TempDir, EnvGuard) {
    let home = temp_home();
    super::super::system_settings::save_gateway_client_bootstrap_config(&gateway_config(port));
    home
}

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn config_desktop_tests_keeps_planned_coverage_targets() {
    for name in [
        "load_app_config_async",
        "save_app_config_async",
        "update_agents_compat_registry_result_async",
        "update_agents_compat_registry_async",
        "load_app_config",
        "save_app_config",
        "set_config_field",
        "load_project_chat_preferences",
        "save_project_chat_preferences",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn load_app_config_async_returns_object_preferences() {
    let server = TestServer::start(1, |request| {
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/v1/desktop/preferences");
        (200, r#"{"agent":{"enabled":true}}"#)
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    let value = super::super::gateway::run_gateway_call(super::load_app_config_async())
        .expect("preferences should load");

    assert_eq!(value, serde_json::json!({"agent":{"enabled":true}}));
    assert_eq!(server.take_requests().len(), 1);
}

#[test]
fn load_app_config_async_replaces_non_object_preferences() {
    let server = TestServer::start(1, |request| {
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/v1/desktop/preferences");
        (200, r#"["invalid"]"#)
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    let value = super::super::gateway::run_gateway_call(super::load_app_config_async())
        .expect("preferences should load");

    assert_eq!(value, serde_json::json!({}));
    assert_eq!(server.take_requests().len(), 1);
}

#[test]
fn load_app_config_falls_back_to_empty_object_on_gateway_error() {
    let server = TestServer::start(1, |request| {
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/v1/desktop/preferences");
        (500, r#"{"error":"unavailable"}"#)
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    let value = super::load_app_config();

    assert_eq!(value, serde_json::json!({}));
    assert_eq!(server.take_requests().len(), 1);
}

#[test]
fn save_app_config_async_sends_object_patch() {
    let server = TestServer::start(1, |request| {
        assert_eq!(request.method, "PATCH");
        assert_eq!(request.path, "/v1/desktop/preferences");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
            serde_json::json!({"theme":"dark"})
        );
        (200, r#"{"theme":"dark"}"#)
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    super::super::gateway::run_gateway_call(super::save_app_config_async(serde_json::json!({
        "theme": "dark"
    })))
    .expect("preferences should save");

    assert_eq!(server.take_requests().len(), 1);
}

#[test]
fn save_app_config_async_replaces_non_object_patch() {
    let server = TestServer::start(1, |request| {
        assert_eq!(request.method, "PATCH");
        assert_eq!(request.path, "/v1/desktop/preferences");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
            serde_json::json!({})
        );
        (200, r#"{}"#)
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    super::super::gateway::run_gateway_call(super::save_app_config_async(serde_json::json!(42)))
        .expect("preferences should save");

    assert_eq!(server.take_requests().len(), 1);
}

#[test]
fn update_agents_compat_registry_result_async_creates_agent_section() {
    let server =
        TestServer::start(2, |request| match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/v1/desktop/preferences") => (200, r#"{"agent":"legacy"}"#),
            ("PATCH", "/v1/desktop/preferences") => {
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                    serde_json::json!({"agent":{"compat_registry":"enabled"}})
                );
                (200, r#"{"agent":{"compat_registry":"enabled"}}"#)
            }
            _ => (404, r#"{"error":"unexpected"}"#),
        });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    super::super::gateway::run_gateway_call(super::update_agents_compat_registry_result_async(
        |agent| {
            agent.insert("compat_registry".to_string(), serde_json::json!("enabled"));
        },
    ))
    .expect("agent registry should update");

    assert_eq!(server.take_requests().len(), 2);
}

#[test]
fn project_chat_preferences_async_round_trips_gateway_shape() {
    let server = TestServer::start(2, |request| match request.method.as_str() {
        "GET" => {
            assert_eq!(request.path, "/v1/desktop/project-preferences?project_path=%2Ftmp%2Fdemo");
            (200, r#"{"model":"openai/gpt-5","auto_model":true,"acp_agent":"coder"}"#)
        }
        "PUT" => {
            assert_eq!(request.path, "/v1/desktop/project-preferences?project_path=%2Ftmp%2Fdemo");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                serde_json::json!({"model":"local/model","auto_model":false,"acp_agent":null})
            );
            (200, r#"{}"#)
        }
        _ => (404, r#"{"error":"unexpected"}"#),
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    let loaded = super::super::gateway::run_gateway_call(
        super::load_project_chat_preferences_async("/tmp/demo"),
    )
    .expect("project preferences should load");
    super::super::gateway::run_gateway_call(super::save_project_chat_preferences_async(
        "/tmp/demo",
        "local/model",
        false,
        None,
    ))
    .expect("project preferences should save");

    assert_eq!(loaded, Some(("openai/gpt-5".to_string(), true, Some("coder".to_string()))));
    assert_eq!(server.take_requests().len(), 2);
}

#[test]
fn tool_content_wrappers_use_expected_tool_types() {
    let server = TestServer::start(3, |request| match request.path.as_str() {
        "/v1/desktop/tool-content/json" => {
            assert_eq!(request.method, "PUT");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                serde_json::json!({"content":"{}"})
            );
            (200, r#"{}"#)
        }
        "/v1/desktop/tool-content/sql" => {
            assert_eq!(request.method, "PUT");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                serde_json::json!({"content":"select 1"})
            );
            (200, r#"{}"#)
        }
        "/v1/desktop/tool-content/html" => {
            assert_eq!(request.method, "PUT");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                serde_json::json!({"content":"<main></main>"})
            );
            (200, r#"{}"#)
        }
        _ => (404, r#"{"error":"unexpected"}"#),
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    super::super::gateway::run_gateway_call(super::save_json_tool_content_async("{}"))
        .expect("json tool content should save");
    super::super::gateway::run_gateway_call(super::save_sql_tool_content_async("select 1"))
        .expect("sql tool content should save");
    super::super::gateway::run_gateway_call(super::save_html_tool_content_async("<main></main>"))
        .expect("html tool content should save");

    assert_eq!(server.take_requests().len(), 3);
}

#[test]
fn mindmap_tabs_async_handles_null_and_owned_save() {
    let server = TestServer::start(2, |request| match request.method.as_str() {
        "GET" => {
            assert_eq!(request.path, "/v1/desktop/mindmap-tabs");
            (200, "null")
        }
        "PUT" => {
            assert_eq!(request.path, "/v1/desktop/mindmap-tabs");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                serde_json::json!({"tabs":[]})
            );
            (200, r#"{}"#)
        }
        _ => (404, r#"{"error":"unexpected"}"#),
    });
    let (_lock, _home, _guard) = configure_gateway(server.port);

    let loaded = super::super::gateway::run_gateway_call(super::load_mindmap_tabs_async())
        .expect("mindmap tabs should load");
    super::super::gateway::run_gateway_call(super::save_mindmap_tabs_owned(serde_json::json!({
        "tabs": []
    })))
    .expect("mindmap tabs should save");

    assert_eq!(loaded, None);
    assert_eq!(server.take_requests().len(), 2);
}
