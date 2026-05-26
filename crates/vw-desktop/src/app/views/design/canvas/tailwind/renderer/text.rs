//! Tailwind 渲染器模块，负责把解析后的节点样式转换为画布中的布局、命中区域和绘制数据。

use iced::widget::canvas::{Frame, Text};
use iced::{Color, Point, Rectangle};

use super::super::parser::ParsedStyle;
use crate::app::views::design::canvas::rendering::utils::{
    apply_text_transform, compute_line_width, wrap_text_words,
};

use super::style::{apply_opacity, resolve_font, resolve_text_align};

#[derive(Debug, Clone)]
/// TailwindTextLayout 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub(super) struct TailwindTextLayout {
    pub(super) lines: Vec<String>,
    pub(super) color: Color,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) letter_spacing: f32,
    pub(super) align: iced::alignment::Horizontal,
    pub(super) decoration: Option<String>,
    pub(super) font: iced::Font,
    pub(super) is_justify: bool,
}

/// 执行 resolve_text_layout 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_text_layout(
    text_content: &str,
    bounds: Rectangle,
    zoom: f32,
    style: &ParsedStyle,
) -> TailwindTextLayout {
    let font_size = style.font_size.unwrap_or(16.0) * zoom;
    let letter_spacing = style.letter_spacing.unwrap_or(0.0) * zoom;
    let line_height = style.line_height.map(|value| value * font_size).unwrap_or(font_size * 1.5);
    let transformed = apply_text_transform(text_content, style.text_transform.as_deref());
    let (lines, _) = wrap_text_words(&transformed, bounds.width, font_size, letter_spacing);
    let color = apply_opacity(style.text_color.unwrap_or(Color::BLACK), style.opacity);

    TailwindTextLayout {
        lines,
        color,
        font_size,
        line_height,
        letter_spacing,
        align: resolve_text_align(style),
        decoration: style.text_decoration.clone(),
        font: resolve_font(style),
        is_justify: style.text_align.as_deref() == Some("justify"),
    }
}

#[cfg(test)]
/// 执行 text_layout_size 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn text_layout_size(layout: &TailwindTextLayout) -> iced::Size {
    if layout.lines.is_empty() {
        return iced::Size::new(0.0, 0.0);
    }

    let width = layout
        .lines
        .iter()
        .map(|line| compute_line_width(line, layout.font_size, layout.letter_spacing))
        .fold(0.0, f32::max);

    iced::Size::new(width, layout.lines.len() as f32 * layout.line_height)
}

/// 执行 fill_text_with_spacing 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn fill_text_with_spacing(
    frame: &mut Frame,
    content: &str,
    x: f32,
    y: f32,
    layout: &TailwindTextLayout,
) {
    if content.is_empty() {
        return;
    }

    if layout.letter_spacing == 0.0 {
        frame.fill_text(Text {
            content: content.to_string(),
            position: Point::new(x, y),
            color: layout.color,
            size: iced::Pixels(layout.font_size),
            font: layout.font,
            ..Default::default()
        });
        return;
    }

    let mut cursor_x = 0.0;
    let chars: Vec<char> = content.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        let glyph = ch.to_string();
        frame.fill_text(Text {
            content: glyph.clone(),
            position: Point::new(x + cursor_x, y),
            color: layout.color,
            size: iced::Pixels(layout.font_size),
            font: layout.font,
            ..Default::default()
        });
        cursor_x += compute_line_width(&glyph, layout.font_size, 0.0);
        if idx < chars.len() - 1 {
            cursor_x += layout.letter_spacing;
        }
    }
}
