//! 思维导图 Markdown 导入面板渲染逻辑，负责导入输入区和反馈状态。

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::{Space, button, column, container, row, stack, text, text_editor};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// 构建或更新 with markdown import overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn with_markdown_import_overlay<'a>(
    tab: &'a MindMapTab,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    let close_btn_style = |theme: &Theme, status: iced::widget::button::Status| {
        let palette = theme.extended_palette();
        let background = match status {
            iced::widget::button::Status::Pressed => Some(palette.background.strong.color),
            iced::widget::button::Status::Hovered => Some(palette.background.weak.color),
            _ => None,
        };
        iced::widget::button::Style {
            background: background.map(Background::Color),
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
            text_color: theme.palette().text.scale_alpha(0.60),
            ..Default::default()
        }
    };

    let modal: Element<'_, Message> = container(
        column![
            row![
                text("Markdown 大纲").size(14),
                Space::new().width(Length::Fill),
                button(text("关闭").size(12))
                    .style(close_btn_style)
                    .on_press(Message::MindMapTool(MindMapMessage::ToggleMarkdownImport))
                    .padding([4, 8]),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            container(
                text_editor(&tab.markdown_import_editor)
                    .height(Length::Fill)
                    .on_action(|action| {
                        Message::MindMapTool(MindMapMessage::MarkdownImportEditorAction(action))
                    })
                    .padding(12),
            )
            .style(|theme: &Theme| iced::widget::container::Style {
                background: Some(theme.extended_palette().background.base.color.into()),
                border: Border {
                    width: 1.0,
                    color: theme.extended_palette().background.weak.color,
                    radius: 12.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(12)
        .padding(14),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &Theme| iced::widget::container::Style {
        background: Some(theme.extended_palette().background.base.color.into()),
        ..Default::default()
    })
    .into();

    stack(vec![base, modal]).into()
}
