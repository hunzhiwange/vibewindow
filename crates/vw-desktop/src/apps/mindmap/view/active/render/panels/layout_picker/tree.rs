//! 思维导图树形布局选择面板，负责树形与组织图格式的可视化选择。

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapTab, TreeLayoutFormat};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::super::super::super::previews::TreeLayoutFormatPreview;

/// 构建或更新 tree layout picker 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(in super::super) fn tree_layout_picker(
    tab: &MindMapTab,
    desc_w: f32,
) -> Option<Element<'static, Message>> {
    let formats = [
        TreeLayoutFormat::SymmetricSplit,
        TreeLayoutFormat::FanDown,
        TreeLayoutFormat::LeftAligned,
        TreeLayoutFormat::RightAligned,
    ];

    let card_gap = 10.0;
    let card_w = ((desc_w - card_gap * 1.0) / 2.0).max(160.0);
    let card_h = 66.0;

    let card = |f: TreeLayoutFormat| {
        let active = tab.tree_layout_format == f;
        let preview: Element<'static, Message> = iced::widget::canvas(TreeLayoutFormatPreview {
            format: f,
            color: Color::from_rgba8(0, 0, 0, 0.68),
        })
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .into();

        button(
            container(
                column![
                    preview,
                    container(text(f.label()).size(11))
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                ]
                .spacing(6),
            )
            .padding([8, 10])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(card_w))
        .height(Length::Fixed(card_h))
        .padding(0)
        .style(move |theme: &Theme, status| {
            let p = theme.extended_palette();
            let hovered = status == iced::widget::button::Status::Hovered;
            let bg = if active {
                p.primary.base.color.scale_alpha(0.12)
            } else if hovered {
                p.background.weak.color
            } else {
                p.background.base.color
            };
            iced::widget::button::Style {
                background: Some(bg.into()),
                border: Border {
                    width: if active { 2.0 } else { 1.0 },
                    color: if active { p.primary.base.color } else { p.background.weak.color },
                    radius: 12.0.into(),
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .on_press(Message::MindMapTool(MindMapMessage::SetTreeLayoutFormat(f)))
    };

    Some(
        container(
            column![
                container(text("布局格式").size(12))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                column![
                    row![card(formats[0]), card(formats[1])]
                        .spacing(card_gap)
                        .align_y(Alignment::Center),
                    row![card(formats[2]), card(formats[3])]
                        .spacing(card_gap)
                        .align_y(Alignment::Center),
                ]
                .spacing(card_gap),
            ]
            .spacing(8),
        )
        .width(Length::Fixed(desc_w))
        .into(),
    )
}
