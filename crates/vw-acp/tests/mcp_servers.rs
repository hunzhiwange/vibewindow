//! 验证 MCP server 配置解析。
//!
//! 这些测试覆盖 HTTP 与 stdio server、可选字段、空白裁剪和错误信息格式，
//! 确保配置文件中的 server 定义能稳定转换为 ACP 协议类型。

use std::path::PathBuf;

use agent_client_protocol::{EnvVariable, HttpHeader, McpServer, McpServerHttp, McpServerStdio};
use serde_json::json;
use vw_acp::{parse_mcp_servers, parse_mcp_servers_with_field_name, parse_optional_mcp_servers};

/// 验证 HTTP server 会裁剪用户输入空白，并保留 headers 与 `_meta`。
#[test]
fn parse_mcp_servers_parses_http_server() {
    let value = json!([
        {
            "type": "http",
            "name": "  Gateway  ",
            "url": " https://example.com/mcp ",
            "headers": [
                {
                    "name": " Authorization ",
                    "value": " Bearer token "
                }
            ],
            "_meta": {
                "region": "global"
            }
        }
    ]);

    let parsed = parse_mcp_servers(&value, "settings.json").unwrap();

    let expected = McpServerHttp::new("Gateway", "https://example.com/mcp")
        .headers(vec![HttpHeader::new("Authorization", "Bearer token")])
        .meta(
            json!({
                "region": "global"
            })
            .as_object()
            .unwrap()
            .clone(),
        );

    assert_eq!(parsed, vec![McpServer::Http(expected)]);
}

/// 验证缺省 type 仍按 stdio 解析，兼容更简短的本地 agent 配置。
#[test]
fn parse_mcp_servers_defaults_missing_type_to_stdio() {
    let value = json!([
        {
            "name": " Local Agent ",
            "command": " npx ",
            "args": ["@scope/acp", "--mode", "stdio"],
            "env": [
                {
                    "name": " API_KEY ",
                    "value": " secret "
                }
            ],
            "_meta": null
        }
    ]);

    let parsed = parse_mcp_servers(&value, "agents.json").unwrap();

    let expected = McpServerStdio::new("Local Agent", PathBuf::from("npx"))
        .args(vec!["@scope/acp".to_string(), "--mode".to_string(), "stdio".to_string()])
        .env(vec![EnvVariable::new("API_KEY", "secret")]);

    assert_eq!(parsed, vec![McpServer::Stdio(expected)]);
}

/// 验证缺失 MCP server 字段时返回 `None`，让调用方区分未配置和空列表。
#[test]
fn parse_optional_mcp_servers_returns_none_for_missing_field() {
    let parsed = parse_optional_mcp_servers(None, "config.json").unwrap();

    assert_eq!(parsed, None);
}

/// 验证不支持的 server type 会给出包含字段路径的错误，便于用户定位配置问题。
#[test]
fn parse_mcp_servers_reports_invalid_type() {
    let value = json!([
        {
            "type": "websocket",
            "name": "Bad Server",
            "url": "https://example.com"
        }
    ]);

    let error = parse_mcp_servers_with_field_name(&value, "config.json", "servers").unwrap_err();

    assert_eq!(
        error.to_string(),
        "Invalid servers[0] in config.json.type: expected http, sse, or stdio"
    );
}
