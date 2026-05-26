use super::*;
use agent_client_protocol::McpServer;
use serde_json::json;

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
    assert!(matches!(servers[0], McpServer::Stdio(_)));
    assert!(matches!(servers[1], McpServer::Http(_)));
    assert!(matches!(servers[2], McpServer::Sse(_)));
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
