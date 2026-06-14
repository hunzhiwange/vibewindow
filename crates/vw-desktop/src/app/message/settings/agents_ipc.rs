//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;
fn persist_agents_ipc_settings(app: &mut App) -> Task<Message> {
    let s = &app.agents_ipc_settings;
    let enabled = s.enabled;
    let staleness_secs = s.staleness_secs.clamp(1, 86_400);
    let db_path = s.db_path_input.trim().to_string();

    crate::app::update_agents_ipc_config_async(move |agents_ipc| {
        agents_ipc.enabled = enabled;
        agents_ipc.db_path =
            if db_path.is_empty() { vw_config_types::paths::agents_ipc_db_path() } else { db_path };
        agents_ipc.staleness_secs = staleness_secs;
    })
}

#[cfg(test)]
#[path = "agents_ipc_tests.rs"]
mod agents_ipc_tests;

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::AgentsIpcEnabledToggled(v) => {
            app.agents_ipc_settings.enabled = v;
            app.agents_ipc_settings.save_error = None;
            persist_agents_ipc_settings(app)
        }
        SettingsMessage::AgentsIpcDbPathChanged(v) => {
            app.agents_ipc_settings.db_path_input = v;
            app.agents_ipc_settings.save_error = None;
            persist_agents_ipc_settings(app)
        }
        SettingsMessage::AgentsIpcStalenessSecsChanged(v) => {
            app.agents_ipc_settings.staleness_secs = v.clamp(1, 86_400);
            app.agents_ipc_settings.save_error = None;
            persist_agents_ipc_settings(app)
        }
        SettingsMessage::AgentsIpcSave => {
            app.agents_ipc_settings.save_error = None;
            persist_agents_ipc_settings(app)
        }
        SettingsMessage::AgentsIpcHelpOpen => {
            app.agents_ipc_settings.show_help_modal = true;
            Task::none()
        }
        SettingsMessage::AgentsIpcHelpClose => {
            app.agents_ipc_settings.show_help_modal = false;
            Task::none()
        }
        _ => Task::none(),
    }
}
