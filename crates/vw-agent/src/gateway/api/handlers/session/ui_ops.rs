//! 会话 UI 快照相关的 Gateway HTTP 处理函数。
//!
//! 本模块负责把前端使用的会话视图模型读写到本地 `ui_store`，并按照实例目录
//! 或显式 scope 隔离不同工作区的会话列表、归档状态和当前会话范围。文件系统
//! 访问通过 `spawn_blocking` 执行，避免阻塞异步运行时中的 HTTP 请求处理线程。

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::HeaderMap;
use vw_api_types::session::GatewaySessionScopeBody;

use super::shared::resolve_scope_from_query;
use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::instance::InstanceQuery;
use crate::app::agent::gateway::instance::resolve_directory;
use crate::session::ui_store;
use crate::session::ui_types as ui_models;

pub(super) async fn session_ui_get(
    Path(session_id): Path<String>,
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Option<ui_models::ChatSession>>, ApiError> {
    let scope = resolve_scope_from_query(&query, &headers);
    let id = session_id.clone();
    // `ui_store` 是同步文件存储接口；放入阻塞线程池可以让网关继续处理其他请求。
    let session =
        tokio::task::spawn_blocking(move || ui_store::load_session_scoped(&id, scope.as_deref()))
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(session))
}

pub(super) async fn session_ui_save(
    Path(session_id): Path<String>,
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
    Json(body): Json<ui_models::ChatSession>,
) -> Result<Json<bool>, ApiError> {
    // URL 中的 session id 是路由授权和定位依据，保存前必须拒绝正文里伪造的 id。
    if body.id != session_id {
        return Err(ApiError::bad_request("body.id does not match session_id"));
    }

    let directory = resolve_directory(&query, &headers);
    let scope = ui_store::resolve_session_scope_id(Some(&directory), None);
    tracing::info!(
        target: "vw_agent",
        session_id = %body.id,
        directory = %directory,
        scope = ?scope,
        message_count = body.messages.len(),
        step_count = body.steps.len(),
        "gateway saving session ui snapshot"
    );
    // 保存完整 UI 快照可能触发磁盘写入，继续隔离到阻塞线程池中执行。
    tokio::task::spawn_blocking(move || ui_store::save_session_scoped(&body, scope.as_deref()))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(true))
}

pub(super) async fn session_ui_previews(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ui_models::ChatSessionMeta>>, ApiError> {
    let scope = resolve_scope_from_query(&query, &headers);
    // 预览列表按 scope 读取，防止不同工作区的历史会话在 UI 中互相串见。
    let previews =
        tokio::task::spawn_blocking(move || ui_store::load_sessions_scoped(scope.as_deref()))
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(previews))
}

pub(super) async fn session_ui_preview_meta(
    Path(session_id): Path<String>,
) -> Result<Json<Option<ui_models::ChatSessionMeta>>, ApiError> {
    let id = session_id.clone();
    let meta = tokio::task::spawn_blocking(move || ui_store::session_preview_meta(&id))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(meta))
}

pub(super) async fn session_archived_get(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<String>>, ApiError> {
    let scope = resolve_scope_from_query(&query, &headers);
    // 归档集合在存储层使用集合语义，对外返回 Vec 便于 JSON 客户端消费。
    let ids = tokio::task::spawn_blocking(move || {
        ui_store::load_archived_session_ids_scoped(scope.as_deref())
            .into_iter()
            .collect::<Vec<String>>()
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(ids))
}

pub(super) async fn session_archived_put(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
    Json(body): Json<Vec<String>>,
) -> Result<Json<bool>, ApiError> {
    let scope = resolve_scope_from_query(&query, &headers);
    tokio::task::spawn_blocking(move || {
        // 写入前去重，保持归档状态是集合而不是顺序敏感列表。
        let set: std::collections::HashSet<String> = body.into_iter().collect();
        ui_store::save_archived_session_ids_scoped(&set, scope.as_deref());
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(true))
}

pub(super) async fn session_path_get(
    Path(session_id): Path<String>,
) -> Result<Json<Option<String>>, ApiError> {
    let id = session_id.clone();
    let path = tokio::task::spawn_blocking(move || {
        ui_store::session_file_path(&id).map(|p| p.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(path))
}

pub(super) async fn session_scope_get() -> Result<Json<Option<String>>, ApiError> {
    let scope = tokio::task::spawn_blocking(ui_store::current_session_scope)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(scope))
}

pub(super) async fn session_scope_put(
    Json(body): Json<GatewaySessionScopeBody>,
) -> Result<Json<bool>, ApiError> {
    tokio::task::spawn_blocking(move || ui_store::set_session_scope(body.scope.as_deref()))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(true))
}

pub(super) async fn session_ui_get_any(
    Path(session_id): Path<String>,
) -> Result<Json<Option<ui_models::ChatSession>>, ApiError> {
    let id = session_id.clone();
    let session = tokio::task::spawn_blocking(move || ui_store::load_session_any(&id))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(session))
}

#[cfg(test)]
#[path = "ui_ops_tests.rs"]
mod ui_ops_tests;
