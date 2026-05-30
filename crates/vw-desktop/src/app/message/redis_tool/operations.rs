//! 处理 Redis 工具的保存、删除、测试、导入导出、创建键和命令执行操作。

use super::draft::{build_draft_uri, draft_to_upsert_body};
use super::helpers::{
    clear_notification_task, current_default_load_count, current_history_query, notify_success,
};
use super::{App, Message, RedisToolMessage, Task};
#[cfg(not(target_arch = "wasm32"))]
use super::{GatewayRedisConfigBundle, redis_export_async, redis_import_async};
use super::{
    RedisToolGatewaySnapshot, load_redis_tool_snapshot_async, redis_command_execute_async,
    redis_connection_create_async, redis_connection_delete_async,
    redis_connection_key_create_async, redis_connection_test_async, redis_connection_update_async,
    redis_settings_update_async,
};

/// 处理 `save_draft` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn save_draft(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let body = match draft_to_upsert_body(&app.redis_tool.draft) {
        Ok(body) => body,
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            return Task::none();
        }
    };

    let selected_id = if app.redis_tool.draft_is_new {
        None
    } else {
        app.redis_tool.selected_connection_id.clone()
    };

    app.redis_tool.begin_gateway_request("保存连接");
    let query = current_history_query(app, Some(0));
    Task::perform(
        async move {
            let saved_connection_id = if let Some(connection_id) = selected_id {
                redis_connection_update_async(&connection_id, &body).await?.id
            } else {
                redis_connection_create_async(&body).await?.id
            };
            let mut snapshot = load_redis_tool_snapshot_async(query).await?;
            snapshot.persisted_state.selected_connection_id = Some(saved_connection_id);
            Ok(snapshot)
        },
        |result| Message::RedisTool(RedisToolMessage::SaveDraftCompleted(result)),
    )
}

#[cfg(test)]
#[path = "operations_tests.rs"]
mod operations_tests;

/// 处理 `save_draft_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn save_draft_completed(
    app: &mut App,
    result: Result<RedisToolGatewaySnapshot, String>,
) -> Task<Message> {
    match result {
        Ok(snapshot) => {
            app.redis_tool.finish_gateway_request();
            app.redis_tool.apply_gateway_snapshot(snapshot);
            app.redis_tool.close_connection_modal();
            app.redis_tool.connection_search_query.clear();
            notify_success(app, "连接配置已保存");
            Task::batch(vec![clear_notification_task()])
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `delete_selected` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn delete_selected(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        return Task::none();
    };

    app.redis_tool.begin_gateway_request("删除连接");
    let query = current_history_query(app, Some(0));
    Task::perform(
        async move {
            redis_connection_delete_async(&selected_id).await.map(|_| ())?;
            load_redis_tool_snapshot_async(query).await
        },
        |result| {
            Message::RedisTool(RedisToolMessage::SnapshotLoaded {
                success_message: Some("连接配置已删除".to_string()),
                result,
            })
        },
    )
}

/// 处理 `test_selected` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn test_selected(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    app.redis_tool.begin_gateway_request("测试连接");
    let query = current_history_query(app, Some(0));
    Task::perform(
        async move {
            let response = redis_connection_test_async(&selected_id).await?;
            let snapshot = load_redis_tool_snapshot_async(query).await?;
            Ok((response, snapshot))
        },
        |result| Message::RedisTool(RedisToolMessage::TestSelectedCompleted(result)),
    )
}

/// 处理 `test_selected_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn test_selected_completed(
    app: &mut App,
    result: Result<(super::GatewayRedisConnectionTestResponse, RedisToolGatewaySnapshot), String>,
) -> Task<Message> {
    match result {
        Ok((response, snapshot)) => {
            app.redis_tool.finish_gateway_request();
            app.redis_tool.apply_gateway_snapshot(snapshot);
            notify_success(
                app,
                &format!("连接测试成功：{}（{} ms）", response.message, response.latency_ms),
            );
            Task::batch(vec![clear_notification_task()])
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `copy_selected_uri` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn copy_selected_uri(app: &mut App) -> Task<Message> {
    match build_draft_uri(&app.redis_tool.draft) {
        Ok(uri) => {
            notify_success(app, "连接 URI 已复制");
            Task::batch(vec![iced::clipboard::write(uri), clear_notification_task()])
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `export_configs` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn export_configs(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    app.redis_tool.begin_gateway_request("导出配置");
    let query = current_history_query(app, Some(0));
    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("redis-connections.json")
                    .save_file()
                    .await;

                let Some(file) = file else {
                    return Ok(None);
                };

                let bundle = redis_export_async().await?;
                let content =
                    serde_json::to_string_pretty(&bundle).map_err(|error| error.to_string())?;
                file.write(content.as_bytes()).await.map_err(|error| error.to_string())?;
                let snapshot = load_redis_tool_snapshot_async(query).await?;
                Ok(Some(snapshot))
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = query;
                Ok(None)
            }
        },
        |result| Message::RedisTool(RedisToolMessage::ExportCompleted(result)),
    )
}

/// 处理 `export_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn export_completed(
    app: &mut App,
    result: Result<Option<RedisToolGatewaySnapshot>, String>,
) -> Task<Message> {
    match result {
        Ok(Some(snapshot)) => {
            app.redis_tool.finish_gateway_request();
            app.redis_tool.apply_gateway_snapshot(snapshot);
            notify_success(app, "连接配置已导出");
            Task::batch(vec![clear_notification_task()])
        }
        Ok(None) => {
            app.redis_tool.finish_gateway_request();
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `import_configs` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn import_configs(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    app.redis_tool.begin_gateway_request("导入配置");
    let query = current_history_query(app, Some(0));
    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file =
                    rfd::AsyncFileDialog::new().add_filter("JSON", &["json"]).pick_file().await;

                let Some(file) = file else {
                    return Ok(None);
                };

                let data = file.read().await;
                let bundle = serde_json::from_slice::<GatewayRedisConfigBundle>(&data)
                    .map_err(|error| error.to_string())?;
                redis_import_async(&bundle).await?;
                let snapshot = load_redis_tool_snapshot_async(query).await?;
                Ok(Some(snapshot))
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = query;
                Ok(None)
            }
        },
        |result| Message::RedisTool(RedisToolMessage::ImportCompleted(result)),
    )
}

/// 处理 `import_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn import_completed(
    app: &mut App,
    result: Result<Option<RedisToolGatewaySnapshot>, String>,
) -> Task<Message> {
    match result {
        Ok(Some(snapshot)) => {
            app.redis_tool.finish_gateway_request();
            app.redis_tool.apply_gateway_snapshot(snapshot);
            notify_success(app, "连接配置已导入");
            Task::batch(vec![clear_notification_task()])
        }
        Ok(None) => {
            app.redis_tool.finish_gateway_request();
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `default_load_count_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn default_load_count_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.default_load_count_input = value;
    Task::none()
}

/// 处理 `save_default_load_count` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn save_default_load_count(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let parsed = match app.redis_tool.default_load_count_input.trim().parse::<u32>() {
        Ok(value) => value.clamp(1, 10_000),
        Err(_) => {
            app.redis_tool.fail_gateway_request("默认加载数量必须是 1-10000 的整数".to_string());
            return Task::none();
        }
    };

    app.redis_tool.default_load_count_input = parsed.to_string();
    app.redis_tool.begin_gateway_request("保存默认加载数量");
    let query = current_history_query(app, Some(0));
    Task::perform(
        async move {
            redis_settings_update_async(parsed).await?;
            load_redis_tool_snapshot_async(query).await
        },
        |result| {
            Message::RedisTool(RedisToolMessage::SnapshotLoaded {
                success_message: Some("默认加载数量已更新".to_string()),
                result,
            })
        },
    )
}

/// 处理 `increase_default_load_count` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn increase_default_load_count(app: &mut App) -> Task<Message> {
    let next = current_default_load_count(app).saturating_add(100).min(10_000);
    app.redis_tool.default_load_count_input = next.to_string();
    Task::none()
}

/// 处理 `decrease_default_load_count` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn decrease_default_load_count(app: &mut App) -> Task<Message> {
    let next = current_default_load_count(app).saturating_sub(100).max(1);
    app.redis_tool.default_load_count_input = next.to_string();
    Task::none()
}

/// 处理 `create_key_name_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn create_key_name_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.create_key_draft.name = value;
    Task::none()
}

/// 处理 `create_key_type_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn create_key_type_changed(
    app: &mut App,
    value: super::RedisKeyValueKind,
) -> Task<Message> {
    app.redis_tool.create_key_draft.key_type = value;
    Task::none()
}

/// 处理 `confirm_create_key` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn confirm_create_key(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    let key = app.redis_tool.create_key_draft.name.trim().to_string();
    if key.is_empty() {
        app.redis_tool.fail_gateway_request("请输入 Key 名称".to_string());
        return Task::none();
    }

    let key_type = app.redis_tool.create_key_draft.key_type;
    app.redis_tool.begin_gateway_request("创建 Key");
    let request_connection_id = selected_id.clone();
    let request_key = key.clone();
    let response_key = key.clone();
    Task::perform(
        async move {
            redis_connection_key_create_async(
                &request_connection_id,
                &request_key,
                key_type.gateway_value(),
            )
            .await
        },
        move |result| {
            Message::RedisTool(RedisToolMessage::CreateKeyCompleted {
                connection_id: selected_id,
                key: response_key,
                result,
            })
        },
    )
}

/// 处理 `create_key_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn create_key_completed(
    app: &mut App,
    connection_id: String,
    key: String,
    result: Result<super::RedisKeyAnalysis, String>,
) -> Task<Message> {
    match result {
        Ok(analysis) => {
            app.redis_tool.finish_gateway_request();
            if app.redis_tool.selected_connection_id.as_deref() == Some(connection_id.as_str()) {
                app.redis_tool.key_browser_connection_id = Some(connection_id);
                app.redis_tool.include_key_browser_item(key);
                app.redis_tool.close_create_key_modal();
                app.redis_tool.apply_key_analysis(analysis);
                app.redis_tool.detail_tab = super::RedisDetailTab::Analysis;
            }
            notify_success(app, "Key 已创建并进入内容分析");
            Task::batch(vec![clear_notification_task()])
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}

/// 处理 `command_input_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn command_input_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.command_input = value;
    Task::none()
}

/// 处理 `run_command` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn run_command(app: &mut App) -> Task<Message> {
    if app.redis_tool.is_gateway_loading() {
        return Task::none();
    }

    let Some(selected_id) = app.redis_tool.selected_connection_id.clone() else {
        app.redis_tool.fail_gateway_request("请先选择已保存的连接".to_string());
        return Task::none();
    };

    let command = app.redis_tool.command_input.trim().to_string();
    if command.is_empty() {
        app.redis_tool.fail_gateway_request("请输入 Redis 命令".to_string());
        return Task::none();
    }

    app.redis_tool.begin_gateway_request("执行 Redis 命令");
    let request_connection_id = selected_id.clone();
    Task::perform(
        async move { redis_command_execute_async(&request_connection_id, &command).await },
        move |result| {
            Message::RedisTool(RedisToolMessage::CommandCompleted {
                connection_id: selected_id,
                result,
            })
        },
    )
}

/// 处理 `command_completed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn command_completed(
    app: &mut App,
    connection_id: String,
    result: Result<super::RedisCommandOutputEntry, String>,
) -> Task<Message> {
    match result {
        Ok(entry) => {
            app.redis_tool.finish_gateway_request();
            if app.redis_tool.selected_connection_id.as_deref() == Some(connection_id.as_str()) {
                app.redis_tool.push_command_output(entry);
            }
            Task::none()
        }
        Err(error) => {
            app.redis_tool.fail_gateway_request(error);
            Task::none()
        }
    }
}
