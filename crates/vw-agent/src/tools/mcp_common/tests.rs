//! MCP 工具共享逻辑测试。
//!
//! 这些测试覆盖 server 摘要、连接错误传播、资源读取输入校验以及 MCP 工具别名，
//! 确保多个 MCP 工具依赖的公共行为保持一致。

use super::super::*;
use super::*;
use agent_client_protocol::{McpServer, McpServerHttp, McpServerSse, McpServerStdio};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn server_helpers_preserve_transport_details() {
    let http = McpServer::Http(McpServerHttp::new("gateway", "https://example.com/mcp"));
    assert_eq!(server_name(&http), "gateway");
    assert_eq!(
        server_summary(&http),
        json!({
            "name": "gateway",
            "transport": "http",
            "url": "https://example.com/mcp"
        })
    );

    let sse = McpServer::Sse(McpServerSse::new("events", "https://example.com/sse"));
    assert_eq!(server_name(&sse), "events");
    assert_eq!(
        server_summary(&sse),
        json!({
            "name": "events",
            "transport": "sse",
            "url": "https://example.com/sse"
        })
    );

    let stdio = McpServer::Stdio(
        McpServerStdio::new("local", PathBuf::from("npx")).args(vec!["mcp".into()]),
    );
    assert_eq!(server_name(&stdio), "local");
    assert_eq!(
        server_summary(&stdio),
        json!({
            "name": "local",
            "transport": "stdio",
            "command": "npx",
            "args": ["mcp"]
        })
    );
}

#[tokio::test]
async fn with_server_peer_stdio_surfaces_spawn_errors() {
    let server =
        McpServer::Stdio(McpServerStdio::new("broken", PathBuf::from("vw-missing-mcp-command")));

    let error = with_server_peer(&server, |_peer| Box::pin(async move { Ok(()) }))
        .await
        .expect_err("missing stdio command should fail");
    assert!(!error.to_string().is_empty());
}

#[tokio::test]
async fn read_mcp_resource_rejects_blank_inputs_before_loading_config() {
    let tool = ReadMcpResourceTool::new();
    let error = tool
        .execute(json!({
            "server": "   ",
            "uri": ""
        }))
        .await
        .expect_err("blank inputs should be rejected");
    assert!(error.to_string().contains("must not be empty"));
}

#[test]
fn mcp_tools_expose_expected_aliases() {
    let list_spec = ListMcpResourcesTool::new().spec();
    assert!(list_spec.aliases.iter().any(|alias| alias == "list_mcp_resources"));

    let read_spec = ReadMcpResourceTool::new().spec();
    assert!(read_spec.aliases.iter().any(|alias| alias == "read_mcp_resource"));

    let auth_spec = McpAuthTool::new().spec();
    assert!(auth_spec.aliases.iter().any(|alias| alias == "mcp_auth"));
}
