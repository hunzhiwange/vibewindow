use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 网关服务配置（`[gateway]` 配置段）。
///
/// 用于控制 webhook 与配对端点所使用的 HTTP 网关。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GatewayConfig {
    /// 网关监听端口，默认值为 `42617`。
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    /// 网关监听主机，默认值为 `127.0.0.1`。
    #[serde(default = "default_gateway_host")]
    pub host: String,
    /// 是否在接受请求前要求先完成配对，默认值为 `true`。
    #[serde(default = "default_true")]
    pub require_pairing: bool,
    /// 是否允许在未使用隧道时绑定到非 localhost 地址，默认值为 `false`。
    #[serde(default)]
    pub allow_public_bind: bool,
    /// 已配对的 bearer token 列表，由系统自动管理，不建议手工编辑。
    #[serde(default)]
    pub paired_tokens: Vec<String>,

    /// 每个客户端键每分钟允许的 `/pair` 请求上限。
    #[serde(default = "default_pair_rate_limit")]
    pub pair_rate_limit_per_minute: u32,

    /// 每个客户端键每分钟允许的 `/webhook` 请求上限。
    #[serde(default = "default_webhook_rate_limit")]
    pub webhook_rate_limit_per_minute: u32,

    /// 是否信任代理转发的客户端 IP 头（`X-Forwarded-For`、`X-Real-IP`）。
    /// 默认关闭，仅应在可信反向代理之后启用。
    #[serde(default)]
    pub trust_forwarded_headers: bool,

    /// 网关限流器映射中可追踪的不同客户端键数量上限。
    #[serde(default = "default_gateway_rate_limit_max_keys")]
    pub rate_limit_max_keys: usize,

    /// Webhook 幂等键的 TTL。
    #[serde(default = "default_idempotency_ttl_secs")]
    pub idempotency_ttl_secs: u64,

    /// 内存中保留的不同幂等键数量上限。
    #[serde(default = "default_gateway_idempotency_max_keys")]
    pub idempotency_max_keys: usize,

    /// Node-control 协议脚手架配置（`[gateway.node_control]`）。
    #[serde(default)]
    pub node_control: NodeControlConfig,
}

/// `[gateway.node_control]` 下的 Node-control 脚手架配置。
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct NodeControlConfig {
    /// 是否启用实验性的 node-control API 端点。
    #[serde(default)]
    pub enabled: bool,

    /// node-control API 调用可选的额外共享 token。
    /// 设置后，客户端必须在 `X-Node-Control-Token` 中发送该值。
    #[serde(default)]
    pub auth_token: Option<String>,

    /// `node.describe` / `node.invoke` 允许访问的远端节点 ID 白名单。
    /// 为空表示“没有显式白名单”，即接受所有 ID。
    #[serde(default)]
    pub allowed_node_ids: Vec<String>,
}

fn default_gateway_port() -> u16 {
    42617
}

fn default_gateway_host() -> String {
    "127.0.0.1".into()
}

fn default_pair_rate_limit() -> u32 {
    10
}

fn default_webhook_rate_limit() -> u32 {
    60
}

fn default_idempotency_ttl_secs() -> u64 {
    300
}

fn default_gateway_rate_limit_max_keys() -> usize {
    10_000
}

fn default_gateway_idempotency_max_keys() -> usize {
    10_000
}

fn default_true() -> bool {
    true
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: default_gateway_port(),
            host: default_gateway_host(),
            require_pairing: true,
            allow_public_bind: false,
            paired_tokens: Vec::new(),
            pair_rate_limit_per_minute: default_pair_rate_limit(),
            webhook_rate_limit_per_minute: default_webhook_rate_limit(),
            trust_forwarded_headers: false,
            rate_limit_max_keys: default_gateway_rate_limit_max_keys(),
            idempotency_ttl_secs: default_idempotency_ttl_secs(),
            idempotency_max_keys: default_gateway_idempotency_max_keys(),
            node_control: NodeControlConfig::default(),
        }
    }
}

/// 对外暴露 gateway 的隧道配置（`[tunnel]` 配置段）。
///
/// 支持的提供方包括：`"none"`（默认）、`"cloudflare"`、`"tailscale"`、`"ngrok"`、`"custom"`。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TunnelConfig {
    /// 隧道提供方：`"none"`、`"cloudflare"`、`"tailscale"`、`"ngrok"` 或 `"custom"`。默认值为 `"none"`。
    pub provider: String,

    /// Cloudflare Tunnel 配置，在 `provider = "cloudflare"` 时使用。
    #[serde(default)]
    pub cloudflare: Option<CloudflareTunnelConfig>,

    /// Tailscale Funnel/Serve 配置，在 `provider = "tailscale"` 时使用。
    #[serde(default)]
    pub tailscale: Option<TailscaleTunnelConfig>,

    /// ngrok 隧道配置，在 `provider = "ngrok"` 时使用。
    #[serde(default)]
    pub ngrok: Option<NgrokTunnelConfig>,

    /// 自定义隧道命令配置，在 `provider = "custom"` 时使用。
    #[serde(default)]
    pub custom: Option<CustomTunnelConfig>,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            provider: "none".into(),
            cloudflare: None,
            tailscale: None,
            ngrok: None,
            custom: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CloudflareTunnelConfig {
    /// Cloudflare Tunnel token，来自 Zero Trust 控制台。
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TailscaleTunnelConfig {
    /// 是否使用 Tailscale Funnel（公网）而不是 Serve（仅 tailnet）。
    #[serde(default)]
    pub funnel: bool,
    /// 可选的主机名覆盖值。
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NgrokTunnelConfig {
    /// ngrok 认证 token。
    pub auth_token: String,
    /// 可选的自定义域名。
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CustomTunnelConfig {
    /// 自定义隧道集成的可选公网 URL。
    #[serde(default)]
    pub url: Option<String>,
    /// 自定义隧道集成的可选认证 token。
    #[serde(default)]
    pub auth_token: Option<String>,
    /// 启动隧道的命令模板，可使用 `{port}` 与 `{host}` 占位符。
    /// 例如：`bore local {port} --to bore.pub`
    pub start_command: String,
    /// 可选的隧道健康检查 URL。
    pub health_url: Option<String>,
    /// 可选的正则表达式，用于从命令标准输出中提取公网 URL。
    pub url_pattern: Option<String>,
}
#[cfg(test)]
#[path = "gateway_tests.rs"]
mod gateway_tests;
