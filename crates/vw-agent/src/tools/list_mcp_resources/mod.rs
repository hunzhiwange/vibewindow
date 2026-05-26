//! 列出 MCP 服务器资源的工具实现。
//!
//! 工具读取当前解析后的 ACP 配置，并只通过真实 MCP peer 查询资源。这样返回值
//! 反映当前会话可连通的资源，而不是静态配置中的推测信息。

use super::mcp_common::{load_resolved_acp_config, server_name, server_summary, with_server_peer};
use super::traits::{Tool, ToolResult, ToolSpec};
use async_trait::async_trait;
use rmcp::service::Peer;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct Args {
    #[serde(default)]
    server: Option<String>,
}

/// 查询 MCP 资源列表的只读工具。
///
/// 可选 `server` 参数用于限制到单个已配置服务器。连接、协议或序列化失败会通过
/// `ToolResult` 外层的 `anyhow::Result` 返回。
pub struct ListMcpResourcesTool;

impl ListMcpResourcesTool {
    /// 创建 MCP 资源列表工具。
    ///
    /// 返回无状态工具实例；配置加载和 MCP 连接在执行阶段完成。
    pub fn new() -> Self {
        Self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ListMcpResourcesTool {
    fn name(&self) -> &str {
        "ListMcpResources"
    }

    fn description(&self) -> &str {
        "列出当前会话可真实连通的 MCP 服务器资源。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "server": {
                    "type": "string",
                    "description": "可选的 MCP server 名称；未提供时遍历全部已配置 server。"
                }
            }
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("ListMcpResources")
            .with_aliases(vec!["list_mcp_resources".to_string()])
            .with_read_only(true)
            .with_destructive(false)
            .with_concurrency_safe(false)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)?;
        let config = load_resolved_acp_config().await?;
        let server_filter = args.server.as_deref().map(str::trim).filter(|value| !value.is_empty());
        let servers = config
            .mcp_servers
            .iter()
            .filter(|server| server_filter.is_none_or(|filter| server_name(server) == filter))
            .collect::<Vec<_>>();

        let mut items = Vec::new();
        for server in servers {
            let summary = server_summary(server);
            // 逐个通过 peer 查询，避免把无法连通的 server 误报为可用资源。
            let resources = with_server_peer(server, |peer: &Peer<rmcp::service::RoleClient>| {
                Box::pin(async move {
                    let resources = peer.list_all_resources().await?;
                    Ok(resources)
                })
            })
            .await?;
            items.push(json!({
                "server": summary,
                "resources": resources,
            }));
        }

        Ok(ToolResult { success: true, output: serde_json::to_string_pretty(&items)?, error: None })
    }
}
#[cfg(test)]
mod tests;
