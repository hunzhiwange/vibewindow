//! 设计变量面板模块，负责变量集合、主题模式和值编辑界面的拆分实现。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::Color;

use crate::app::views::design::models::VariableDef;
use crate::app::views::design::properties::color_picker::format_rgba_to_hex;

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `def`: 当前视图构建所需的状态、配置或消息。
/// - `current_collection`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回判断结果，供调用方选择分支或样式。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn variable_belongs_to_collection(def: &VariableDef, current_collection: &str) -> bool {
    def.collection
        .as_ref()
        .map(|collection| collection.eq_ignore_ascii_case(current_collection))
        .unwrap_or(false)
}

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `def`: 当前视图构建所需的状态、配置或消息。
/// - `mode`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn direct_variable_value(def: &VariableDef, mode: Option<&str>) -> String {
    def.value
        .iter()
        .find_map(|entry| match (&entry.theme, mode) {
            (None, None) => Some(entry.value.clone()),
            (Some(theme), Some(target_mode)) if theme.mode.eq_ignore_ascii_case(target_mode) => {
                Some(entry.value.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// 解析输入值。
///
/// # 参数
/// - `input`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回匹配到的值；无法安全转换或当前状态不适用时返回 `None`。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn parse_hex_color(input: &str) -> Option<Color> {
    let raw = input.trim().trim_start_matches('#');
    let (r, g, b, a) = match raw.len() {
        6 => {
            let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
            let a = u8::from_str_radix(&raw[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };
    Some(Color::from_rgba8(r, g, b, (a as f32) / 255.0))
}

/// 计算颜色表现。
///
/// # 参数
/// - `value`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn color_hex_input_value(value: &str) -> String {
    parse_hex_color(value)
        .map(|color| {
            format!(
                "#{:02x}{:02x}{:02x}",
                (color.r * 255.0).round() as u8,
                (color.g * 255.0).round() as u8,
                (color.b * 255.0).round() as u8
            )
        })
        .unwrap_or_else(|| {
            let trimmed = value.trim();
            if trimmed.is_empty() { "#000000".to_string() } else { trimmed.to_string() }
        })
}

/// 计算颜色表现。
///
/// # 参数
/// - `value`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn color_alpha_input_value(value: &str) -> String {
    parse_hex_color(value)
        .map(|color| ((color.a * 100.0).round() as i32).clamp(0, 100).to_string())
        .unwrap_or_else(|| "100".to_string())
}

/// 计算颜色表现。
///
/// # 参数
/// - `current_value`: 当前视图构建所需的状态、配置或消息。
/// - `new_hex`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn update_color_hex_value(current_value: &str, new_hex: &str) -> String {
    let normalized = normalize_hex_input(new_hex);
    let alpha = parse_hex_color(current_value).map(|color| color.a).unwrap_or(1.0);
    if let Some(color) = parse_hex_color(&normalized) {
        format_rgba_to_hex(color.r, color.g, color.b, alpha)
    } else {
        normalized
    }
}

/// 计算颜色表现。
///
/// # 参数
/// - `current_value`: 当前视图构建所需的状态、配置或消息。
/// - `new_alpha`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn update_color_alpha_value(current_value: &str, new_alpha: &str) -> String {
    let Some(color) = parse_hex_color(current_value) else {
        return current_value.to_string();
    };
    let alpha = new_alpha.trim().parse::<f32>().unwrap_or(100.0).clamp(0.0, 100.0) / 100.0;
    format_rgba_to_hex(color.r, color.g, color.b, alpha)
}

/// 计算颜色表现。
///
/// # 参数
/// - `color`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回根据输入和主题计算出的颜色。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn swatch_border_color(color: Color) -> Color {
    if color.r + color.g + color.b > 2.1 {
        Color::from_rgba8(15, 23, 42, 0.12)
    } else {
        Color::from_rgba8(255, 255, 255, 0.18)
    }
}

fn normalize_hex_input(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        String::new()
    } else if trimmed.starts_with('#') {
        trimmed.to_string()
    } else {
        format!("#{trimmed}")
    }
}
#[cfg(test)]
#[path = "utils_tests.rs"]
mod utils_tests;
