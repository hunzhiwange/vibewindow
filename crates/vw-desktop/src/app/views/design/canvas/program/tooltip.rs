//! 设计画布程序节点模块。
//!
//! 该模块处理程序节点展示与提示信息，让画布能够表达可执行或可交互的设计元素。

use iced::{
    Color, Pixels, Point, Rectangle, Size,
    widget::canvas::{Frame, Path, Stroke, Text},
};

fn estimate_text_width(label: &str, font_size: f32) -> f32 {
    label.chars().map(|ch| if ch.is_ascii() { font_size * 0.56 } else { font_size * 0.94 }).sum()
}

/// 模块内部可见的 draw_tooltip 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_tooltip(
    frame: &mut Frame,
    target_rect: Rectangle,
    bounds: Rectangle,
    label: &str,
) {
    let font_size = 14.0;
    let padding_x = 11.0;
    let padding_y = 7.0;
    let text_width = estimate_text_width(label, font_size);
    let box_w = text_width + padding_x * 2.0;
    let box_h = font_size * 1.2 + padding_y * 2.0;
    let desired_x = target_rect.x + (target_rect.width - box_w) / 2.0;
    let min_x = 8.0;
    let max_x = (bounds.width - box_w - 8.0).max(min_x);
    let x = desired_x.clamp(min_x, max_x);
    let above_y = target_rect.y - box_h - 8.0;
    let y = if above_y < 8.0 {
        (target_rect.y + target_rect.height + 8.0).min(bounds.height - box_h - 8.0)
    } else {
        above_y
    };
    let background = Color::from_rgba8(6, 8, 12, 0.96);
    let border = Color::from_rgba8(112, 123, 145, 0.34);
    let text_color = Color::from_rgba8(244, 247, 250, 0.96);
    let path = Path::rounded_rectangle(Point::new(x, y), Size::new(box_w, box_h), 11.0.into());

    frame.fill(&path, background);
    frame.stroke(&path, Stroke::default().with_color(border).with_width(1.0));
    frame.fill_text(Text {
        content: label.to_string(),
        position: Point::new(x + padding_x, y + box_h / 2.0),
        size: Pixels(font_size),
        color: text_color,
        align_x: iced::alignment::Horizontal::Left.into(),
        align_y: iced::alignment::Vertical::Center,
        ..Default::default()
    });
}

#[cfg(test)]
#[path = "tooltip_tests.rs"]
mod tooltip_tests;
