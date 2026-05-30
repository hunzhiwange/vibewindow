//! Redis 工具的 HTTP handler 集合。
//!
//! 本模块负责把桌面端 Redis 工具请求映射到配置存储、运行时 Redis 操作和历史记录。
//! 可能阻塞的 Redis 网络访问会交给 `spawn_blocking`，避免占用 async runtime 的执行线程。

use std::collections::HashSet;
use std::time::Instant;

use axum::{
    Json,
    extract::{Path, Query},
};
use vw_api_types::tool::{
    GatewayRedisCommandRequest, GatewayRedisCommandResponse, GatewayRedisConfigBundle,
    GatewayRedisConnectionConfig, GatewayRedisConnectionTestResponse,
    GatewayRedisConnectionUpsertBody, GatewayRedisDeleteResponse, GatewayRedisHistoryListQuery,
    GatewayRedisHistoryPage, GatewayRedisHistoryRecord, GatewayRedisImportResponse,
    GatewayRedisKeyAnalysis, GatewayRedisKeyAnalysisRequest, GatewayRedisKeyCreateRequest,
    GatewayRedisKeyListQuery, GatewayRedisKeyPage, GatewayRedisRuntimeOverview,
    GatewayRedisSettings, GatewayRedisSettingsUpdateBody,
};

use super::{config_support, runtime, storage_support};
use crate::app::agent::gateway::ApiError;

/// 获取 Redis 工具全局设置。
///
/// # 返回值
///
/// 返回当前设置，缺失时使用默认值。
pub(super) async fn redis_settings_get() -> Result<Json<GatewayRedisSettings>, ApiError> {
    Ok(Json(storage_support::load_settings().await))
}

/// 更新 Redis 工具全局设置。
///
/// # 参数
///
/// * `body` - 可更新的设置字段。
///
/// # 返回值
///
/// 返回保存后的设置。
///
/// # 错误处理
///
/// 设置保存失败时返回 [`ApiError`]；历史记录失败不影响主结果。
pub(super) async fn redis_settings_put(
    Json(body): Json<GatewayRedisSettingsUpdateBody>,
) -> Result<Json<GatewayRedisSettings>, ApiError> {
    let mut settings = storage_support::load_settings().await;
    settings.default_load_count = body.default_load_count.clamp(1, 10_000);
    storage_support::save_settings(&settings).await?;

    storage_support::append_history_best_effort(GatewayRedisHistoryRecord {
        time_ms: storage_support::now_ms(),
        connection_id: None,
        connection_label: "全局配置".to_string(),
        command: "UPDATE_SETTINGS".to_string(),
        args: format!("default_load_count={}", settings.default_load_count),
        cost_ms: 0,
        is_write: true,
    })
    .await;

    Ok(Json(settings))
}

/// 列出所有 Redis 连接配置。
///
/// # 返回值
///
/// 返回按最近使用时间排序的连接列表。
pub(super) async fn redis_connections_list()
-> Result<Json<Vec<GatewayRedisConnectionConfig>>, ApiError> {
    let mut connections = storage_support::load_connections().await;
    storage_support::sort_connections(&mut connections);
    Ok(Json(connections))
}

/// 获取单个 Redis 连接配置。
///
/// # 参数
///
/// * `connection_id` - 路径中的连接 id。
///
/// # 错误处理
///
/// 连接不存在时返回 404。
pub(super) async fn redis_connection_get(
    Path(connection_id): Path<String>,
) -> Result<Json<GatewayRedisConnectionConfig>, ApiError> {
    let connections = storage_support::load_connections().await;
    let connection = connections
        .into_iter()
        .find(|item| item.id == connection_id)
        .ok_or_else(|| ApiError::not_found("Redis 连接不存在"))?;
    Ok(Json(connection))
}

/// 创建 Redis 连接配置。
///
/// # 参数
///
/// * `body` - 前端提交的连接配置。
///
/// # 返回值
///
/// 返回新建连接，并将其设为当前选中连接。
///
/// # 错误处理
///
/// 配置校验或存储写入失败时返回 [`ApiError`]。
pub(super) async fn redis_connection_create(
    Json(body): Json<GatewayRedisConnectionUpsertBody>,
) -> Result<Json<GatewayRedisConnectionConfig>, ApiError> {
    let now_ms = storage_support::now_ms();
    let mut connections = storage_support::load_connections().await;
    let connection = config_support::new_connection_from_upsert(&body, now_ms)?;
    connections.push(connection.clone());
    storage_support::sort_connections(&mut connections);
    storage_support::save_connections(&connections).await?;

    let mut settings = storage_support::load_settings().await;
    settings.selected_connection_id = Some(connection.id.clone());
    storage_support::save_settings(&settings).await?;

    storage_support::append_history_best_effort(storage_support::history_record(
        Some(&connection),
        "SAVE_CONFIG",
        storage_support::compact_connection_args(&connection),
        0,
        true,
    ))
    .await;

    Ok(Json(connection))
}

/// 更新 Redis 连接配置。
///
/// # 参数
///
/// * `connection_id` - 待更新连接 id。
/// * `body` - 新连接配置。
///
/// # 错误处理
///
/// 连接不存在、配置非法或存储写入失败时返回 [`ApiError`]。
pub(super) async fn redis_connection_update(
    Path(connection_id): Path<String>,
    Json(body): Json<GatewayRedisConnectionUpsertBody>,
) -> Result<Json<GatewayRedisConnectionConfig>, ApiError> {
    let now_ms = storage_support::now_ms();
    let mut connections = storage_support::load_connections().await;
    let Some(index) = connections.iter().position(|item| item.id == connection_id) else {
        return Err(ApiError::not_found("Redis 连接不存在"));
    };

    let existing = connections[index].clone();
    let updated = config_support::updated_connection_from_upsert(&existing, &body, now_ms)?;
    connections[index] = updated.clone();
    storage_support::sort_connections(&mut connections);
    storage_support::save_connections(&connections).await?;

    let mut settings = storage_support::load_settings().await;
    settings.selected_connection_id = Some(updated.id.clone());
    storage_support::save_settings(&settings).await?;

    storage_support::append_history_best_effort(storage_support::history_record(
        Some(&updated),
        "UPDATE_CONFIG",
        storage_support::compact_connection_args(&updated),
        0,
        true,
    ))
    .await;

    Ok(Json(updated))
}

/// 删除 Redis 连接配置。
///
/// # 参数
///
/// * `connection_id` - 待删除连接 id。
///
/// # 返回值
///
/// 返回被删除的连接 id。
///
/// # 错误处理
///
/// 连接不存在或存储写入失败时返回 [`ApiError`]。
pub(super) async fn redis_connection_delete(
    Path(connection_id): Path<String>,
) -> Result<Json<GatewayRedisDeleteResponse>, ApiError> {
    let mut connections = storage_support::load_connections().await;
    let Some(index) = connections.iter().position(|item| item.id == connection_id) else {
        return Err(ApiError::not_found("Redis 连接不存在"));
    };

    let removed = connections.remove(index);
    storage_support::sort_connections(&mut connections);
    storage_support::save_connections(&connections).await?;

    let mut settings = storage_support::load_settings().await;
    if settings.selected_connection_id.as_deref() == Some(removed.id.as_str()) {
        settings.selected_connection_id = connections.first().map(|item| item.id.clone());
    }
    storage_support::save_settings(&settings).await?;

    storage_support::append_history_best_effort(storage_support::history_record(
        Some(&removed),
        "DELETE_CONFIG",
        storage_support::compact_connection_args(&removed),
        0,
        true,
    ))
    .await;

    Ok(Json(GatewayRedisDeleteResponse { deleted_id: removed.id }))
}

/// 将指定 Redis 连接设为当前连接。
///
/// # 参数
///
/// * `connection_id` - 待激活连接 id。
///
/// # 返回值
///
/// 返回更新后的全局设置。
pub(super) async fn redis_connection_activate(
    Path(connection_id): Path<String>,
) -> Result<Json<GatewayRedisSettings>, ApiError> {
    let now_ms = storage_support::now_ms();
    let mut connections = storage_support::load_connections().await;
    let Some(index) = connections.iter().position(|item| item.id == connection_id) else {
        return Err(ApiError::not_found("Redis 连接不存在"));
    };

    connections[index].last_used_ms = Some(now_ms);
    let connection = connections[index].clone();
    storage_support::sort_connections(&mut connections);
    storage_support::save_connections(&connections).await?;

    let mut settings = storage_support::load_settings().await;
    settings.selected_connection_id = Some(connection.id.clone());
    storage_support::save_settings(&settings).await?;

    storage_support::append_history_best_effort(storage_support::history_record(
        Some(&connection),
        "OPEN_CONNECTION",
        storage_support::compact_connection_args(&connection),
        0,
        false,
    ))
    .await;

    Ok(Json(settings))
}

/// 测试 Redis 连接可用性。
///
/// # 参数
///
/// * `connection_id` - 待测试连接 id。
///
/// # 返回值
///
/// 返回测试结果、服务端消息和耗时。
///
/// # 错误处理
///
/// 连接不存在、阻塞任务失败或 Redis PING 失败时返回 [`ApiError`]。
pub(super) async fn redis_connection_test(
    Path(connection_id): Path<String>,
) -> Result<Json<GatewayRedisConnectionTestResponse>, ApiError> {
    let connection = storage_support::load_connection_by_id(&connection_id).await?;

    let test_connection = connection.clone();
    let started_at = Instant::now();
    let result =
        tokio::task::spawn_blocking(move || runtime::ping_redis_connection(&test_connection))
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;
    let latency_ms = started_at.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

    match result {
        Ok(message) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "TEST_CONNECTION",
                storage_support::compact_connection_args(&connection),
                latency_ms,
                false,
            ))
            .await;

            Ok(Json(GatewayRedisConnectionTestResponse { ok: true, message, latency_ms }))
        }
        Err(error) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "TEST_CONNECTION",
                format!("{} error={error}", storage_support::compact_connection_args(&connection)),
                latency_ms,
                false,
            ))
            .await;

            Err(ApiError::bad_request(error))
        }
    }
}

/// 获取 Redis 运行时概览。
///
/// # 参数
///
/// * `connection_id` - 目标连接 id。
///
/// # 返回值
///
/// 返回 INFO 解析后的服务端、内存、客户端和 keyspace 概览。
pub(super) async fn redis_connection_overview(
    Path(connection_id): Path<String>,
) -> Result<Json<GatewayRedisRuntimeOverview>, ApiError> {
    let connection = storage_support::load_connection_by_id(&connection_id).await?;
    let overview_connection = connection.clone();
    let started_at = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        runtime::load_redis_runtime_overview(&overview_connection)
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    let cost_ms = started_at.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

    match result {
        Ok(overview) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "LOAD_RUNTIME",
                format!(
                    "info_entries={} keyspace={}",
                    overview.info_entries.len(),
                    overview.keyspace.len()
                ),
                cost_ms,
                false,
            ))
            .await;
            Ok(Json(overview))
        }
        Err(error) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "LOAD_RUNTIME",
                "status=error".to_string(),
                cost_ms,
                false,
            ))
            .await;
            Err(ApiError::bad_request(error))
        }
    }
}

/// 分页扫描 Redis key。
///
/// # 参数
///
/// * `connection_id` - 目标连接 id。
/// * `query` - SCAN 游标、数量与可选匹配模式。
///
/// # 返回值
///
/// 返回本页 key、下一游标与是否还有更多数据。
pub(super) async fn redis_connection_keys(
    Path(connection_id): Path<String>,
    Query(query): Query<GatewayRedisKeyListQuery>,
) -> Result<Json<GatewayRedisKeyPage>, ApiError> {
    let connection = storage_support::load_connection_by_id(&connection_id).await?;
    let cursor = query.cursor.unwrap_or(0);
    let count = query.count.unwrap_or(200).clamp(1, 10_000);
    let pattern = query
        .pattern
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(connection.key_pattern.trim());
    let pattern = if pattern.is_empty() { "*".to_string() } else { pattern.to_string() };

    let scan_connection = connection.clone();
    let scan_pattern = pattern.clone();
    let started_at = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        runtime::scan_redis_keys(&scan_connection, cursor, count, &scan_pattern)
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    let cost_ms = started_at.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

    match result {
        Ok(page) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "LOAD_KEYS",
                format!(
                    "pattern={} cursor={} count={} returned={} next_cursor={}",
                    pattern,
                    cursor,
                    count,
                    page.keys.len(),
                    page.next_cursor
                ),
                cost_ms,
                false,
            ))
            .await;
            Ok(Json(page))
        }
        Err(error) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "LOAD_KEYS",
                format!("pattern={} cursor={} count={} status=error", pattern, cursor, count),
                cost_ms,
                false,
            ))
            .await;
            Err(ApiError::bad_request(error))
        }
    }
}

/// 新建一个带默认内容的 Redis key。
///
/// # 参数
///
/// * `connection_id` - 目标连接 id。
/// * `body` - key 名和类型。
///
/// # 返回值
///
/// 返回新 key 的分析结果。
///
/// # 错误处理
///
/// 只读连接、key 已存在、类型不支持或 Redis 写入失败时返回 [`ApiError`]。
pub(super) async fn redis_connection_key_create(
    Path(connection_id): Path<String>,
    Json(body): Json<GatewayRedisKeyCreateRequest>,
) -> Result<Json<GatewayRedisKeyAnalysis>, ApiError> {
    let connection = storage_support::load_connection_by_id(&connection_id).await?;
    let key = body.key.trim().to_string();
    let key_type = body.key_type.trim().to_string();
    if key.is_empty() {
        return Err(ApiError::bad_request("Key 名不能为空"));
    }
    if key_type.is_empty() {
        return Err(ApiError::bad_request("请选择 Key 类型"));
    }

    let create_connection = connection.clone();
    let create_key = key.clone();
    let create_type = key_type.clone();
    let started_at = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        runtime::create_redis_key_with_default(&create_connection, &create_key, &create_type)
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    let cost_ms = started_at.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

    match result {
        Ok(analysis) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "CREATE_KEY",
                format!("key={} type={}", key, key_type),
                cost_ms,
                true,
            ))
            .await;
            Ok(Json(analysis))
        }
        Err(error) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "CREATE_KEY",
                format!("key={} type={} status=error", key, key_type),
                cost_ms,
                true,
            ))
            .await;
            Err(ApiError::bad_request(error))
        }
    }
}

/// 分析单个 Redis key。
///
/// # 参数
///
/// * `connection_id` - 目标连接 id。
/// * `body` - key 名。
///
/// # 返回值
///
/// 返回类型、TTL、内存占用和预览输出。
pub(super) async fn redis_connection_key_analyze(
    Path(connection_id): Path<String>,
    Json(body): Json<GatewayRedisKeyAnalysisRequest>,
) -> Result<Json<GatewayRedisKeyAnalysis>, ApiError> {
    let connection = storage_support::load_connection_by_id(&connection_id).await?;
    let key = body.key.trim().to_string();
    if key.is_empty() {
        return Err(ApiError::bad_request("Key 名不能为空"));
    }

    let analyze_connection = connection.clone();
    let analyze_key = key.clone();
    let started_at = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        runtime::analyze_redis_key(&analyze_connection, &analyze_key)
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    let cost_ms = started_at.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

    match result {
        Ok(analysis) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "ANALYZE_KEY",
                format!("key={} type={}", key, analysis.key_type),
                cost_ms,
                false,
            ))
            .await;
            Ok(Json(analysis))
        }
        Err(error) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                "ANALYZE_KEY",
                format!("key={} status=error", key),
                cost_ms,
                false,
            ))
            .await;
            Err(ApiError::bad_request(error))
        }
    }
}

/// 执行原始 Redis 命令。
///
/// # 参数
///
/// * `connection_id` - 目标连接 id。
/// * `body` - shell-like 命令行。
///
/// # 返回值
///
/// 返回命令输出和耗时；Redis 命令错误会以 `is_error=true` 形式返回，便于 UI 展示。
pub(super) async fn redis_command_execute(
    Path(connection_id): Path<String>,
    Json(body): Json<GatewayRedisCommandRequest>,
) -> Result<Json<GatewayRedisCommandResponse>, ApiError> {
    let connection = storage_support::load_connection_by_id(&connection_id).await?;
    let command = body.command.trim().to_string();
    if command.is_empty() {
        return Err(ApiError::bad_request("请输入 Redis 命令"));
    }

    let execute_connection = connection.clone();
    let execute_command = command.clone();
    let started_at = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        runtime::execute_redis_command(&execute_connection, &execute_command)
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    let cost_ms = started_at.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let command_name = runtime::summarize_command_name(&command);
    let args_summary = runtime::summarize_command_args(&command);
    let is_write = runtime::classify_write_command(&command_name);

    match result {
        Ok(output) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                &command_name,
                args_summary,
                cost_ms,
                is_write,
            ))
            .await;

            Ok(Json(GatewayRedisCommandResponse { command, output, cost_ms, is_error: false }))
        }
        Err(error) => {
            storage_support::append_history_best_effort(storage_support::history_record(
                Some(&connection),
                &command_name,
                args_summary,
                cost_ms,
                is_write,
            ))
            .await;

            Ok(Json(GatewayRedisCommandResponse {
                command,
                output: error,
                cost_ms,
                is_error: true,
            }))
        }
    }
}

/// 查询 Redis 操作历史。
///
/// # 参数
///
/// * `query` - 分页、连接过滤、文本搜索和只看写操作开关。
///
/// # 返回值
///
/// 返回过滤后的分页历史。
pub(super) async fn redis_history_list(
    Query(query): Query<GatewayRedisHistoryListQuery>,
) -> Result<Json<GatewayRedisHistoryPage>, ApiError> {
    let records = storage_support::load_history().await;
    let offset = query.offset.unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(storage_support::REDIS_HISTORY_PAGE_LIMIT)
        .clamp(1, storage_support::REDIS_HISTORY_PAGE_LIMIT_MAX);
    let filter_connection_id =
        query.connection_id.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let filter_text = query
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);
    let only_write = query.only_write.unwrap_or(false);

    let filtered: Vec<_> = records
        .into_iter()
        .filter(|record| {
            (!only_write || record.is_write)
                && filter_connection_id.is_none_or(|connection_id| {
                    record.connection_id.as_deref() == Some(connection_id)
                })
                && filter_text.as_ref().is_none_or(|query| {
                    record.connection_label.to_ascii_lowercase().contains(query)
                        || record.command.to_ascii_lowercase().contains(query)
                        || record.args.to_ascii_lowercase().contains(query)
                })
        })
        .collect();

    let total = filtered.len();
    let items = filtered.into_iter().skip(offset).take(limit).collect::<Vec<_>>();
    let has_more = offset.saturating_add(items.len()) < total;

    Ok(Json(GatewayRedisHistoryPage { items, offset, limit, total, has_more }))
}

/// 导入 Redis 工具配置包。
///
/// # 参数
///
/// * `body` - 配置包，包含 schema、默认加载数量和连接列表。
///
/// # 返回值
///
/// 返回导入数量、保存后的默认加载数量和选中连接 id。
///
/// # 错误处理
///
/// 任意连接配置非法或存储写入失败时返回 [`ApiError`]。
pub(super) async fn redis_import(
    Json(body): Json<GatewayRedisConfigBundle>,
) -> Result<Json<GatewayRedisImportResponse>, ApiError> {
    let GatewayRedisConfigBundle {
        schema_version,
        default_load_count,
        connections: imported_connections,
    } = body;
    let now_ms = storage_support::now_ms();
    let mut seen_ids = HashSet::new();
    let mut connections = Vec::with_capacity(imported_connections.len());

    for connection in imported_connections {
        connections.push(config_support::normalize_import_connection(
            connection,
            now_ms,
            &mut seen_ids,
        )?);
    }

    storage_support::sort_connections(&mut connections);
    storage_support::save_connections(&connections).await?;

    let mut settings = storage_support::load_settings().await;
    settings.schema_version = schema_version.max(1);
    settings.default_load_count = default_load_count.clamp(1, 10_000);
    settings.selected_connection_id = connections.first().map(|item| item.id.clone());
    storage_support::save_settings(&settings).await?;

    storage_support::append_history_best_effort(GatewayRedisHistoryRecord {
        time_ms: now_ms,
        connection_id: None,
        connection_label: format!("{} 个连接", connections.len()),
        command: "IMPORT_CONFIG".to_string(),
        args: format!("default_load_count={}", settings.default_load_count),
        cost_ms: 0,
        is_write: true,
    })
    .await;

    Ok(Json(GatewayRedisImportResponse {
        imported_count: connections.len(),
        default_load_count: settings.default_load_count,
        selected_connection_id: settings.selected_connection_id,
    }))
}

/// 导出 Redis 工具配置包。
///
/// # 返回值
///
/// 返回当前 settings 和所有连接配置。
pub(super) async fn redis_export() -> Result<Json<GatewayRedisConfigBundle>, ApiError> {
    let settings = storage_support::load_settings().await;
    let mut connections = storage_support::load_connections().await;
    storage_support::sort_connections(&mut connections);

    storage_support::append_history_best_effort(GatewayRedisHistoryRecord {
        time_ms: storage_support::now_ms(),
        connection_id: None,
        connection_label: format!("{} 个连接", connections.len()),
        command: "EXPORT_CONFIG".to_string(),
        args: "导出全部连接配置".to_string(),
        cost_ms: 0,
        is_write: false,
    })
    .await;

    Ok(Json(GatewayRedisConfigBundle {
        schema_version: settings.schema_version,
        default_load_count: settings.default_load_count,
        connections,
    }))
}

#[cfg(test)]
#[path = "redis_handlers_tests.rs"]
mod redis_handlers_tests;
