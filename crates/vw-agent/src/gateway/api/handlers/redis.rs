//! Redis 路由模块。
//!
//! 本模块为 `/v1/redis` 提供细粒度 REST 接口，覆盖：
//! - 设置读取与更新
//! - 单连接增删改查与激活
//! - 按连接测试
//! - 运行时概览读取与 Key 分析
//! - Key 列表读取与默认创建
//! - 命令执行
//! - 历史分页查询
//! - 服务端导入导出

use axum::{
    Router,
    routing::{get, post},
};

#[path = "redis_config_support.rs"]
mod config_support;
#[path = "redis_handlers.rs"]
mod handlers;
#[path = "redis_runtime.rs"]
mod runtime;
#[path = "redis_storage_support.rs"]
mod storage_support;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/redis/settings", get(handlers::redis_settings_get).put(handlers::redis_settings_put))
        .route(
            "/redis/connections",
            get(handlers::redis_connections_list).post(handlers::redis_connection_create),
        )
        .route(
            "/redis/connections/{connection_id}",
            get(handlers::redis_connection_get)
                .put(handlers::redis_connection_update)
                .delete(handlers::redis_connection_delete),
        )
        .route(
            "/redis/connections/{connection_id}/activate",
            post(handlers::redis_connection_activate),
        )
        .route(
            "/redis/connections/{connection_id}/test",
            post(handlers::redis_connection_test),
        )
        .route(
            "/redis/connections/{connection_id}/overview",
            get(handlers::redis_connection_overview),
        )
        .route(
            "/redis/connections/{connection_id}/keys",
            get(handlers::redis_connection_keys).post(handlers::redis_connection_key_create),
        )
        .route(
            "/redis/connections/{connection_id}/keys/analyze",
            post(handlers::redis_connection_key_analyze),
        )
        .route(
            "/redis/connections/{connection_id}/command",
            post(handlers::redis_command_execute),
        )
        .route("/redis/history", get(handlers::redis_history_list))
        .route("/redis/import", post(handlers::redis_import))
        .route("/redis/export", get(handlers::redis_export))
}

#[cfg(test)]
#[path = "redis_tests.rs"]
mod redis_tests;
