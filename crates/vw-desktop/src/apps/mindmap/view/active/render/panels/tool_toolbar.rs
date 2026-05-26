//! 思维导图工具栏渲染逻辑，负责常用画布工具和绘制工具入口。

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapTab};
use iced::widget::svg;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// 构建或更新 tool toolbar 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(in super::super) fn tool_toolbar(
    tab: &MindMapTab,
    tool_toolbar_w: f32,
    tool_toolbar_h: f32,
) -> Element<'_, Message> {
    let tool_btn = |icon: Icon,
                    active: bool,
                    tool: MindMapCanvasTool,
                    tip: &'static str|
     -> Element<'_, Message> {
        let btn_size = 32.0;
        let icon_el: Element<'_, Message> = svg(assets::get_icon(icon))
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(18.0))
            .content_fit(iced::ContentFit::Contain)
            .style(move |theme: &Theme, _| {
                let p = theme.extended_palette();
                let c = if active { p.background.base.text } else { theme.palette().text };
                iced::widget::svg::Style { color: Some(c) }
            })
            .into();

        let btn = button(
            container(icon_el)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .width(Length::Fixed(btn_size))
        .height(Length::Fixed(btn_size))
        .on_press(Message::MindMapTool(MindMapMessage::SetCanvasTool(tool)))
        .style(move |theme: &Theme, status| {
            let p = theme.extended_palette();
            let mix = |a: Color, b: Color, t: f32| -> Color {
                let t = t.clamp(0.0, 1.0);
                Color {
                    r: a.r * (1.0 - t) + b.r * t,
                    g: a.g * (1.0 - t) + b.g * t,
                    b: a.b * (1.0 - t) + b.b * t,
                    a: a.a * (1.0 - t) + b.a * t,
                }
            };
            let bg = if active {
                match status {
                    iced::widget::button::Status::Pressed => {
                        Some(mix(p.background.strong.color, p.background.base.color, 0.30))
                    }
                    _ => Some(mix(p.background.weak.color, p.background.base.color, 0.55)),
                }
            } else {
                match status {
                    iced::widget::button::Status::Hovered => Some(p.background.weak.color),
                    iced::widget::button::Status::Pressed => Some(p.background.strong.color),
                    _ => None,
                }
            };

            iced::widget::button::Style {
                background: bg.map(Background::Color),
                border: Border {
                    width: 1.0,
                    color: if active { p.background.strong.color } else { Color::TRANSPARENT },
                    radius: 6.0.into(),
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        });

        let tip_content = container(text(tip).size(12)).padding([6, 8]).style(|_theme: &Theme| {
            iced::widget::container::Style {
                background: Some(Color::from_rgba8(16, 16, 16, 0.96).into()),
                text_color: Some(Color::WHITE),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.40),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 18.0,
                },
                ..Default::default()
            }
        });

        tooltip::Tooltip::new(btn, tip_content, tooltip::Position::Bottom).gap(8).into()
    };

    let row = row![
        tool_btn(
            Icon::HandIndex,
            tab.canvas_tool == MindMapCanvasTool::Pan,
            MindMapCanvasTool::Pan,
            "拖动",
        ),
        tool_btn(
            Icon::Cursor,
            tab.canvas_tool == MindMapCanvasTool::Select,
            MindMapCanvasTool::Select,
            "选择",
        ),
        tool_btn(
            Icon::Pen,
            tab.canvas_tool == MindMapCanvasTool::Pen,
            MindMapCanvasTool::Pen,
            "画笔",
        ),
        tool_btn(
            Icon::Eraser,
            tab.canvas_tool == MindMapCanvasTool::Eraser,
            MindMapCanvasTool::Eraser,
            "橡皮",
        ),
    ]
    .spacing(6)
    .height(Length::Fixed(32.0))
    .align_y(Alignment::Center);

    container(row)
        .width(Length::Fixed(tool_toolbar_w))
        .height(Length::Fixed(tool_toolbar_h))
        .padding(4)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(p.background.base.color.into()),
                border: Border { width: 1.0, color: p.background.weak.color, radius: 10.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.12),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 22.0,
                },
                ..Default::default()
            }
        })
        .into()
}
