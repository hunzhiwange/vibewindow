//! 工具规格与网关工具列表相关类型。
//!
//! 本模块描述代理可调用工具的元信息，主要用于：
//! - 向模型声明当前可用工具
//! - 在 UI 中展示工具名称与说明
//! - 与网关兼容层交换工具 schema

pub use crate::tools::{ListToolSpecsResponse, ToolSpecDto};
use serde::{Deserialize, Serialize};

fn default_redis_tool_schema_version() -> u32 {
    1
}

fn default_redis_tool_load_count() -> u32 {
    500
}

fn default_redis_key_pattern() -> String {
    "*".to_string()
}

fn default_redis_ssh_port() -> u16 {
    22
}

fn default_redis_ssh_timeout_secs() -> u32 {
    30
}

fn default_redis_history_limit() -> usize {
    50
}

/// Redis TLS 证书配置 DTO。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisTlsCertConfig {
    /// 客户端私钥路径。
    #[serde(default)]
    pub private_key_path: String,
    /// 客户端证书路径。
    #[serde(default)]
    pub public_cert_path: String,
    /// CA 证书路径。
    #[serde(default, alias = "ca_path")]
    pub ca_cert_path: String,
}

/// Redis SSH 隧道配置 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisSshTunnelConfig {
    /// 是否启用 SSH 隧道。
    #[serde(default)]
    pub enabled: bool,
    /// SSH 主机地址。
    #[serde(default)]
    pub host: String,
    /// SSH 端口。
    #[serde(default = "default_redis_ssh_port")]
    pub port: u16,
    /// SSH 用户名。
    #[serde(default)]
    pub username: String,
    /// SSH 密码。
    #[serde(default)]
    pub password: String,
    /// SSH 私钥路径。
    #[serde(default)]
    pub private_key_path: String,
    /// SSH 私钥口令。
    #[serde(default)]
    pub passphrase: String,
    /// SSH 连接超时（秒）。
    #[serde(default = "default_redis_ssh_timeout_secs")]
    pub timeout_secs: u32,
}

impl Default for GatewayRedisSshTunnelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_redis_ssh_port(),
            username: String::new(),
            password: String::new(),
            private_key_path: String::new(),
            passphrase: String::new(),
            timeout_secs: default_redis_ssh_timeout_secs(),
        }
    }
}

/// Redis Sentinel 配置 DTO。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisSentinelConfig {
    /// 是否启用 Sentinel。
    #[serde(default)]
    pub enabled: bool,
    /// Sentinel 监控的主节点组名称。
    #[serde(default)]
    pub master_name: String,
    /// Redis 节点密码。
    #[serde(default)]
    pub node_password: String,
}

/// Redis 连接配置 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisConnectionConfig {
    /// 连接唯一标识。
    pub id: String,
    /// 连接名称。
    pub name: String,
    /// 主机地址。
    pub host: String,
    /// 端口。
    pub port: u16,
    /// 数据库编号。
    pub db: i64,
    /// 用户名。
    #[serde(default)]
    pub username: String,
    /// 密码。
    #[serde(default)]
    pub password: String,
    /// 是否启用 TLS。
    #[serde(default)]
    pub use_tls: bool,
    /// TLS 证书材料配置。
    #[serde(default)]
    pub tls_cert: GatewayRedisTlsCertConfig,
    /// SSH 隧道配置。
    #[serde(default)]
    pub ssh_tunnel: GatewayRedisSshTunnelConfig,
    /// Sentinel 配置。
    #[serde(default)]
    pub sentinel: GatewayRedisSentinelConfig,
    /// 是否启用 Cluster 模式。
    #[serde(default)]
    pub use_cluster: bool,
    /// 是否只读访问。
    #[serde(default)]
    pub read_only: bool,
    /// 键匹配模式。
    #[serde(default = "default_redis_key_pattern")]
    pub key_pattern: String,
    /// 最近一次进入时间戳。
    #[serde(default)]
    pub last_used_ms: Option<u64>,
    /// 最近一次更新时间戳。
    #[serde(default)]
    pub updated_at_ms: u64,
}

/// Redis 历史记录 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisHistoryRecord {
    /// 记录时间戳（毫秒）。
    pub time_ms: u64,
    /// 关联连接 ID。
    #[serde(default)]
    pub connection_id: Option<String>,
    /// 显示用连接名称。
    pub connection_label: String,
    /// 操作或命令名称。
    pub command: String,
    /// 参数摘要。
    pub args: String,
    /// 耗时（毫秒）。
    #[serde(default)]
    pub cost_ms: u64,
    /// 是否写操作。
    #[serde(default)]
    pub is_write: bool,
}

/// Redis 设置 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisSettings {
    /// 结构版本。
    #[serde(default = "default_redis_tool_schema_version")]
    pub schema_version: u32,
    /// 默认加载数量。
    #[serde(default = "default_redis_tool_load_count")]
    pub default_load_count: u32,
    /// 上次选中的连接。
    #[serde(default)]
    pub selected_connection_id: Option<String>,
}

impl Default for GatewayRedisSettings {
    fn default() -> Self {
        Self {
            schema_version: default_redis_tool_schema_version(),
            default_load_count: default_redis_tool_load_count(),
            selected_connection_id: None,
        }
    }
}

/// Redis 设置更新请求体。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisSettingsUpdateBody {
    /// 默认加载数量。
    pub default_load_count: u32,
}

/// Redis 连接写入请求体。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisConnectionUpsertBody {
    /// 连接名称。
    pub name: String,
    /// 主机地址。
    pub host: String,
    /// 端口。
    pub port: u16,
    /// 数据库编号。
    pub db: i64,
    /// 用户名。
    #[serde(default)]
    pub username: String,
    /// 密码。
    #[serde(default)]
    pub password: String,
    /// 是否启用 TLS。
    #[serde(default)]
    pub use_tls: bool,
    /// TLS 证书材料配置。
    #[serde(default)]
    pub tls_cert: GatewayRedisTlsCertConfig,
    /// SSH 隧道配置。
    #[serde(default)]
    pub ssh_tunnel: GatewayRedisSshTunnelConfig,
    /// Sentinel 配置。
    #[serde(default)]
    pub sentinel: GatewayRedisSentinelConfig,
    /// 是否启用 Cluster 模式。
    #[serde(default)]
    pub use_cluster: bool,
    /// 是否只读访问。
    #[serde(default)]
    pub read_only: bool,
    /// 键匹配模式。
    #[serde(default = "default_redis_key_pattern")]
    pub key_pattern: String,
}

/// Redis 历史分页查询参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisHistoryListQuery {
    /// 起始偏移。
    #[serde(default)]
    pub offset: Option<usize>,
    /// 单页数量。
    #[serde(default)]
    pub limit: Option<usize>,
    /// 连接 ID 过滤。
    #[serde(default)]
    pub connection_id: Option<String>,
    /// 关键字过滤。
    #[serde(default)]
    pub query: Option<String>,
    /// 是否仅返回写操作。
    #[serde(default)]
    pub only_write: Option<bool>,
}

/// Redis 历史分页响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisHistoryPage {
    /// 当前页记录。
    #[serde(default)]
    pub items: Vec<GatewayRedisHistoryRecord>,
    /// 当前偏移。
    #[serde(default)]
    pub offset: usize,
    /// 当前页大小。
    #[serde(default = "default_redis_history_limit")]
    pub limit: usize,
    /// 过滤后的总量。
    #[serde(default)]
    pub total: usize,
    /// 是否仍有下一页。
    #[serde(default)]
    pub has_more: bool,
}

impl Default for GatewayRedisHistoryPage {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            offset: 0,
            limit: default_redis_history_limit(),
            total: 0,
            has_more: false,
        }
    }
}

/// Redis 连接测试响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisConnectionTestResponse {
    /// 连接是否成功。
    #[serde(default)]
    pub ok: bool,
    /// 结果说明。
    pub message: String,
    /// 测试耗时（毫秒）。
    #[serde(default)]
    pub latency_ms: u64,
}

/// Redis INFO 键值条目。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisInfoEntry {
    /// INFO 字段名。
    #[serde(default)]
    pub key: String,
    /// INFO 字段值。
    #[serde(default)]
    pub value: String,
}

/// Redis Keyspace 统计条目。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisKeyspaceStat {
    /// 数据库名称，例如 db0。
    #[serde(default)]
    pub db: String,
    /// 键数量。
    #[serde(default)]
    pub keys: u64,
    /// 过期键数量。
    #[serde(default)]
    pub expires: u64,
    /// 平均 TTL。
    #[serde(default)]
    pub avg_ttl: i64,
}

/// Redis 运行时概览。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisRuntimeOverview {
    /// 连接 ID。
    #[serde(default)]
    pub connection_id: String,
    /// 连接显示名称。
    #[serde(default)]
    pub connection_label: String,
    /// Redis 版本。
    #[serde(default)]
    pub server_version: String,
    /// 操作系统。
    #[serde(default)]
    pub os: String,
    /// 进程 ID。
    #[serde(default)]
    pub process_id: String,
    /// 已用内存。
    #[serde(default)]
    pub used_memory_human: String,
    /// 内存峰值。
    #[serde(default)]
    pub used_memory_peak_human: String,
    /// Lua 占用内存。
    #[serde(default)]
    pub used_memory_lua_human: String,
    /// 当前客户端连接数。
    #[serde(default)]
    pub connected_clients: u64,
    /// 历史连接数。
    #[serde(default)]
    pub total_connections_received: u64,
    /// 历史命令数。
    #[serde(default)]
    pub total_commands_processed: u64,
    /// Keyspace 统计。
    #[serde(default)]
    pub keyspace: Vec<GatewayRedisKeyspaceStat>,
    /// INFO 全量键值。
    #[serde(default)]
    pub info_entries: Vec<GatewayRedisInfoEntry>,
}

/// Redis 键分页查询参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisKeyListQuery {
    /// SCAN 游标。
    #[serde(default)]
    pub cursor: Option<u64>,
    /// 单页数量。
    #[serde(default)]
    pub count: Option<u32>,
    /// 匹配模式。
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Redis 键分页响应。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisKeyPage {
    /// 连接 ID。
    #[serde(default)]
    pub connection_id: String,
    /// 本次使用的匹配模式。
    #[serde(default)]
    pub pattern: String,
    /// 当前批次键列表。
    #[serde(default)]
    pub keys: Vec<String>,
    /// 下一次 SCAN 游标。
    #[serde(default)]
    pub next_cursor: u64,
    /// 是否仍有后续数据。
    #[serde(default)]
    pub has_more: bool,
}

/// Redis Key 分析请求。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisKeyAnalysisRequest {
    /// 目标 Key 名称。
    #[serde(default)]
    pub key: String,
}

/// Redis Key 默认创建请求。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisKeyCreateRequest {
    /// 目标 Key 名称。
    #[serde(default)]
    pub key: String,
    /// 期望的数据类型，例如 String / Hash / List。
    #[serde(default)]
    pub key_type: String,
}

/// Redis Key 内容分析结果。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisKeyAnalysis {
    /// 连接 ID。
    #[serde(default)]
    pub connection_id: String,
    /// Key 名称。
    #[serde(default)]
    pub key: String,
    /// 数据类型标签。
    #[serde(default)]
    pub key_type: String,
    /// TTL 秒数，`-1` 表示永久，`-2` 表示不存在。
    #[serde(default)]
    pub ttl_secs: i64,
    /// 内存占用（字节）。
    #[serde(default)]
    pub memory_usage_bytes: Option<u64>,
    /// 用于值预览的命令。
    #[serde(default)]
    pub preview_command: String,
    /// 格式化后的值预览。
    #[serde(default)]
    pub preview_output: String,
}

/// Redis 命令执行请求。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisCommandRequest {
    /// 原始命令行。
    #[serde(default)]
    pub command: String,
}

/// Redis 命令执行响应。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisCommandResponse {
    /// 原始命令行。
    #[serde(default)]
    pub command: String,
    /// 格式化后的命令输出。
    #[serde(default)]
    pub output: String,
    /// 执行耗时（毫秒）。
    #[serde(default)]
    pub cost_ms: u64,
    /// 是否为错误输出。
    #[serde(default)]
    pub is_error: bool,
}

/// Redis 配置导入导出载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisConfigBundle {
    /// 结构版本。
    #[serde(default = "default_redis_tool_schema_version")]
    pub schema_version: u32,
    /// 默认加载数量。
    #[serde(default = "default_redis_tool_load_count")]
    pub default_load_count: u32,
    /// 连接列表。
    #[serde(default)]
    pub connections: Vec<GatewayRedisConnectionConfig>,
}

impl Default for GatewayRedisConfigBundle {
    fn default() -> Self {
        Self {
            schema_version: default_redis_tool_schema_version(),
            default_load_count: default_redis_tool_load_count(),
            connections: Vec::new(),
        }
    }
}

/// Redis 导入完成响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisImportResponse {
    /// 导入连接数。
    #[serde(default)]
    pub imported_count: usize,
    /// 导入后默认加载数量。
    #[serde(default = "default_redis_tool_load_count")]
    pub default_load_count: u32,
    /// 导入后默认选中的连接。
    #[serde(default)]
    pub selected_connection_id: Option<String>,
}

/// Redis 删除连接响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisDeleteResponse {
    /// 已删除的连接 ID。
    pub deleted_id: String,
}

/// Redis 工具状态 DTO。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayRedisToolState {
    /// 结构版本。
    #[serde(default = "default_redis_tool_schema_version")]
    pub schema_version: u32,
    /// 默认加载数量。
    #[serde(default = "default_redis_tool_load_count")]
    pub default_load_count: u32,
    /// 连接列表。
    #[serde(default)]
    pub connections: Vec<GatewayRedisConnectionConfig>,
    /// 历史记录。
    #[serde(default)]
    pub history: Vec<GatewayRedisHistoryRecord>,
    /// 上次选中的连接。
    #[serde(default)]
    pub selected_connection_id: Option<String>,
}

impl Default for GatewayRedisToolState {
    fn default() -> Self {
        Self {
            schema_version: default_redis_tool_schema_version(),
            default_load_count: default_redis_tool_load_count(),
            connections: Vec::new(),
            history: Vec::new(),
            selected_connection_id: None,
        }
    }
}
