//! 应用界面状态同步相关的 lsp.rs 模块。
//!
//! 该模块封装 UI 层对外部服务或子系统的轻量桥接逻辑，保持界面状态更新路径集中、可追踪。

use crate::app::{App, Message, message};
use iced::widget::{Space, container};
use iced::{Element, Length};

/// 公开的 view_lsp_panel 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn view_lsp_panel() -> Element<'static, Message> {
    container(Space::new().width(Length::Shrink).height(Length::Shrink)).into()
}

#[cfg(not(target_arch = "wasm32"))]
fn map_lsp_overlay_message(msg: iced_code_editor::LspOverlayMessage) -> Message {
    use iced_code_editor::LspOverlayMessage;
    match msg {
        LspOverlayMessage::HoverEntered => Message::Preview(message::PreviewMessage::LspHoverEntered),
        LspOverlayMessage::HoverExited => Message::Preview(message::PreviewMessage::LspHoverExited),
        LspOverlayMessage::CompletionSelected(index) => {
            Message::Preview(message::PreviewMessage::LspCompletionSelected(index))
        }
        LspOverlayMessage::CompletionClosed => Message::Preview(message::PreviewMessage::LspCompletionClosed),
        LspOverlayMessage::CompletionNavigateUp => {
            Message::Preview(message::PreviewMessage::LspCompletionNavigateUp)
        }
        LspOverlayMessage::CompletionNavigateDown => {
            Message::Preview(message::PreviewMessage::LspCompletionNavigateDown)
        }
        LspOverlayMessage::CompletionConfirm => {
            Message::Preview(message::PreviewMessage::LspCompletionConfirm)
        }
    }
}

/// 公开的 view_lsp_overlay 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn view_lsp_overlay(app: &App) -> Element<'_, Message> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if app.lsp_overlay_path.as_ref() != app.active_preview_path.as_ref() {
            return container(Space::new().width(Length::Shrink).height(Length::Shrink)).into();
        }

        let Some(path) = app.active_preview_path.as_ref() else {
            return container(Space::new().width(Length::Shrink).height(Length::Shrink)).into();
        };
        let Some(tab) = app.preview_tabs.iter().find(|t| &t.path == path) else {
            return container(Space::new().width(Length::Shrink).height(Length::Shrink)).into();
        };

        return iced_code_editor::view_lsp_overlay(
            &app.lsp_overlay,
            &tab.editor.inner,
            app.effective_editor_theme_ref(),
            app.current_font_size,
            app.current_line_height,
            map_lsp_overlay_message,
        );
    }

    #[cfg(target_arch = "wasm32")]
    container(Space::new().width(Length::Shrink).height(Length::Shrink)).into()
}

#[cfg(test)]
#[path = "lsp_tests.rs"]
mod lsp_tests;
