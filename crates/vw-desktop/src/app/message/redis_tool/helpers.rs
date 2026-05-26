//! 提供 Redis 工具消息处理使用的局部辅助函数。

use super::{
    App, GatewayRedisHistoryListQuery, Message, REDIS_HISTORY_PAGE_SIZE, RedisToolMessage, Task,
};

/// 处理 `current_default_load_count` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn current_default_load_count(app: &App) -> u32 {
    app.redis_tool
        .default_load_count_input
        .trim()
        .parse::<u32>()
        .ok()
        .unwrap_or(500)
        .clamp(1, 10_000)
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;

/// 处理 `current_history_query` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn current_history_query(
    app: &App,
    offset: Option<usize>,
) -> GatewayRedisHistoryListQuery {
    let limit = app.redis_tool.history_page_limit.max(REDIS_HISTORY_PAGE_SIZE);
    GatewayRedisHistoryListQuery {
        offset: Some(offset.unwrap_or(app.redis_tool.history_page_offset)),
        limit: Some(limit),
        connection_id: None,
        query: (!app.redis_tool.history_filter.trim().is_empty())
            .then_some(app.redis_tool.history_filter.trim().to_string()),
        only_write: Some(app.redis_tool.history_only_write),
    }
}

/// 处理 `pick_file_path` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回 `None` 表示输入为空或当前状态不需要生成后续值。
pub(super) async fn pick_file_path(
    filters: Vec<(&'static str, Vec<&'static str>)>,
) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut dialog = rfd::AsyncFileDialog::new();
        for (name, extensions) in filters {
            dialog = dialog.add_filter(name, &extensions);
        }

        let handle = dialog.pick_file().await;
        return handle.map(|file| file.path().to_string_lossy().to_string());
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = filters;
        None
    }
}

/// 处理 `notify_success` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn notify_success(app: &mut App, message: &str) {
    app.redis_tool.notification = Some(message.to_string());
    app.redis_tool.clear_gateway_error();
}

/// 处理 `clear_notification_task` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::RedisTool(RedisToolMessage::ClearNotification),
    )
}
