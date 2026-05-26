//! MCP 工具共享辅助模块。
//!
//! 本模块封装 MCP server 配置加载、传输连接、server 元信息序列化以及资源内容
//! 规范化输出逻辑。上层工具通过这些函数保持一致的连接生命周期与 JSON 返回结构。

use agent_client_protocol::McpServer;
use rmcp::model::{ClientInfo, ResourceContents};
use rmcp::service::{Peer, RoleClient, ServiceExt};
use rmcp::transport::{ConfigureCommandExt, SseClientTransport, StreamableHttpClientTransport, TokioChildProcess};
use serde_json::{Value, json};
use tokio::process::Command;

struct NoopClient {
    info: ClientInfo,
}

impl Default for NoopClient {
    fn default() -> Self {
        Self {
            info: ClientInfo {
                protocol_version: rmcp::model::ProtocolVersion::LATEST,
                capabilities: Default::default(),
                client_info: Default::default(),
            },
        }
    }
}

impl rmcp::ClientHandler for NoopClient {
    fn get_info(&self) -> ClientInfo {
        self.info.clone()
    }
}

/// 加载当前工作目录下解析完成的 ACP 配置。
///
/// # 返回值
///
/// 成功时返回已合并默认值和本地配置的 `ResolvedAcpxConfig`。
///
/// # 错误
///
/// 当当前目录不可读取，或 ACP 配置加载、解析失败时返回错误。
pub async fn load_resolved_acp_config() -> anyhow::Result<vw_acp::ResolvedAcpxConfig> {
    vw_acp::load_resolved_config(std::env::current_dir()?).await.map_err(anyhow::Error::from)
}

/// 为指定 MCP server 建立客户端 peer，并在 peer 生命周期内执行回调。
///
/// # 参数
///
/// - `server`: 已解析配置中的 MCP server 定义。
/// - `f`: 接收活动 peer 的异步回调，所有 MCP 请求应在该回调内完成。
///
/// # 返回值
///
/// 返回回调的执行结果。
///
/// # 错误
///
/// 当传输不受支持、进程启动失败、网络连接失败、MCP 握手失败或回调自身失败时
/// 返回错误。
pub async fn with_server_peer<T>(
    server: &McpServer,
    f: impl for<'a> FnOnce(
        &'a Peer<RoleClient>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<T>> + Send + 'a>,
    >,
) -> anyhow::Result<T> {
    match server {
        McpServer::Stdio(server) => {
            let command = Command::new(&server.command);
            let args = server.args.clone();
            let env = server
                .env
                .iter()
                .map(|env| (env.name.clone(), env.value.clone()))
                .collect::<Vec<_>>();
            let transport = TokioChildProcess::new(command.configure(move |cmd| {
                cmd.args(args);
                for (key, value) in &env {
                    cmd.env(key, value);
                }
            }))?;
            let running = NoopClient::default().serve(transport).await?;
            let result = f(running.peer()).await;
            // 回调结束后主动取消运行中的 client，避免 stdio 子进程或网络连接泄漏到工具调用外。
            let _ = running.cancel().await;
            result
        }
        McpServer::Sse(server) => {
            let transport = SseClientTransport::start(server.url.clone()).await?;
            let running = NoopClient::default().serve(transport).await?;
            let result = f(running.peer()).await;
            // SSE 连接是长连接；这里显式取消，保证一次工具调用只占用一次会话资源。
            let _ = running.cancel().await;
            result
        }
        McpServer::Http(server) => {
            let transport = StreamableHttpClientTransport::from_uri(server.url.clone());
            let running = NoopClient::default().serve(transport).await?;
            let result = f(running.peer()).await;
            // Streamable HTTP 同样由 rmcp service 持有运行状态，回调后释放更容易定位连接边界。
            let _ = running.cancel().await;
            result
        }
        _ => anyhow::bail!("unsupported MCP server transport"),
    }
}

/// 返回 MCP server 的稳定名称。
///
/// # 参数
///
/// - `server`: MCP server 配置枚举。
///
/// # 返回值
///
/// 对受支持传输返回配置中的名称；未知传输返回 `"unknown"`，用于诊断输出。
pub fn server_name(server: &McpServer) -> &str {
    match server {
        McpServer::Http(server) => &server.name,
        McpServer::Sse(server) => &server.name,
        McpServer::Stdio(server) => &server.name,
        _ => "unknown",
    }
}

/// 将 MCP server 的关键连接信息转换为可展示的 JSON 摘要。
///
/// # 参数
///
/// - `server`: MCP server 配置枚举。
///
/// # 返回值
///
/// 返回包含名称、传输类型及非敏感连接字段的 JSON 值。stdio 环境变量不会写入摘要，
/// 避免把潜在敏感配置暴露给工具输出。
pub fn server_summary(server: &McpServer) -> Value {
    match server {
        McpServer::Http(server) => json!({
            "name": server.name,
            "transport": "http",
            "url": server.url,
        }),
        McpServer::Sse(server) => json!({
            "name": server.name,
            "transport": "sse",
            "url": server.url,
        }),
        McpServer::Stdio(server) => json!({
            "name": server.name,
            "transport": "stdio",
            "command": server.command,
            "args": server.args,
        }),
        _ => json!({
            "name": "unknown",
            "transport": "unknown",
        }),
    }
}

/// 将 MCP 资源内容转换为工具输出使用的 JSON 数组。
///
/// # 参数
///
/// - `contents`: MCP `read_resource` 返回的资源内容列表。
///
/// # 返回值
///
/// 每个资源会保留 URI、MIME 类型、内容载荷以及 `kind` 字段，方便调用端区分文本与
/// 二进制资源。
pub fn resource_contents_json(contents: &[ResourceContents]) -> Value {
    Value::Array(
        contents
            .iter()
            .map(|content| match content {
                ResourceContents::TextResourceContents { uri, mime_type, text, .. } => {
                    json!({
                        "uri": uri,
                        "mime_type": mime_type,
                        "text": text,
                        "kind": "text",
                    })
                }
                ResourceContents::BlobResourceContents { uri, mime_type, blob, .. } => {
                    json!({
                        "uri": uri,
                        "mime_type": mime_type,
                        "blob": blob,
                        "kind": "blob",
                    })
                }
            })
            .collect(),
    )
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
