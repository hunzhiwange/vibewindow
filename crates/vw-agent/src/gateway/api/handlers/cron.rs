//! 定时任务（Cron）API 处理器模块
//!
//! 本模块提供与定时任务管理相关的 HTTP API 端点处理器，包括：
//! - 列出所有定时任务
//! - 添加新的定时任务
//! - 删除指定的定时任务
//!
//! 所有端点均需要身份认证，通过 `auth::require_auth` 进行验证。
//!
//! # API 端点
//!
//! - `GET /api/cron` - 获取所有定时任务列表
//! - `POST /api/cron` - 添加新的定时任务
//! - `DELETE /api/cron/:id` - 删除指定的定时任务

use super::super::types::CronAddBody;
use crate::app::agent::gateway::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json as JsonResponse},
};

/// 处理定时任务列表请求
///
/// 返回当前系统中所有已配置的定时任务，包括其 ID、名称、命令、
/// 下次执行时间、上次执行时间、上次执行状态以及是否启用等信息。
///
/// # 参数
///
/// - `State(state)` - 应用状态，包含配置和运行时信息
/// - `headers` - HTTP 请求头，用于身份认证
///
/// # 返回值
///
/// 成功时返回 JSON 格式的任务列表：
/// ```json
/// {
///   "jobs": [
///     {
///       "id": "job_123",
///       "name": "备份任务",
///       "command": "/usr/bin/backup.sh",
///       "next_run": "2024-01-01T00:00:00Z",
///       "last_run": "2023-12-31T00:00:00Z",
///       "last_status": "success",
///       "enabled": true
///     }
///   ]
/// }
/// ```
///
/// 失败时返回 500 状态码和错误信息。
///
/// # 认证
///
/// 需要有效的认证令牌，否则返回 401 未授权错误。
pub async fn handle_api_cron_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 验证请求的身份认证信息
    if let Err(e) = super::super::auth::require_auth(&state, &headers) {
        return e.into_response();
    }

    // 获取配置的克隆副本以避免长时间持有锁
    let config = state.config.lock().clone();

    // 尝试列出所有定时任务
    match crate::app::agent::cron::list_jobs(&config) {
        Ok(jobs) => {
            // 将任务列表转换为 JSON 格式
            let jobs_json: Vec<serde_json::Value> = jobs
                .iter()
                .map(|job| {
                    // 构建每个任务的 JSON 对象
                    serde_json::json!({
                        "id": job.id,
                        "name": job.name,
                        "command": job.command,
                        "next_run": job.next_run.to_rfc3339(),
                        "last_run": job.last_run.map(|t| t.to_rfc3339()),
                        "last_status": job.last_status,
                        "enabled": job.enabled,
                    })
                })
                .collect();
            JsonResponse(serde_json::json!({"jobs": jobs_json})).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({"error": format!("Failed to list cron jobs: {e}")})),
        )
            .into_response(),
    }
}

/// 处理添加定时任务请求
///
/// 根据请求体中的信息创建新的定时任务。任务创建后将按照
/// 指定的 cron 表达式自动执行。
///
/// # 参数
///
/// - `State(state)` - 应用状态，包含配置和运行时信息
/// - `headers` - HTTP 请求头，用于身份认证
/// - `Json(body)` - 请求体，包含任务名称、调度表达式和要执行的命令
///
/// # 返回值
///
/// 成功时返回新创建的任务信息：
/// ```json
/// {
///   "status": "ok",
///   "job": {
///     "id": "job_456",
///     "name": "日志清理",
///     "command": "/usr/bin/clean-logs.sh",
///     "enabled": true
///   }
/// }
/// ```
///
/// 失败时返回 500 状态码和错误信息。
///
/// # 认证
///
/// 需要有效的认证令牌，否则返回 401 未授权错误。
pub async fn handle_api_cron_add(
    State(state): State<AppState>,
    headers: HeaderMap,
    JsonResponse(body): JsonResponse<CronAddBody>,
) -> impl IntoResponse {
    // 验证请求的身份认证信息
    if let Err(e) = super::super::auth::require_auth(&state, &headers) {
        return e.into_response();
    }

    // 获取配置的克隆副本以避免长时间持有锁
    let config = state.config.lock().clone();

    // 构建调度表达式，使用 cron 表达式格式
    let schedule = crate::app::agent::cron::Schedule::Cron { expr: body.schedule, tz: None };

    // 尝试添加新的 shell 任务
    match crate::app::agent::cron::add_shell_job(&config, body.name, schedule, &body.command) {
        Ok(job) => JsonResponse(serde_json::json!({
            "status": "ok",
            "job": {
                "id": job.id,
                "name": job.name,
                "command": job.command,
                "enabled": job.enabled,
            }
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({"error": format!("Failed to add cron job: {e}")})),
        )
            .into_response(),
    }
}

/// 处理删除定时任务请求
///
/// 根据 ID 删除指定的定时任务。删除后任务将不再执行。
///
/// # 参数
///
/// - `State(state)` - 应用状态，包含配置和运行时信息
/// - `headers` - HTTP 请求头，用于身份认证
/// - `Path(id)` - URL 路径参数，指定要删除的任务 ID
///
/// # 返回值
///
/// 成功时返回：
/// ```json
/// {
///   "status": "ok"
/// }
/// ```
///
/// 失败时返回 500 状态码和错误信息。
///
/// # 认证
///
/// 需要有效的认证令牌，否则返回 401 未授权错误。
pub async fn handle_api_cron_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // 验证请求的身份认证信息
    if let Err(e) = super::super::auth::require_auth(&state, &headers) {
        return e.into_response();
    }

    // 获取配置的克隆副本以避免长时间持有锁
    let config = state.config.lock().clone();

    // 尝试删除指定的任务
    match crate::app::agent::cron::remove_job(&config, &id) {
        Ok(()) => JsonResponse(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(serde_json::json!({"error": format!("Failed to remove cron job: {e}")})),
        )
            .into_response(),
    }
}
