use super::*;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use vw_gateway_client::GatewayChatStreamRequest;

#[test]
fn gateway_health_probe_keeps_write_side_open_until_response_arrives() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind health probe fixture");
    let port = listener.local_addr().expect("read fixture addr").port();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept health probe client");
        stream.set_read_timeout(Some(Duration::from_millis(50))).expect("set fixture read timeout");

        let mut request = Vec::new();
        let mut buffer = [0_u8; 512];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut buffer).expect("read health probe request");
            if read == 0 {
                return;
            }
            request.extend_from_slice(&buffer[..read]);
        }

        let mut extra = [0_u8; 1];
        match stream.read(&mut extra) {
            Ok(0) => return,
            Ok(_) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(err) => panic!("unexpected fixture read error: {err}"),
        }

        stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"ok\"}",
                )
                .expect("write health probe response");
    });

    let endpoint = GatewayEndpoint::new("127.0.0.1", port);

    assert!(gateway_health_ready(&endpoint));

    server.join().expect("join health probe fixture");
}

#[test]
fn gateway_preflight_outcome_reports_started_flag() {
    assert!(!GatewayPreflightOutcome::Ready.started_gateway());
    assert!(GatewayPreflightOutcome::Started.started_gateway());
}

#[test]
fn bootstrap_config_defaults_and_json_mapping_are_stable() {
    let mut config = GatewayClientBootstrapConfig::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 42617);
    assert!(config.endpoint().auth.is_none());

    config.apply_json_root(&serde_json::json!({
        "gateway_client": {
            "host": " gateway.internal ",
            "port": 8080,
            "username": " user ",
            "password": " pass ",
            "skey": " key "
        }
    }));
    let endpoint = config.endpoint();
    assert_eq!(endpoint.normalized_host(), "gateway.internal");
    assert_eq!(endpoint.port, 8080);
    let auth = endpoint.auth.expect("auth should be attached");
    assert_eq!(auth.skey.as_deref(), Some("key"));

    config.apply_json_root(&serde_json::json!({
        "gateway_client": {
            "host": " ",
            "port": 0,
            "username": "",
            "password": "   ",
            "skey": null
        }
    }));
    assert_eq!(config.host, "gateway.internal");
    assert_eq!(config.port, 8080);
    assert_eq!(config.skey, None);
}

#[test]
fn session_seed_normalizes_optional_values_and_preserves_directory() {
    let seed = GatewaySessionSeed::new(PathBuf::from("/tmp/work"))
        .with_id(Some(" session-1 ".to_string()))
        .with_scope(Some(" project ".to_string()))
        .with_title(Some(" Title ".to_string()));

    assert_eq!(seed.id(), Some("session-1"));
    assert_eq!(seed.scope(), Some("project"));
    assert_eq!(seed.title(), Some("Title"));
    assert_eq!(seed.directory(), PathBuf::from("/tmp/work").as_path());

    let cleared = seed
        .clone()
        .with_id(Some(" ".to_string()))
        .with_scope(None)
        .with_title(Some(String::new()));
    assert_eq!(cleared.id(), None);
    assert_eq!(cleared.scope(), None);
    assert_eq!(cleared.title(), None);
}

#[test]
fn gateway_runtime_accessors_binding_resolution_and_directory_value_are_stable() {
    let client = GatewayClient::new(GatewayEndpoint::new("127.0.0.1", 42617))
        .expect("client should construct");
    let mut runtime = GatewayUiRuntime::new(
        client,
        GatewaySessionSeed::new(PathBuf::from("/tmp/work")).with_id(Some("seed".to_string())),
    );

    assert_eq!(runtime.endpoint().describe(), "127.0.0.1:42617");
    assert_eq!(runtime.session_id(), Some("seed"));
    assert_eq!(runtime.resolve_session_id(None), Ok("seed"));
    assert_eq!(runtime.resolve_session_id(Some(" explicit ")), Ok("explicit"));
    assert_eq!(runtime.directory_value().as_deref(), Some("/tmp/work"));

    runtime.bind_session_seed(
        Some(" rebound ".to_string()),
        Some(" scope ".to_string()),
        Some(" title ".to_string()),
    );
    assert_eq!(runtime.session_id(), Some("rebound"));
    assert_eq!(runtime.scope(), Some("scope"));
    assert_eq!(runtime.title(), Some("title"));

    let unbound = GatewayUiRuntime::new(
        GatewayClient::new(GatewayEndpoint::new("127.0.0.1", 42617))
            .expect("client should construct"),
        GatewaySessionSeed::new(PathBuf::from("/tmp/work")),
    );
    assert_eq!(
        unbound.resolve_session_id(None),
        Err("gateway runtime session id is required".to_string())
    );
}

#[test]
fn prepare_stream_request_fills_missing_session_without_overwriting_explicit_session() {
    let runtime = GatewayUiRuntime::new(
        GatewayClient::new(GatewayEndpoint::new("127.0.0.1", 42617))
            .expect("client should construct"),
        GatewaySessionSeed::new(PathBuf::from("/tmp/work")).with_id(Some("runtime-session".into())),
    );

    let prepared = runtime.prepare_stream_request(&GatewayChatStreamRequest::default());
    assert_eq!(prepared.session_id.as_ref().map(|id| id.as_ref()), Some("runtime-session"));

    let explicit = GatewayChatStreamRequest {
        session_id: Some(SessionId::from("explicit-session")),
        model: Some("gpt".to_string()),
        ..GatewayChatStreamRequest::default()
    };
    let prepared = runtime.prepare_stream_request(&explicit);
    assert_eq!(prepared.session_id.as_ref().map(|id| id.as_ref()), Some("explicit-session"));
    assert_eq!(prepared.model.as_deref(), Some("gpt"));
}

#[test]
fn optional_string_normalization_error_annotation_and_stream_terminal_finalization() {
    assert_eq!(normalize_optional_str_ref(Some("  value  ")), Some("value"));
    assert_eq!(normalize_optional_str_ref(Some("  ")), None);
    assert_eq!(normalize_optional_str_ref(None), None);

    let endpoint = GatewayEndpoint::new("127.0.0.1", 9);
    let annotated =
        annotate_gateway_transport_error("connection refused by fixture".to_string(), &endpoint);
    assert!(annotated.contains("Gateway endpoint 127.0.0.1:9 is unavailable"));
    assert_eq!(
        annotate_gateway_transport_error("application error".to_string(), &endpoint),
        "application error"
    );
    assert!(looks_like_gateway_transport_error("tcp connect error"));
    assert!(!looks_like_gateway_transport_error("validation failed"));

    assert_eq!(
        finalize_stream_terminal(Ok(()), None),
        UiRuntimeTerminalEvent::Error("gateway stream closed before terminal event".to_string())
    );
    assert_eq!(
        finalize_stream_terminal(
            Err("transport failed".to_string()),
            Some(UiRuntimeTerminalEvent::Done {
                finish_reason: Some("stop".to_string()),
                usage: None,
                message_id: None,
                parent_message_id: None,
            }),
        ),
        UiRuntimeTerminalEvent::Done {
            finish_reason: Some("stop".to_string()),
            usage: None,
            message_id: None,
            parent_message_id: None,
        }
    );
    assert!(matches!(
        cancelled_by_consumer_terminal(),
        UiRuntimeTerminalEvent::Cancelled { reason: Some(_), .. }
    ));
}
