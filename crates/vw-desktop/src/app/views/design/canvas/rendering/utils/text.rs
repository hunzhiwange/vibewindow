//! 设计画布渲染工具模块。
//!
//! 该模块提供路径、图片、文本和 Tailwind 样式转换等底层辅助函数，减少渲染主流程中的重复样板逻辑。

use iced::{
    Color, Point, Size,
    widget::canvas::{Frame, Path, Stroke},
};

/// 公开的 compute_line_width 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn compute_line_width(s: &str, font_size: f32, letter_spacing: f32) -> f32 {
    let mut w = 0.0;
    let chars: Vec<char> = s.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        w += if *ch as u32 > 127 { font_size } else { font_size * 0.6 };
        if i < chars.len() - 1 {
            w += letter_spacing;
        }
    }
    w
}

/// 公开的 apply_text_transform 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn apply_text_transform(content: &str, transform: Option<&str>) -> String {
    match transform {
        Some("uppercase") => content.to_uppercase(),
        Some("lowercase") => content.to_lowercase(),
        Some("capitalize") => content
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => content.to_string(),
    }
}

/// 公开的 wrap_text_words 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn wrap_text_words(
    content: &str,
    max_width: f32,
    font_size: f32,
    letter_spacing: f32,
) -> (Vec<String>, Size) {
    let line_height = font_size * 1.5;
    let available_width = max_width.max(0.0);

    if content.is_empty() {
        return (Vec::new(), Size::new(0.0, 0.0));
    }

    let mut lines = Vec::new();
    let mut max_line_width: f32 = 0.0;

    for raw_line in content.split('\n') {
        let words: Vec<&str> = raw_line.split_whitespace().collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();

        for word in words {
            let candidate = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{current_line} {word}")
            };
            let candidate_width = compute_line_width(&candidate, font_size, letter_spacing);

            if current_line.is_empty()
                || available_width <= 0.0
                || candidate_width <= available_width
            {
                current_line = candidate;
            } else {
                max_line_width = max_line_width.max(compute_line_width(
                    &current_line,
                    font_size,
                    letter_spacing,
                ));
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            max_line_width =
                max_line_width.max(compute_line_width(&current_line, font_size, letter_spacing));
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        return (vec![], Size::new(0.0, 0.0));
    }

    let height = lines.len() as f32 * line_height;
    let width = max_line_width;

    (lines, Size::new(width, height))
}

/// 公开的 draw_text_decoration 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_text_decoration(
    frame: &mut Frame,
    decoration: &str,
    start_x: f32,
    line_y: f32,
    line_width: f32,
    font_size: f32,
    color: Color,
    zoom: f32,
) {
    let y_underline = line_y + font_size * 0.85;
    let y_strike = line_y + font_size * 0.45;

    if decoration.contains("underline") {
        let underline = Path::line(
            Point::new(start_x, y_underline),
            Point::new(start_x + line_width, y_underline),
        );
        frame.stroke(&underline, Stroke::default().with_color(color).with_width(1.0 * zoom));
    }

    if decoration.contains("line-through") {
        let strike =
            Path::line(Point::new(start_x, y_strike), Point::new(start_x + line_width, y_strike));
        frame.stroke(&strike, Stroke::default().with_color(color).with_width(1.0 * zoom));
    }
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod text_tests;
