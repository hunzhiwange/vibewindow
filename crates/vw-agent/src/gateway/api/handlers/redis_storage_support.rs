//! Redis 工具的持久化辅助函数。
//!
//! 本模块封装 Redis 工具设置、连接列表和操作历史的 storage key，
//! 让上层 handler 只关心领域对象，不直接散落存储路径和历史裁剪策略。

use vw_api_types::tool::{
    GatewayRedisConnectionConfig, GatewayRedisHistoryRecord, GatewayRedisSettings,
};

use crate::app::agent::gateway::ApiError;
use crate::storage;

/// Redis 操作历史最多保留的记录数。
pub(super) const REDIS_HISTORY_LIMIT: usize = 200;
/// 历史分页默认条数。
pub(super) const REDIS_HISTORY_PAGE_LIMIT: usize = 50;
/// 历史分页允许的最大条数。
pub(super) const REDIS_HISTORY_PAGE_LIMIT_MAX: usize = 200;

/// 读取 Redis 工具全局设置。
///
/// # 返回值
///
/// 返回已保存设置；缺失或读取失败时返回默认设置。
///
/// # 错误处理
///
/// 读取失败被降级为默认值，避免配置损坏阻断桌面 UI 打开。
pub(super) async fn load_settings() -> GatewayRedisSettings {
    storage::read(&["redis", "settings"]).await.unwrap_or_default()
}

/// 保存 Redis 工具全局设置。
///
/// # 参数
///
/// * `settings` - 要持久化的设置。
///
/// # 错误处理
///
/// storage 写入失败时返回 [`ApiError::internal`]。
pub(super) async fn save_settings(settings: &GatewayRedisSettings) -> Result<(), ApiError> {
    storage::write(&["redis", "settings"], settings)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// 读取已保存的 Redis 连接列表。
///
/// # 返回值
///
/// 返回连接配置列表；缺失或读取失败时返回空列表。
pub(super) async fn load_connections() -> Vec<GatewayRedisConnectionConfig> {
    storage::read(&["redis", "connections"]).await.unwrap_or_default()
}

/// 保存 Redis 连接列表。
///
/// # 参数
///
/// * `connections` - 要持久化的连接配置列表。
///
/// # 错误处理
///
/// storage 写入失败时返回 [`ApiError::internal`]。
pub(super) async fn save_connections(
    connections: &[GatewayRedisConnectionConfig],
) -> Result<(), ApiError> {
    let owned = connections.to_vec();
    storage::write(&["redis", "connections"], &owned)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// 读取 Redis 操作历史。
///
/// # 返回值
///
/// 返回历史记录；缺失或读取失败时返回空列表。
pub(super) async fn load_history() -> Vec<GatewayRedisHistoryRecord> {
    storage::read(&["redis", "history"]).await.unwrap_or_default()
}

/// 保存 Redis 操作历史。
///
/// # 参数
///
/// * `records` - 要持久化的历史记录。
///
/// # 错误处理
///
/// storage 写入失败时返回 [`ApiError::internal`]。
pub(super) async fn save_history(records: &[GatewayRedisHistoryRecord]) -> Result<(), ApiError> {
    let owned = records.to_vec();
    storage::write(&["redis", "history"], &owned)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// 追加 Redis 操作历史，失败时仅记录日志。
///
/// # 参数
///
/// * `record` - 新的历史记录。
///
/// # 错误处理
///
/// 历史保存失败不会影响主操作结果，只写入 warning 日志。
pub(super) async fn append_history_best_effort(record: GatewayRedisHistoryRecord) {
    let mut records = load_history().await;
    records.insert(0, record);
    if records.len() > REDIS_HISTORY_LIMIT {
        records.truncate(REDIS_HISTORY_LIMIT);
    }

    if let Err(error) = save_history(&records).await {
        tracing::warn!(target: "vw_gateway", error = %error, "failed to save redis history");
    }
}

/// 根据 id 加载 Redis 连接。
///
/// # 参数
///
/// * `connection_id` - 连接 id。
///
/// # 返回值
///
/// 返回匹配的连接配置。
///
/// # 错误处理
///
/// 连接不存在时返回 404 语义的 [`ApiError`]。
pub(super) async fn load_connection_by_id(
    connection_id: &str,
) -> Result<GatewayRedisConnectionConfig, ApiError> {
    load_connections()
        .await
        .into_iter()
        .find(|item| item.id == connection_id)
        .ok_or_else(|| ApiError::not_found("Redis 连接不存在"))
}

/// 按最近使用时间和名称排序连接列表。
///
/// # 参数
///
/// * `connections` - 待原地排序的连接列表。
pub(super) fn sort_connections(connections: &mut [GatewayRedisConnectionConfig]) {
    connections.sort_by(|left, right| {
        let left_key = left.last_used_ms.unwrap_or(left.updated_at_ms);
        let right_key = right.last_used_ms.unwrap_or(right.updated_at_ms);
        right_key.cmp(&left_key).then_with(|| left.name.cmp(&right.name))
    });
}

/// 构造一条 Redis 操作历史记录。
///
/// # 参数
///
/// * `connection` - 可选连接上下文；为空表示全局配置操作。
/// * `command` - 操作或 Redis 命令名称。
/// * `args` - 已脱敏/压缩后的参数摘要。
/// * `cost_ms` - 操作耗时。
/// * `is_write` - 是否为写操作。
///
/// # 返回值
///
/// 返回带当前时间戳的历史记录。
pub(super) fn history_record(
    connection: Option<&GatewayRedisConnectionConfig>,
    command: &str,
    args: String,
    cost_ms: u64,
    is_write: bool,
) -> GatewayRedisHistoryRecord {
    GatewayRedisHistoryRecord {
        time_ms: now_ms(),
        connection_id: connection.map(|item| item.id.clone()),
        connection_label: connection
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "全局配置".to_string()),
        command: command.to_string(),
        args,
        cost_ms,
        is_write,
    }
}

/// 生成连接配置的简短历史摘要。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接。
///
/// # 返回值
///
/// 返回不包含密码、证书内容等敏感信息的摘要字符串。
pub(super) fn compact_connection_args(connection: &GatewayRedisConnectionConfig) -> String {
    let mut modes = Vec::new();
    if connection.use_tls {
        modes.push("tls");
    }
    if connection.ssh_tunnel.enabled {
        modes.push("ssh");
    }
    if connection.sentinel.enabled {
        modes.push("sentinel");
    }
    if connection.use_cluster {
        modes.push("cluster");
    }
    if connection.read_only {
        modes.push("readonly");
    }

    let mode = if modes.is_empty() { "direct".to_string() } else { modes.join("+") };

    format!(
        "{}:{} db={} pattern={} mode={}",
        connection.host, connection.port, connection.db, connection.key_pattern, mode
    )
}

/// 获取当前 Unix 毫秒时间戳。
///
/// # 返回值
///
/// 返回当前毫秒时间；系统时间异常或转换溢出时使用安全默认值。
pub(super) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
#[path = "redis_storage_support_tests.rs"]
mod redis_storage_support_tests;
