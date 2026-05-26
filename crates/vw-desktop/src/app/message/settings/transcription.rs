//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;
fn persist_transcription_settings(app: &mut App) -> Task<Message> {
    let s = &app.transcription_settings;
    let enabled = s.enabled;
    let max_duration_secs = s.max_duration_secs.clamp(1, 3600);
    let api_url = s.api_url.trim().to_string();
    let model = s.model.trim().to_string();
    let language = s.language.trim().to_string();

    crate::app::config::update_transcription_config_async(move |transcription| {
        transcription.enabled = enabled;
        transcription.api_url = if api_url.is_empty() {
            "https://api.groq.com/openai/v1/audio/transcriptions".to_string()
        } else {
            api_url
        };
        transcription.model =
            if model.is_empty() { "whisper-large-v3-turbo".to_string() } else { model };
        transcription.language = if language.is_empty() { None } else { Some(language) };
        transcription.max_duration_secs = max_duration_secs;
    })
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::TranscriptionEnabledToggled(v) => {
            app.transcription_settings.enabled = v;
            app.transcription_settings.save_error = None;
            persist_transcription_settings(app)
        }
        SettingsMessage::TranscriptionApiUrlChanged(v) => {
            app.transcription_settings.api_url = v;
            app.transcription_settings.save_error = None;
            persist_transcription_settings(app)
        }
        SettingsMessage::TranscriptionModelChanged(v) => {
            app.transcription_settings.model = v;
            app.transcription_settings.save_error = None;
            persist_transcription_settings(app)
        }
        SettingsMessage::TranscriptionLanguageChanged(v) => {
            app.transcription_settings.language = v;
            app.transcription_settings.save_error = None;
            persist_transcription_settings(app)
        }
        SettingsMessage::TranscriptionMaxDurationSecsChanged(v) => {
            app.transcription_settings.max_duration_secs = v.clamp(1, 3600);
            app.transcription_settings.save_error = None;
            persist_transcription_settings(app)
        }
        SettingsMessage::TranscriptionHelpOpen => {
            app.transcription_settings.show_help_modal = true;
            Task::none()
        }
        SettingsMessage::TranscriptionHelpClose => {
            app.transcription_settings.show_help_modal = false;
            Task::none()
        }
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "transcription_tests.rs"]
mod transcription_tests;
