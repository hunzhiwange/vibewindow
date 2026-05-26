//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message, update_multimodal_config_async};
use iced::Task;

use super::messages::{MultimodalMessage, SettingsMessage};

fn persist_multimodal_settings(app: &mut App) -> Task<Message> {
    let s = &app.multimodal_settings;
    let max_images = s.max_images.clamp(1, 16) as usize;
    let max_image_size_mb = s.max_image_size_mb.clamp(1, 20) as usize;
    let allow_remote_fetch = s.allow_remote_fetch;

    update_multimodal_config_async(move |multimodal| {
        multimodal.max_images = max_images;
        multimodal.max_image_size_mb = max_image_size_mb;
        multimodal.allow_remote_fetch = allow_remote_fetch;
    })
}

#[cfg(test)]
#[path = "multimodal_tests.rs"]
mod multimodal_tests;

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let SettingsMessage::Multimodal(message) = message else {
        return Task::none();
    };

    match message {
        MultimodalMessage::MaxImagesChanged(value) => {
            app.multimodal_settings.max_images = value.clamp(1, 16)
        }
        MultimodalMessage::MaxImageSizeMbChanged(value) => {
            app.multimodal_settings.max_image_size_mb = value.clamp(1, 20)
        }
        MultimodalMessage::AllowRemoteFetchToggled(value) => {
            app.multimodal_settings.allow_remote_fetch = value
        }
    }

    app.multimodal_settings.save_error = None;
    persist_multimodal_settings(app)
}
