//! 导出属性面板模块
//!
//! 本模块提供设计元素的导出功能界面，允许用户将选中的设计元素导出为多种格式。
//! 支持的导出格式包括：
//! - HTML：超文本标记语言格式
//! - JPEG：联合图像专家组格式（压缩图像）
//! - PNG：便携式网络图形格式（无损图像）
//! - SVG：可缩放矢量图形格式
//!
//! # 功能特性
//!
//! - 多种导出尺寸选择（1x、2x、3x）
//! - 多种导出格式支持
//! - 质量控制（固定为100%）
//! - 快速导出和预览功能

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::DesignElement;
use iced::widget::{button, column, row, text};
use iced::{Color, Element, Theme};

/// 渲染导出属性面板
///
/// 为指定的设计元素创建导出界面，包含导出选项和操作按钮。
///
/// # 参数
///
/// * `element` - 要导出的设计元素的引用，包含元素的唯一标识符等信息
///
/// # 返回值
///
/// 返回一个 Iced UI 元素，包含完整的导出面板界面
///
/// # 界面结构
///
/// 界面包含以下部分：
/// 1. 标题栏：显示"导出"文本
/// 2. 尺寸选择：提供 1x、2x、3x 三种导出尺寸选项
/// 3. 格式选择：提供 HTML、JPEG、PNG、SVG 四种导出格式按钮
/// 4. 质量显示：显示导出质量（100%）
/// 5. 操作按钮：提供"导出 HTML"和"查看 HTML"两个主要操作按钮
///
/// # 示例
///
/// ```ignore
/// use crate::app::views::design::models::DesignElement;
/// use crate::app::views::design::properties::export::render;
///
/// let element = DesignElement {
///     id: "element-123".to_string(),
///     // ... 其他字段
/// };
/// let ui = render(&element);
/// ```
pub fn render<'a>(element: &'a DesignElement) -> Element<'a, Message> {
    let id = element.id.clone();

    // 主操作按钮样式生成器
    //
    // 为主要操作按钮（如选中的格式、导出按钮）生成样式。
    // 使用主题的主色调作为背景，悬停和按下时调整透明度。
    let export_btn_primary = |theme: &Theme, status: button::Status| {
        let p = theme.palette();
        let bg = match status {
            button::Status::Hovered => p.primary.scale_alpha(0.92),
            button::Status::Pressed => p.primary.scale_alpha(0.85),
            _ => p.primary,
        };

        button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: iced::Border { color: Color::TRANSPARENT, width: 0.0, radius: 8.0.into() },
            ..button::Style::default()
        }
    };

    // 次要操作按钮样式生成器
    //
    // 为次要操作按钮（如未选中的格式、查看按钮）生成样式。
    // 使用主题的背景色系，具有边框区分。
    let export_btn_secondary = |theme: &Theme, status: button::Status| {
        let p = theme.palette();
        let ext = theme.extended_palette();
        let bg = match status {
            button::Status::Hovered => ext.background.strong.color,
            button::Status::Pressed => ext.background.strong.color,
            _ => ext.background.weak.color,
        };

        button::Style {
            background: Some(bg.into()),
            text_color: p.text,
            border: iced::Border {
                color: ext.background.strong.color,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..button::Style::default()
        }
    };

    column![
        text("导出")
            .size(12)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        column![
            row![
                text("大小").size(12).width(40),
                text("1x").size(12).style(|t: &Theme| iced::widget::text::Style {
                    color: Some(t.palette().primary),
                }),
                text("2x")
                    .size(12)
                    .style(|t: &Theme| iced::widget::text::Style { color: Some(t.palette().text) }),
                text("3x")
                    .size(12)
                    .style(|t: &Theme| iced::widget::text::Style { color: Some(t.palette().text) }),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
            row![
                text("类型").size(12).width(40),
                button(text("HTML").size(10))
                    .padding(4)
                    .style(export_btn_primary)
                    .on_press(Message::Design(DesignMessage::ExportElementHtml(id.clone()))),
                button(text("JPEG").size(10))
                    .padding(4)
                    .style(export_btn_secondary)
                    .on_press(Message::Design(DesignMessage::ExportElementJpeg(id.clone()))),
                button(text("PNG").size(10))
                    .padding(4)
                    .style(export_btn_secondary)
                    .on_press(Message::Design(DesignMessage::ExportElementPng(id.clone()))),
                button(text("SVG").size(10))
                    .padding(4)
                    .style(export_btn_secondary)
                    .on_press(Message::Design(DesignMessage::ExportElementSvg(id.clone()))),
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center),
            row![text("质量").size(12).width(40), text("100%").size(12),]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            row![
                button(text("导出 HTML").size(12))
                    .width(iced::Length::Fill)
                    .on_press(Message::Design(DesignMessage::ExportElementHtml(id.clone())))
                    .style(export_btn_primary),
                button(text("查看 HTML").size(12))
                    .width(iced::Length::Fill)
                    .on_press(Message::Design(DesignMessage::ViewElementHtml(id.clone())))
                    .style(export_btn_secondary),
            ]
            .spacing(5),
        ]
        .spacing(8)
    ]
    .spacing(10)
    .into()
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod export_tests;
