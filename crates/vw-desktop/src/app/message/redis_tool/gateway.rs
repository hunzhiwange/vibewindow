//! 封装 Redis 工具访问桌面网关的异步加载任务。

use super::helpers::{current_default_load_count, current_history_query};
use super::{
    App, Message, RedisToolMessage, Task, load_redis_tool_snapshot_async,
    redis_connection_key_analyze_async, redis_connection_keys_async,
    redis_connection_overview_async,
};

/// 处理 `start_snapshot_reload` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn start_snapshot_reload(
    app: &mut App,
    offset: usize,
    loading_label: &str,
    success_message: Option<String>,
) -> Task<Message> {
    app.redis_tool.begin_gateway_request(loading_label);
    let query = current_history_query(app, Some(offset));
    Task::perform(async move { load_redis_tool_snapshot_async(query).await }, move |result| {
        Message::RedisTool(RedisToolMessage::SnapshotLoaded { success_message, result })
    })
}

#[cfg(test)]
#[path = "gateway_tests.rs"]
mod gateway_tests;

/// 处理 `start_runtime_reload` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn start_runtime_reload(
    app: &mut App,
    connection_id: String,
    loading_label: &str,
    success_message: Option<String>,
) -> Task<Message> {
    app.redis_tool.begin_gateway_request(loading_label);
    let request_connection_id = connection_id.clone();
    Task::perform(
        async move { redis_connection_overview_async(&request_connection_id).await },
        move |result| {
            Message::RedisTool(RedisToolMessage::RuntimeLoaded {
                connection_id,
                success_message,
                result,
            })
        },
    )
}

/// 处理 `start_key_page_reload` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn start_key_page_reload(
    app: &mut App,
    connection_id: String,
    append: bool,
    loading_label: &str,
) -> Task<Message> {
    app.redis_tool.begin_gateway_request(loading_label);
    let request_connection_id = connection_id.clone();
    let pattern = if app.redis_tool.key_browser_pattern.trim().is_empty() {
        "*".to_string()
    } else {
        app.redis_tool.key_browser_pattern.trim().to_string()
    };
    let cursor = if append { app.redis_tool.key_browser_cursor } else { 0 };
    let count = current_default_load_count(app);
    Task::perform(
        async move {
            redis_connection_keys_async(&request_connection_id, &pattern, cursor, count).await
        },
        move |result| {
            Message::RedisTool(RedisToolMessage::KeyPageLoaded { connection_id, append, result })
        },
    )
}

/// 处理 `start_key_analysis_reload` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn start_key_analysis_reload(
    app: &mut App,
    connection_id: String,
    key: String,
    loading_label: &str,
) -> Task<Message> {
    app.redis_tool.begin_gateway_request(loading_label);
    let request_connection_id = connection_id.clone();
    let request_key = key.clone();
    Task::perform(
        async move { redis_connection_key_analyze_async(&request_connection_id, &request_key).await },
        move |result| {
            Message::RedisTool(RedisToolMessage::KeyAnalysisLoaded { connection_id, key, result })
        },
    )
}
