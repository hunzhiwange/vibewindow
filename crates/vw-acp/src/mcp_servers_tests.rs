use super::*;
use agent_client_protocol::McpServer;
use serde_json::{Value, json};
use std::path::PathBuf;

#[test]
fn parse_mcp_servers_accepts_supported_server_types() {
    let servers = parse_mcp_servers(
        &json!([
            {"name": "local", "command": "agent", "args": ["--json"], "env": [{"name": "A", "value": "B"}]},
            {"name": "remote", "type": "http", "url": "https://example.test/mcp", "headers": [{"name": "X-Test", "value": "1"}]},
            {"name": "events", "type": "sse", "url": "https://example.test/sse", "_meta": {"k": "v"}}
        ]),
        "config.json",
    )
    .expect("valid server config");

    assert_eq!(servers.len(), 3);

    let McpServer::Stdio(stdio) = &servers[0] else {
        panic!("first server must use stdio transport");
    };
    assert_eq!(stdio.name, "local");
    assert_eq!(stdio.command, PathBuf::from("agent"));
    assert_eq!(stdio.args, vec!["--json".to_string()]);
    assert_eq!(stdio.env.len(), 1);
    assert_eq!(stdio.env[0].name, "A");
    assert_eq!(stdio.env[0].value, "B");
    assert_eq!(stdio.meta, None);

    let McpServer::Http(http) = &servers[1] else {
        panic!("second server must use http transport");
    };
    assert_eq!(http.name, "remote");
    assert_eq!(http.url, "https://example.test/mcp");
    assert_eq!(http.headers.len(), 1);
    assert_eq!(http.headers[0].name, "X-Test");
    assert_eq!(http.headers[0].value, "1");
    assert_eq!(http.meta, None);

    let McpServer::Sse(sse) = &servers[2] else {
        panic!("third server must use sse transport");
    };
    assert_eq!(sse.name, "events");
    assert_eq!(sse.url, "https://example.test/sse");
    assert!(sse.headers.is_empty());
    assert_eq!(sse.meta.as_ref().and_then(|meta| meta.get("k")).and_then(Value::as_str), Some("v"));
}

#[test]
fn parse_mcp_servers_reports_precise_field_path() {
    let error = parse_mcp_servers(&json!([{"name": "bad", "type": "stdio"}]), "config.json")
        .expect_err("missing command must fail");

    assert_eq!(
        error.to_string(),
        "Invalid mcpServers[0] in config.json.command: expected non-empty string"
    );
}

#[test]
fn parse_mcp_servers_trims_required_strings_and_accepts_null_meta() {
    let servers = parse_mcp_servers(
        &json!([
            {"name": " local ", "type": "stdio", "command": " agent ", "args": [], "env": [], "_meta": null},
            {"name": " remote ", "type": "http", "url": " https://example.test/mcp ", "headers": []}
        ]),
        "config.json",
    )
    .expect("valid server config");

    let McpServer::Stdio(stdio) = &servers[0] else {
        panic!("first server must use stdio transport");
    };
    assert_eq!(stdio.name, "local");
    assert_eq!(stdio.command, PathBuf::from("agent"));
    assert!(stdio.args.is_empty());
    assert!(stdio.env.is_empty());
    assert_eq!(stdio.meta, None);

    let McpServer::Http(http) = &servers[1] else {
        panic!("second server must use http transport");
    };
    assert_eq!(http.name, "remote");
    assert_eq!(http.url, "https://example.test/mcp");
    assert!(http.headers.is_empty());
}

#[test]
fn parse_mcp_servers_with_field_name_uses_custom_field_in_errors() {
    let error = parse_mcp_servers_with_field_name(&json!({}), "queue.json", "servers")
        .expect_err("non-array field must fail");

    assert_eq!(error.to_string(), "Invalid servers in queue.json: expected array");
}

#[test]
fn parse_optional_mcp_servers_distinguishes_absent_and_present_fields() {
    let absent = parse_optional_mcp_servers(None, "config.json").expect("missing field is allowed");
    assert_eq!(absent, None);

    let present =
        parse_optional_mcp_servers(Some(&json!([])), "config.json").expect("empty list is valid");
    assert_eq!(present, Some(Vec::new()));
}

#[test]
fn parse_optional_mcp_servers_with_field_name_reports_nested_errors() {
    let error = parse_optional_mcp_servers_with_field_name(
        Some(&json!([{"name": "remote", "type": "http"}])),
        "queue.json",
        "servers",
    )
    .expect_err("missing http url must fail");

    assert_eq!(
        error.to_string(),
        "Invalid servers[0] in queue.json.url: expected non-empty string"
    );
}

#[test]
fn parse_mcp_servers_rejects_invalid_server_shapes() {
    let cases = [
        (json!([null]), "Invalid mcpServers[0] in config.json: expected object"),
        (
            json!([{"type": "stdio", "command": "agent"}]),
            "Invalid mcpServers[0] in config.json.name: expected non-empty string",
        ),
        (
            json!([{"name": "", "type": "stdio", "command": "agent"}]),
            "Invalid mcpServers[0] in config.json.name: expected non-empty string",
        ),
        (
            json!([{"name": "bad", "type": ""}]),
            "Invalid mcpServers[0] in config.json.type: expected non-empty string",
        ),
        (
            json!([{"name": "bad", "type": "websocket"}]),
            "Invalid mcpServers[0] in config.json.type: expected http, sse, or stdio",
        ),
        (
            json!([{"name": "bad", "type": "sse"}]),
            "Invalid mcpServers[0] in config.json.url: expected non-empty string",
        ),
        (
            json!([{"name": "bad", "type": "http", "url": ""}]),
            "Invalid mcpServers[0] in config.json.url: expected non-empty string",
        ),
        (
            json!([{"name": "bad", "type": "stdio", "command": ""}]),
            "Invalid mcpServers[0] in config.json.command: expected non-empty string",
        ),
        (
            json!([{"name": "bad", "type": "stdio", "_meta": []}]),
            "Invalid mcpServers[0] in config.json._meta: expected object or null",
        ),
    ];

    for (raw_servers, expected_error) in cases {
        let error = parse_mcp_servers(&raw_servers, "config.json")
            .expect_err("invalid server config must fail");

        assert_eq!(error.to_string(), expected_error);
    }
}

#[test]
fn parse_mcp_servers_rejects_invalid_header_entries() {
    let cases = [
        (
            json!([{"name": "remote", "type": "http", "url": "https://example.test", "headers": {}}]),
            "Invalid mcpServers[0] in config.json.headers: expected array",
        ),
        (
            json!([{"name": "remote", "type": "http", "url": "https://example.test", "headers": [null]}]),
            "Invalid mcpServers[0] in config.json.headers[0]: expected object",
        ),
        (
            json!([{"name": "remote", "type": "http", "url": "https://example.test", "headers": [{"value": "1"}]}]),
            "Invalid mcpServers[0] in config.json.headers[0].name: expected non-empty string",
        ),
        (
            json!([{"name": "remote", "type": "http", "url": "https://example.test", "headers": [{"name": "X-Test"}]}]),
            "Invalid mcpServers[0] in config.json.headers[0].value: expected non-empty string",
        ),
    ];

    for (raw_servers, expected_error) in cases {
        let error =
            parse_mcp_servers(&raw_servers, "config.json").expect_err("invalid headers must fail");

        assert_eq!(error.to_string(), expected_error);
    }
}

#[test]
fn parse_mcp_servers_rejects_invalid_args_and_env_entries() {
    let cases = [
        (
            json!([{"name": "local", "command": "agent", "args": {}}]),
            "Invalid mcpServers[0] in config.json.args: expected array",
        ),
        (
            json!([{"name": "local", "command": "agent", "args": [1]}]),
            "Invalid mcpServers[0] in config.json.args[0]: expected string",
        ),
        (
            json!([{"name": "local", "command": "agent", "env": {}}]),
            "Invalid mcpServers[0] in config.json.env: expected array",
        ),
        (
            json!([{"name": "local", "command": "agent", "env": [null]}]),
            "Invalid mcpServers[0] in config.json.env[0]: expected object",
        ),
        (
            json!([{"name": "local", "command": "agent", "env": [{"value": "1"}]}]),
            "Invalid mcpServers[0] in config.json.env[0].name: expected non-empty string",
        ),
        (
            json!([{"name": "local", "command": "agent", "env": [{"name": "A"}]}]),
            "Invalid mcpServers[0] in config.json.env[0].value: expected non-empty string",
        ),
    ];

    for (raw_servers, expected_error) in cases {
        let error = parse_mcp_servers(&raw_servers, "config.json")
            .expect_err("invalid stdio detail must fail");

        assert_eq!(error.to_string(), expected_error);
    }
}
