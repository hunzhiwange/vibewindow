//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message};
use iced::Task;

use super::messages::{HooksMessage, SettingsMessage};
fn persist_hooks_settings(app: &mut App) -> Task<Message> {
    let s = &app.hooks_settings;
    let enabled = s.enabled;
    let command_logger = s.command_logger;

    crate::app::update_hooks_config_async(move |hooks| {
        hooks.enabled = enabled;
        hooks.builtin.command_logger = command_logger;
    })
}

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod hooks_tests;

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::Hooks(hooks_message) => {
            match hooks_message {
                HooksMessage::EnabledToggled(value) => {
                    app.hooks_settings.enabled = value;
                }
                HooksMessage::CommandLoggerToggled(value) => {
                    app.hooks_settings.command_logger = value;
                }
            }

            app.hooks_settings.save_error = None;
            persist_hooks_settings(app)
        }
        _ => Task::none(),
    }
}
