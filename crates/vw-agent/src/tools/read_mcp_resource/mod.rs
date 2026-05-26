//! MCP 资源读取工具。
//!
//! 本模块根据用户提供的 server 名称和资源 URI，连接已配置的 MCP server 并读取真实
//! 资源内容。输出会包含 server 摘要与标准化后的资源内容，便于 UI 和模型一致消费。

use super::mcp_common::{
    load_resolved_acp_config, resource_contents_json, server_name, server_summary, with_server_peer,
};
use super::traits::{Tool, ToolResult, ToolSpec};
use async_trait::async_trait;
use rmcp::model::ReadResourceRequestParam;
use rmcp::service::Peer;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct Args {
    server: String,
    uri: String,
}

/// 读取指定 MCP 资源内容的工具。
///
/// 该工具是只读工具，不修改 server 状态；不支持空 server 名称或空 URI。
pub struct ReadMcpResourceTool;

impl ReadMcpResourceTool {
    /// 创建 MCP 资源读取工具。
    ///
    /// # 返回值
    ///
    /// 返回无状态工具实例。
    pub fn new() -> Self {
        Self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ReadMcpResourceTool {
    fn name(&self) -> &str {
        "ReadMcpResource"
    }

    fn description(&self) -> &str {
        "读取特定 MCP 资源的真实内容。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "server": {
                    "type": "string",
                    "description": "MCP server 名称。"
                },
                "uri": {
                    "type": "string",
                    "description": "资源 URI。"
                }
            },
            "required": ["server", "uri"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("ReadMcpResource")
            .with_aliases(vec!["read_mcp_resource".to_string()])
            .with_read_only(true)
            .with_destructive(false)
            .with_concurrency_safe(false)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)?;
        let server_name_filter = args.server.trim();
        let uri = args.uri.trim();
        if server_name_filter.is_empty() || uri.is_empty() {
            // 在加载配置前拒绝空输入，让调用错误保持局部且不触发不必要的外部配置读取。
            anyhow::bail!("'server' and 'uri' must not be empty");
        }

        let config = load_resolved_acp_config().await?;
        let server = config
            .mcp_servers
            .iter()
            .find(|server| server_name(server) == server_name_filter)
            .ok_or_else(|| anyhow::anyhow!("unknown MCP server '{}'", server_name_filter))?;

        let result = with_server_peer(server, |peer: &Peer<rmcp::service::RoleClient>| {
            let uri = uri.to_string();
            Box::pin(async move {
                let result = peer.read_resource(ReadResourceRequestParam { uri }).await?;
                Ok(result)
            })
        })
        .await?;

        let data = json!({
            "server": server_summary(server),
            "uri": args.uri,
            "contents": resource_contents_json(&result.contents),
        });
        Ok(ToolResult { success: true, output: serde_json::to_string_pretty(&data)?, error: None })
    }
}
#[cfg(test)]
mod tests;
