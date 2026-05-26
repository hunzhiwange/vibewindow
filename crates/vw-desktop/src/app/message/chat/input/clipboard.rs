//! 处理聊天输入区的局部消息。
//! 本模块将编辑器操作、文件检索和工具细节限制在输入面板边界内。

use super::super::{ChatMessage, ClipboardPastePayload};
use crate::app::message::project::helpers::persist_clipboard_image_attachment;
use crate::app::Message;
use iced::Task;

/// 模块内可见函数，执行 read_clipboard_for_input 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn read_clipboard_for_input() -> Task<Message> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Task::perform(async move { read_native_clipboard_payload() }, |payload| {
            Message::Chat(ChatMessage::ClipboardPasteResolved(payload))
        })
    }

    #[cfg(target_arch = "wasm32")]
    {
        iced::clipboard::read().map(|content| {
            Message::Chat(ChatMessage::ClipboardPasteResolved(match content {
                Some(content) if !content.is_empty() => ClipboardPastePayload::Text(content),
                _ => ClipboardPastePayload::Empty,
            }))
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_native_clipboard_payload() -> ClipboardPastePayload {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(error) => {
            return ClipboardPastePayload::Error(format!("读取系统剪贴板失败：{}", error));
        }
    };

    if let Ok(image) = clipboard.get_image() {
        let width = match u32::try_from(image.width) {
            Ok(width) => width,
            Err(_) => {
                return ClipboardPastePayload::Error("剪贴板图片宽度超出范围".to_string());
            }
        };
        let height = match u32::try_from(image.height) {
            Ok(height) => height,
            Err(_) => {
                return ClipboardPastePayload::Error("剪贴板图片高度超出范围".to_string());
            }
        };

        return match persist_clipboard_image_attachment(width, height, image.bytes.as_ref()) {
            Ok(path) => ClipboardPastePayload::AttachmentPath(path),
            Err(error) => ClipboardPastePayload::Error(error),
        };
    }

    match clipboard.get_text() {
        Ok(text) if !text.is_empty() => ClipboardPastePayload::Text(text),
        Ok(_) => ClipboardPastePayload::Empty,
        Err(_) => ClipboardPastePayload::Empty,
    }
}
#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod clipboard_tests;
