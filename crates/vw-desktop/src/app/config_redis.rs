//! 通过网关同步 Redis 工具的连接、历史和运行状态。
//! 本模块只做桌面状态与网关 DTO 的转换，避免 UI 直接依赖传输层结构。

use crate::app::state::{
    RedisCommandOutputEntry, RedisConnectionConfig, RedisHistoryRecord, RedisInfoEntry,
    RedisKeyAnalysis, RedisKeyPage, RedisKeyspaceStat, RedisRuntimeOverview,
    RedisToolPersistedState,
};
use vw_gateway_client::vw_api_types::tool::{
    GatewayRedisCommandRequest, GatewayRedisCommandResponse, GatewayRedisConfigBundle,
    GatewayRedisConnectionConfig, GatewayRedisConnectionTestResponse,
    GatewayRedisConnectionUpsertBody, GatewayRedisDeleteResponse, GatewayRedisHistoryListQuery,
    GatewayRedisHistoryPage, GatewayRedisHistoryRecord, GatewayRedisImportResponse,
    GatewayRedisInfoEntry, GatewayRedisKeyAnalysis as GatewayRedisKeyAnalysisDto,
    GatewayRedisKeyAnalysisRequest, GatewayRedisKeyCreateRequest, GatewayRedisKeyListQuery,
    GatewayRedisKeyPage,
    GatewayRedisKeyspaceStat, GatewayRedisRuntimeOverview, GatewayRedisSettings,
    GatewayRedisSettingsUpdateBody,
};
use super::gateway::gateway_client;
#[cfg(not(target_arch = "wasm32"))]
use super::gateway::run_gateway_call;

pub(crate) const REDIS_HISTORY_PAGE_SIZE: usize = 50;

/// 公开结构体，承载 RedisToolGatewaySnapshot 对应的状态数据。
/// 字段保持与相邻业务流程和序列化格式一致。
#[derive(Debug, Clone)]
pub struct RedisToolGatewaySnapshot {
    pub persisted_state: RedisToolPersistedState,
    pub history_offset: usize,
    pub history_limit: usize,
    pub history_total: usize,
    pub history_has_more: bool,
}

/// 公开函数，执行 load_redis_tool_state 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_redis_tool_state() -> RedisToolPersistedState {
    let result = run_gateway_call(async {
        load_redis_tool_snapshot_async(default_redis_history_query()).await
    });
    match result {
        Ok(snapshot) => snapshot.persisted_state,
        Err(err) => {
            tracing::warn!(target: "vw_desktop", error = %err, "failed to load redis tool state via gateway");
            RedisToolPersistedState::default()
        }
    }
}

/// 公开函数，执行 load_redis_tool_state 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(target_arch = "wasm32")]
pub fn load_redis_tool_state() -> RedisToolPersistedState {
    RedisToolPersistedState::default()
}

/// 公开函数，执行 load_redis_tool_state_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn load_redis_tool_state_async() -> Result<RedisToolPersistedState, String> {
    Ok(load_redis_tool_snapshot_async(default_redis_history_query())
        .await?
        .persisted_state)
}

/// 公开函数，执行 load_redis_tool_snapshot_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn load_redis_tool_snapshot_async(
    query: GatewayRedisHistoryListQuery,
) -> Result<RedisToolGatewaySnapshot, String> {
    let client = gateway_client()?;
    let settings = client.redis_settings_get().await?;
    let connections = client.redis_connections_list().await?;
    let history_page = client.redis_history_list(&query).await?;
    Ok(redis_snapshot_from_gateway(settings, connections, history_page))
}

/// 公开函数，执行 redis_settings_update_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_settings_update_async(
    default_load_count: u32,
) -> Result<GatewayRedisSettings, String> {
    let client = gateway_client()?;
    client
        .redis_settings_put(&GatewayRedisSettingsUpdateBody {
            default_load_count: default_load_count.clamp(1, 10_000),
        })
        .await
}

/// 公开函数，执行 redis_connection_create_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_create_async(
    body: &GatewayRedisConnectionUpsertBody,
) -> Result<RedisConnectionConfig, String> {
    let client = gateway_client()?;
    let connection = client.redis_connection_create(body).await?;
    Ok(redis_connection_from_gateway(connection))
}

/// 公开函数，执行 redis_connection_update_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_update_async(
    connection_id: &str,
    body: &GatewayRedisConnectionUpsertBody,
) -> Result<RedisConnectionConfig, String> {
    let client = gateway_client()?;
    let connection = client.redis_connection_update(connection_id, body).await?;
    Ok(redis_connection_from_gateway(connection))
}

/// 公开函数，执行 redis_connection_delete_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_delete_async(
    connection_id: &str,
) -> Result<GatewayRedisDeleteResponse, String> {
    let client = gateway_client()?;
    client.redis_connection_delete(connection_id).await
}

/// 公开函数，执行 redis_connection_activate_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_activate_async(
    connection_id: &str,
) -> Result<GatewayRedisSettings, String> {
    let client = gateway_client()?;
    client.redis_connection_activate(connection_id).await
}

/// 公开函数，执行 redis_connection_test_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_test_async(
    connection_id: &str,
) -> Result<GatewayRedisConnectionTestResponse, String> {
    let client = gateway_client()?;
    client.redis_connection_test(connection_id).await
}

/// 公开函数，执行 redis_connection_overview_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_overview_async(
    connection_id: &str,
) -> Result<RedisRuntimeOverview, String> {
    let client = gateway_client()?;
    let overview = client.redis_connection_overview(connection_id).await?;
    Ok(redis_runtime_overview_from_gateway(overview))
}

/// 公开函数，执行 redis_connection_keys_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_keys_async(
    connection_id: &str,
    pattern: &str,
    cursor: u64,
    count: u32,
) -> Result<RedisKeyPage, String> {
    let client = gateway_client()?;
    let page = client
        .redis_connection_keys(
            connection_id,
            &GatewayRedisKeyListQuery {
                cursor: Some(cursor),
                count: Some(count.clamp(1, 10_000)),
                pattern: Some(pattern.trim().to_string()),
            },
        )
        .await?;
    Ok(redis_key_page_from_gateway(page))
}

/// 公开函数，执行 redis_connection_key_analyze_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_key_analyze_async(
    connection_id: &str,
    key: &str,
) -> Result<RedisKeyAnalysis, String> {
    let client = gateway_client()?;
    let analysis = client
        .redis_connection_key_analyze(
            connection_id,
            &GatewayRedisKeyAnalysisRequest {
                key: key.trim().to_string(),
            },
        )
        .await?;
    Ok(redis_key_analysis_from_gateway(analysis))
}

/// 公开函数，执行 redis_connection_key_create_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_connection_key_create_async(
    connection_id: &str,
    key: &str,
    key_type: &str,
) -> Result<RedisKeyAnalysis, String> {
    let client = gateway_client()?;
    let analysis = client
        .redis_connection_key_create(
            connection_id,
            &GatewayRedisKeyCreateRequest {
                key: key.trim().to_string(),
                key_type: key_type.trim().to_string(),
            },
        )
        .await?;
    Ok(redis_key_analysis_from_gateway(analysis))
}

/// 公开函数，执行 redis_command_execute_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_command_execute_async(
    connection_id: &str,
    command: &str,
) -> Result<RedisCommandOutputEntry, String> {
    let client = gateway_client()?;
    let response = client
        .redis_command_execute(
            connection_id,
            &GatewayRedisCommandRequest {
                command: command.to_string(),
            },
        )
        .await?;
    Ok(redis_command_output_from_gateway(response))
}

/// 公开函数，执行 redis_export_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_export_async() -> Result<GatewayRedisConfigBundle, String> {
    let client = gateway_client()?;
    client.redis_export().await
}

/// 公开函数，执行 redis_import_async 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub async fn redis_import_async(
    bundle: &GatewayRedisConfigBundle,
) -> Result<GatewayRedisImportResponse, String> {
    let client = gateway_client()?;
    client.redis_import(bundle).await
}

fn default_redis_history_query() -> GatewayRedisHistoryListQuery {
    GatewayRedisHistoryListQuery {
        offset: Some(0),
        limit: Some(REDIS_HISTORY_PAGE_SIZE),
        connection_id: None,
        query: None,
        only_write: Some(false),
    }
}

fn redis_snapshot_from_gateway(
    settings: GatewayRedisSettings,
    connections: Vec<GatewayRedisConnectionConfig>,
    history_page: GatewayRedisHistoryPage,
) -> RedisToolGatewaySnapshot {
    RedisToolGatewaySnapshot {
        persisted_state: RedisToolPersistedState {
            schema_version: settings.schema_version,
            default_load_count: settings.default_load_count,
            connections: connections
                .into_iter()
                .map(redis_connection_from_gateway)
                .collect(),
            history: history_page
                .items
                .into_iter()
                .map(redis_history_from_gateway)
                .collect(),
            selected_connection_id: settings.selected_connection_id,
        },
        history_offset: history_page.offset,
        history_limit: history_page.limit,
        history_total: history_page.total,
        history_has_more: history_page.has_more,
    }
}

fn redis_connection_from_gateway(connection: GatewayRedisConnectionConfig) -> RedisConnectionConfig {
    RedisConnectionConfig {
        id: connection.id,
        name: connection.name,
        host: connection.host,
        port: connection.port,
        db: connection.db,
        username: connection.username,
        password: connection.password,
        use_tls: connection.use_tls,
        tls_cert: crate::app::state::RedisTlsCertConfig {
            private_key_path: connection.tls_cert.private_key_path,
            public_cert_path: connection.tls_cert.public_cert_path,
            ca_cert_path: connection.tls_cert.ca_cert_path,
        },
        ssh_tunnel: crate::app::state::RedisSshTunnelConfig {
            enabled: connection.ssh_tunnel.enabled,
            host: connection.ssh_tunnel.host,
            port: connection.ssh_tunnel.port,
            username: connection.ssh_tunnel.username,
            password: connection.ssh_tunnel.password,
            private_key_path: connection.ssh_tunnel.private_key_path,
            passphrase: connection.ssh_tunnel.passphrase,
            timeout_secs: connection.ssh_tunnel.timeout_secs,
        },
        sentinel: crate::app::state::RedisSentinelConfig {
            enabled: connection.sentinel.enabled,
            master_name: connection.sentinel.master_name,
            node_password: connection.sentinel.node_password,
        },
        use_cluster: connection.use_cluster,
        read_only: connection.read_only,
        key_pattern: connection.key_pattern,
        last_used_ms: connection.last_used_ms,
        updated_at_ms: connection.updated_at_ms,
    }
}

fn redis_history_from_gateway(record: GatewayRedisHistoryRecord) -> RedisHistoryRecord {
    RedisHistoryRecord {
        time_ms: record.time_ms,
        connection_id: record.connection_id,
        connection_label: record.connection_label,
        command: record.command,
        args: record.args,
        cost_ms: record.cost_ms,
        is_write: record.is_write,
    }
}

fn redis_runtime_overview_from_gateway(overview: GatewayRedisRuntimeOverview) -> RedisRuntimeOverview {
    RedisRuntimeOverview {
        connection_id: overview.connection_id,
        connection_label: overview.connection_label,
        server_version: overview.server_version,
        os: overview.os,
        process_id: overview.process_id,
        used_memory_human: overview.used_memory_human,
        used_memory_peak_human: overview.used_memory_peak_human,
        used_memory_lua_human: overview.used_memory_lua_human,
        connected_clients: overview.connected_clients,
        total_connections_received: overview.total_connections_received,
        total_commands_processed: overview.total_commands_processed,
        keyspace: overview
            .keyspace
            .into_iter()
            .map(redis_keyspace_stat_from_gateway)
            .collect(),
        info_entries: overview
            .info_entries
            .into_iter()
            .map(redis_info_entry_from_gateway)
            .collect(),
    }
}

fn redis_keyspace_stat_from_gateway(stat: GatewayRedisKeyspaceStat) -> RedisKeyspaceStat {
    RedisKeyspaceStat {
        db: stat.db,
        keys: stat.keys,
        expires: stat.expires,
        avg_ttl: stat.avg_ttl,
    }
}

fn redis_info_entry_from_gateway(entry: GatewayRedisInfoEntry) -> RedisInfoEntry {
    RedisInfoEntry {
        key: entry.key,
        value: entry.value,
    }
}

fn redis_key_page_from_gateway(page: GatewayRedisKeyPage) -> RedisKeyPage {
    RedisKeyPage {
        connection_id: page.connection_id,
        pattern: page.pattern,
        keys: page.keys,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }
}

fn redis_key_analysis_from_gateway(analysis: GatewayRedisKeyAnalysisDto) -> RedisKeyAnalysis {
    RedisKeyAnalysis {
        connection_id: analysis.connection_id,
        key: analysis.key,
        key_type: analysis.key_type,
        ttl_secs: analysis.ttl_secs,
        memory_usage_bytes: analysis.memory_usage_bytes,
        preview_command: analysis.preview_command,
        preview_output: analysis.preview_output,
    }
}

fn redis_command_output_from_gateway(response: GatewayRedisCommandResponse) -> RedisCommandOutputEntry {
    RedisCommandOutputEntry {
        command: response.command,
        output: response.output,
        cost_ms: response.cost_ms,
        is_error: response.is_error,
        time_ms: web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX),
    }
}

#[cfg(test)]
#[path = "config_redis_tests.rs"]
mod config_redis_tests;
