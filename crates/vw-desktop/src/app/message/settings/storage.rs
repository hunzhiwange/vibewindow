//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message};
use iced::Task;

use super::messages::{SettingsMessage, StorageMessage};

fn persist_storage_settings(app: &mut App) -> Result<Task<Message>, String> {
    let s = &app.storage_settings;
    let provider = s.provider.trim().to_string();
    let db_url = s.db_url_input.trim().to_string();
    let schema = {
        let value = s.schema.trim();
        if value.is_empty() { "public".to_string() } else { value.to_string() }
    };
    let table = {
        let value = s.table.trim();
        if value.is_empty() { "memories".to_string() } else { value.to_string() }
    };
    let connect_timeout_secs = if s.connect_timeout_secs_input.trim().is_empty() {
        None
    } else {
        Some(
            s.connect_timeout_secs_input
                .trim()
                .parse::<u64>()
                .map_err(|_| "connect_timeout_secs 必须是整数秒或留空".to_string())?,
        )
    };
    let tls = s.tls;

    Ok(crate::app::update_storage_config_async(move |storage| {
        storage.provider.config.provider = provider;
        storage.provider.config.db_url = if db_url.is_empty() { None } else { Some(db_url) };
        storage.provider.config.schema = schema;
        storage.provider.config.table = table;
        storage.provider.config.connect_timeout_secs = connect_timeout_secs;
        storage.provider.config.tls = tls;
    }))
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::Storage(storage_message) => {
            match storage_message {
                StorageMessage::ProviderChanged(value) => {
                    app.storage_settings.provider = value;
                }
                StorageMessage::DbUrlChanged(value) => {
                    app.storage_settings.db_url_input = value;
                }
                StorageMessage::SchemaChanged(value) => {
                    app.storage_settings.schema = value;
                }
                StorageMessage::TableChanged(value) => {
                    app.storage_settings.table = value;
                }
                StorageMessage::ConnectTimeoutSecsChanged(value) => {
                    app.storage_settings.connect_timeout_secs_input = value;
                }
                StorageMessage::TlsToggled(value) => {
                    app.storage_settings.tls = value;
                }
                StorageMessage::Save => {}
            }

            match persist_storage_settings(app) {
                Ok(task) => {
                    app.storage_settings.save_error = None;
                    task
                }
                Err(err) => {
                    app.storage_settings.save_error = Some(err);
                    Task::none()
                }
            }
        }
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "storage_tests.rs"]
mod storage_tests;
