//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::message::settings::{SettingsMessage, SopMessage};
use crate::app::{App, Message, update_sop_config_async};
use iced::Task;
use vw_config_types::automation::SopExecutionMode;

fn normalize_execution_mode(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "autonomous" => "autonomous".to_string(),
        _ => "supervised".to_string(),
    }
}

fn trim_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

fn persist_sop_settings(app: &mut App) -> Task<Message> {
    let mode = normalize_execution_mode(&app.sop_settings.default_execution_mode);
    let execution_mode =
        if mode == "autonomous" { SopExecutionMode::Auto } else { SopExecutionMode::Supervised };
    let sops_dir = trim_optional(&app.sop_settings.sops_dir_input);
    let max_finished_runs = app.sop_settings.max_finished_runs.min(100_000);
    let max_concurrent_total = app.sop_settings.max_concurrent_total.clamp(1, 1_000);
    let approval_timeout_secs = app.sop_settings.approval_timeout_secs.min(86_400);

    update_sop_config_async(move |sop| {
        sop.sops_dir = sops_dir;
        sop.default_execution_mode = execution_mode;
        sop.max_finished_runs = max_finished_runs;
        sop.max_concurrent_total = max_concurrent_total;
        sop.approval_timeout_secs = approval_timeout_secs;
    })
}

fn pick_sops_dir_task() -> Task<Message> {
    Task::perform(
        async move {
            #[cfg(target_arch = "wasm32")]
            {
                None
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_string_lossy().to_string())
            }
        },
        |picked| Message::Settings(SettingsMessage::Sop(SopMessage::SopsDirPicked(picked))),
    )
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let SettingsMessage::Sop(message) = message else {
        return Task::none();
    };

    match message {
        SopMessage::SopsDirChanged(value) => {
            app.sop_settings.sops_dir_input = value;
            app.sop_settings.save_error = None;
            return persist_sop_settings(app);
        }
        SopMessage::SopsDirPickFolder => {
            return pick_sops_dir_task();
        }
        SopMessage::SopsDirPicked(picked) => {
            if let Some(path) = picked {
                app.sop_settings.sops_dir_input = path;
                app.sop_settings.save_error = None;
                return persist_sop_settings(app);
            }
        }
        SopMessage::DefaultExecutionModeChanged(value) => {
            app.sop_settings.default_execution_mode = normalize_execution_mode(&value);
            app.sop_settings.save_error = None;
            return persist_sop_settings(app);
        }
        SopMessage::MaxFinishedRunsChanged(value) => {
            app.sop_settings.max_finished_runs = value.min(100_000);
            app.sop_settings.save_error = None;
            return persist_sop_settings(app);
        }
        SopMessage::MaxConcurrentTotalChanged(value) => {
            app.sop_settings.max_concurrent_total = value.clamp(1, 1_000);
            app.sop_settings.save_error = None;
            return persist_sop_settings(app);
        }
        SopMessage::ApprovalTimeoutSecsChanged(value) => {
            app.sop_settings.approval_timeout_secs = value.min(86_400);
            app.sop_settings.save_error = None;
            return persist_sop_settings(app);
        }
    }
    Task::none()
}
#[cfg(test)]
#[path = "sop_tests.rs"]
mod sop_tests;
