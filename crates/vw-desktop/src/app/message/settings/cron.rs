//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;

fn persist_cron_settings(app: &mut App) -> Task<Message> {
    let enabled = app.cron_settings.enabled;
    let max_run_history = app.cron_settings.max_run_history.clamp(1, 10_000);
    crate::app::update_cron_config_async(move |cron| {
        cron.enabled = enabled;
        cron.max_run_history = max_run_history;
    })
}

#[cfg(test)]
#[path = "cron_tests.rs"]
mod cron_tests;

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::CronEnabledToggled(v) => {
            app.cron_settings.enabled = v;
            persist_cron_settings(app)
        }
        SettingsMessage::CronMaxRunHistoryChanged(v) => {
            app.cron_settings.max_run_history = v.clamp(1, 10_000);
            persist_cron_settings(app)
        }
        SettingsMessage::CronSave => persist_cron_settings(app),
        SettingsMessage::CronHelpOpen => {
            app.cron_settings.show_help_modal = true;
            Task::none()
        }
        SettingsMessage::CronHelpClose => {
            app.cron_settings.show_help_modal = false;
            Task::none()
        }
        _ => Task::none(),
    }
}
