//! 处理 Redis 工具页面导航、选中连接、键浏览和加载结果回写。

use super::gateway::{
    start_key_analysis_reload, start_key_page_reload, start_runtime_reload, start_snapshot_reload,
};
use super::helpers::{clear_notification_task, current_history_query, notify_success};
use super::{App, Message, RedisDetailTab, RedisToolMessage, Task};
use super::{
    load_redis_tool_snapshot_async, redis_connection_activate_async, redis_connection_keys_async,
    redis_connection_overview_async,
};

/// 处理 `open_settings_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn open_settings_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.show_history_modal = false;
    app.redis_tool.close_connection_modal();
    app.redis_tool.close_create_key_modal();
    app.redis_tool.show_settings_modal = true;
    Task::none()
}

#[cfg(test)]
#[path = "navigation_tests.rs"]
mod navigation_tests;

/// 处理 `close_settings_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn close_settings_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.show_settings_modal = false;
    Task::none()
}

/// 处理 `open_history_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn open_history_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.show_settings_modal = false;
    app.redis_tool.close_connection_modal();
    app.redis_tool.close_create_key_modal();
    app.redis_tool.show_history_modal = true;
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }
    start_snapshot_reload(app, 0, "加载历史", None)
}

/// 处理 `close_history_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn close_history_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.show_history_modal = false;
    Task::none()
}

/// 处理 `open_connection_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn open_connection_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.show_settings_modal = false;
    app.redis_tool.show_history_modal = false;
    app.redis_tool.close_create_key_modal();
    app.redis_tool.open_connection_modal();
    Task::none()
}

/// 处理 `close_connection_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn close_connection_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.close_connection_modal();
    Task::none()
}

/// 处理 `open_create_key_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn open_create_key_modal(app: &mut App) -> Task<Message> {
    if app.redis_tool.selected_connection_id.is_none() {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    }

    app.redis_tool.show_settings_modal = false;
    app.redis_tool.show_history_modal = false;
    app.redis_tool.close_connection_modal();
    app.redis_tool.open_create_key_modal();
    Task::none()
}

/// 处理 `close_create_key_modal` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn close_create_key_modal(app: &mut App) -> Task<Message> {
    app.redis_tool.close_create_key_modal();
    Task::none()
}

/// 处理 `new_connection` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn new_connection(app: &mut App) -> Task<Message> {
    app.redis_tool.selected_connection_id = None;
    app.redis_tool.reset_draft();
    app.redis_tool.clear_runtime_state();
    app.redis_tool.connection_search_query.clear();
    app.redis_tool.show_settings_modal = false;
    app.redis_tool.show_history_modal = false;
    app.redis_tool.close_create_key_modal();
    app.redis_tool.open_connection_modal();
    notify_success(app, "已打开新建连接");
    Task::batch(vec![clear_notification_task()])
}

/// 处理 `search_connections_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn search_connections_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.connection_search_query = value;
    Task::none()
}

/// 处理 `select_connection` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn select_connection(app: &mut App, connection_id: String) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    if let Some(connection) =
        app.redis_tool.connections.iter().find(|connection| connection.id == connection_id).cloned()
    {
        app.redis_tool.selected_connection_id = Some(connection_id.clone());
        app.redis_tool.load_connection_into_draft(&connection);
        app.redis_tool.draft_is_new = false;
        app.redis_tool.detail_tab = RedisDetailTab::Connection;
    }

    app.redis_tool.begin_gateway_request("切换连接");
    let query = current_history_query(app, Some(0));
    let active_detail_tab = app.redis_tool.detail_tab;
    Task::perform(
        async move {
            redis_connection_activate_async(&connection_id).await?;
            let snapshot = load_redis_tool_snapshot_async(query).await?;
            let default_load_count = snapshot.persisted_state.default_load_count.max(1);
            let pattern = snapshot
                .persisted_state
                .connections
                .iter()
                .find(|connection| connection.id == connection_id)
                .map(|connection| connection.key_pattern.clone())
                .unwrap_or_else(|| "*".to_string());
            let runtime = if active_detail_tab.requires_runtime() {
                Some(redis_connection_overview_async(&connection_id).await)
            } else {
                None
            };
            let keys = if active_detail_tab.requires_keys() {
                Some(
                    redis_connection_keys_async(&connection_id, &pattern, 0, default_load_count)
                        .await,
                )
            } else {
                None
            };
            Ok((snapshot, runtime, keys))
        },
        |result| Message::RedisTool(RedisToolMessage::SelectConnectionCompleted(result)),
    )
}

/// 处理 `select_connection_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn select_connection_completed(
    app: &mut App,
    result: Result<
        (
            super::RedisToolGatewaySnapshot,
            Option<Result<super::RedisRuntimeOverview, String>>,
            Option<Result<super::RedisKeyPage, String>>,
        ),
        String,
    >,
) -> Task<Message> {
    match result {
        Ok((snapshot, runtime_result, key_result)) => {
            app.redis_tool.finish_gateway_request();
            app.redis_tool.apply_gateway_snapshot(snapshot);
            let mut first_error = None;
            if let Some(runtime_result) = runtime_result {
                match runtime_result {
                    Ok(runtime) => {
                        app.redis_tool.apply_runtime_overview(runtime);
                    }
                    Err(error) => {
                        first_error = Some(error);
                    }
                }
            }

            if let Some(key_result) = key_result {
                match key_result {
                    Ok(page) => {
                        app.redis_tool.apply_key_page(page, false);
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                    }
                }
            }

            if let Some(error) = first_error {
                app.redis_tool.gateway_error = Some(error);
                Task::none()
            } else {
                let message = if app.redis_tool.detail_tab.requires_runtime()
                    || app.redis_tool.detail_tab.requires_keys()
                {
                    format!("已展开连接并加载 {} 标签", app.redis_tool.detail_tab.title())
                } else {
                    "已展开连接".to_string()
                };
                notify_success(app, &message);
                Task::batch(vec![clear_notification_task()])
            }
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `detail_tab_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn detail_tab_changed(app: &mut App, value: RedisDetailTab) -> Task<Message> {
    if app.redis_tool.detail_tab == value || app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    app.redis_tool.detail_tab = value;

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        return Task::none();
    };

    if app.redis_tool.has_detail_tab_data_for_selected(value) {
        return Task::none();
    }

    match value {
        RedisDetailTab::Overview | RedisDetailTab::Info => {
            start_runtime_reload(app, selected_id, "加载 Redis 信息", None)
        }
        RedisDetailTab::Keys => start_key_page_reload(app, selected_id, false, "加载键树"),
        RedisDetailTab::Analysis => {
            let Some(selected_key) = app.redis_tool.selected_key.clone() else {
                return Task::none();
            };
            start_key_analysis_reload(app, selected_id, selected_key, "加载 Key 内容")
        }
        RedisDetailTab::Command | RedisDetailTab::Connection => Task::none(),
    }
}

/// 处理 `refresh_selected_runtime` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn refresh_selected_runtime(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    start_runtime_reload(app, selected_id, "加载 Redis 信息", Some("Redis 信息已刷新".to_string()))
}

/// 处理 `reload_selected_keys` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn reload_selected_keys(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    start_key_page_reload(app, selected_id, false, "加载键树")
}

/// 处理 `load_more_keys` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn load_more_keys(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() || !app.redis_tool.key_browser_has_more {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    start_key_page_reload(app, selected_id, true, "加载更多键")
}

/// 处理 `select_key` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn select_key(app: &mut App, key: String) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    app.redis_tool.select_key(key.clone());
    app.redis_tool.detail_tab = RedisDetailTab::Analysis;
    if app.redis_tool.has_key_analysis_for_selected() {
        return Task::none();
    }

    start_key_analysis_reload(app, selected_id, key, "加载 Key 内容")
}

/// 处理 `refresh_selected_key_analysis` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn refresh_selected_key_analysis(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };
    let Some(selected_key) = app.redis_tool.selected_key.clone() else {
        app.redis_tool.fail_gateway_request("请先在键树中选择一个 Key".to_string());
        return Task::none();
    };

    start_key_analysis_reload(app, selected_id, selected_key, "刷新 Key 内容")
}

/// 处理 `key_analysis_loaded` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn key_analysis_loaded(
    app: &mut App,
    connection_id: String,
    key: String,
    result: Result<super::RedisKeyAnalysis, String>,
) -> Task<Message> {
    match result {
        Ok(analysis) => {
            app.redis_tool.finish_gateway_request();
            if app.redis_tool.selected_connection_id.as_deref() == Some(connection_id.as_str())
                && app.redis_tool.selected_key.as_deref() == Some(key.as_str())
            {
                app.redis_tool.apply_key_analysis(analysis);
            }
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `runtime_loaded` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn runtime_loaded(
    app: &mut App,
    connection_id: String,
    success_message: Option<String>,
    result: Result<super::RedisRuntimeOverview, String>,
) -> Task<Message> {
    match result {
        Ok(runtime) => {
            app.redis_tool.finish_gateway_request();
            if app.redis_tool.selected_connection_id.as_deref() == Some(connection_id.as_str()) {
                app.redis_tool.apply_runtime_overview(runtime);
            }
            if let Some(message) = success_message {
                notify_success(app, &message);
                return Task::batch(vec![clear_notification_task()]);
            }
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `key_page_loaded` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn key_page_loaded(
    app: &mut App,
    connection_id: String,
    append: bool,
    result: Result<super::RedisKeyPage, String>,
) -> Task<Message> {
    match result {
        Ok(page) => {
            app.redis_tool.finish_gateway_request();
            if app.redis_tool.selected_connection_id.as_deref() == Some(connection_id.as_str()) {
                app.redis_tool.apply_key_page(page, append);
            }
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `key_browser_pattern_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn key_browser_pattern_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.key_browser_pattern = value;
    Task::none()
}

/// 处理 `toggle_key_tree_path` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn toggle_key_tree_path(app: &mut App, path: String) -> Task<Message> {
    app.redis_tool.toggle_key_tree_path(path);
    Task::none()
}

/// 处理 `info_filter_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn info_filter_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.info_filter = value;
    Task::none()
}

/// 处理 `history_previous_page` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn history_previous_page(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() || app.redis_tool.history_page_offset == 0 {
        return Task::none();
    }

    let offset =
        app.redis_tool.history_page_offset.saturating_sub(app.redis_tool.history_page_limit.max(1));
    start_snapshot_reload(app, offset, "加载历史", None)
}

/// 处理 `history_next_page` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn history_next_page(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() || !app.redis_tool.history_has_more {
        return Task::none();
    }

    let offset =
        app.redis_tool.history_page_offset.saturating_add(app.redis_tool.history_page_limit.max(1));
    start_snapshot_reload(app, offset, "加载历史", None)
}

/// 处理 `history_filter_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn history_filter_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.history_filter = value;
    if app.redis_tool.is_gateway_loading() {
        Task::none()
    } else {
        start_snapshot_reload(app, 0, "筛选历史", None)
    }
}

/// 处理 `history_only_write_toggled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn history_only_write_toggled(app: &mut App, value: bool) -> Task<Message> {
    app.redis_tool.history_only_write = value;
    if app.redis_tool.is_gateway_loading() {
        Task::none()
    } else {
        start_snapshot_reload(app, 0, "筛选历史", None)
    }
}

/// 处理 `snapshot_loaded` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn snapshot_loaded(
    app: &mut App,
    success_message: Option<String>,
    result: Result<super::RedisToolGatewaySnapshot, String>,
) -> Task<Message> {
    match result {
        Ok(snapshot) => {
            app.redis_tool.finish_gateway_request();
            app.redis_tool.apply_gateway_snapshot(snapshot);
            if let Some(message) = success_message {
                notify_success(app, &message);
                return Task::batch(vec![clear_notification_task()]);
            }
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `clear_notification` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn clear_notification(app: &mut App) -> Task<Message> {
    app.redis_tool.notification = None;
    Task::none()
}

/// 处理 `clear_gateway_error` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn clear_gateway_error(app: &mut App) -> Task<Message> {
    app.redis_tool.clear_gateway_error();
    Task::none()
}
