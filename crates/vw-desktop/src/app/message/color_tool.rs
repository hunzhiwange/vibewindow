//! 颜色工具消息处理模块
//!
//! 本模块提供颜色选择器的消息处理和颜色格式转换功能。
//! 支持多种颜色格式的输入验证和解析，包括：
//! - HEX（十六进制）格式
//! - RGB/RGBA 格式
//! - HSL/HSLA 格式
//! - HSV/HSVA 格式
//!
//! # 主要功能
//!
//! - 处理颜色选择器的变更消息
//! - 验证并解析各种颜色格式输入
//! - 在不同颜色空间之间进行转换
//! - 提供颜色值复制到剪贴板的功能
//! - 显示操作通知消息

use crate::app::views::design::models::ColorFormat;
use crate::app::views::design::properties::color_picker::{
    Hsv, format_rgba_to_css, format_rgba_to_hex, hsla_to_rgba, parse_color, parse_css_color,
    rgba_to_hsla,
};
use crate::app::{App, Message};
use iced::Color;
use iced::Task;

/// 颜色工具消息枚举
///
/// 定义颜色选择器工具支持的所有消息类型，
/// 用于用户交互和状态更新。
#[derive(Debug, Clone)]
pub enum ColorToolMessage {
    /// 颜色值变更消息
    ///
    /// 当用户通过颜色选择器选择新颜色时触发
    ColorChanged(Color),

    /// 颜色格式变更消息
    ///
    /// 当用户切换输出颜色格式（如从 HEX 切换到 RGB）时触发
    ColorFormatChanged(ColorFormat),

    /// HEX 输入框内容变更消息
    ///
    /// 当用户在 HEX 输入框中输入文本时触发
    HexInputChanged(String),

    /// HEX 输入验证消息
    ///
    /// 当用户完成 HEX 输入并需要验证时触发
    HexValidate,

    /// RGB 输入框内容变更消息
    ///
    /// 当用户在 RGB/A 输入框中输入文本时触发
    RgbInputChanged(String),

    /// RGB 输入验证消息
    ///
    /// 当用户完成 RGB/A 输入并需要验证时触发
    RgbValidate,

    /// HSL 输入框内容变更消息
    ///
    /// 当用户在 HSL/A 输入框中输入文本时触发
    HslInputChanged(String),

    /// HSL 输入验证消息
    ///
    /// 当用户完成 HSL/A 输入并需要验证时触发
    HslValidate,

    /// HSV 输入框内容变更消息
    ///
    /// 当用户在 HSV/A 输入框中输入文本时触发
    HsvInputChanged(String),

    /// HSV 输入验证消息
    ///
    /// 当用户完成 HSV/A 输入并需要验证时触发
    HsvValidate,

    /// 复制到剪贴板消息
    ///
    /// 将指定的文本内容复制到系统剪贴板
    Copy(String),

    /// 清除通知消息
    ///
    /// 清除当前显示的通知消息
    ClearNotification,
}

/// 处理颜色工具消息
///
/// 根据传入的消息类型更新应用状态，并返回相应的任务。
///
/// # 参数
///
/// * `app` - 可变引用的应用状态
/// * `message` - 要处理的颜色工具消息
///
/// # 返回值
///
/// 返回一个 `Task<Message>`，可能包含异步操作（如剪贴板写入、延迟通知清除等）
///
/// # 示例
///
/// ```ignore
/// let task = update(&mut app, ColorToolMessage::HexValidate);
/// ```
pub fn update(app: &mut App, message: ColorToolMessage) -> Task<Message> {
    match message {
        // 处理颜色选择器变更：直接更新当前颜色
        ColorToolMessage::ColorChanged(c) => {
            app.color_tool_color = c;
            sync_inputs_from_color(app);
            Task::none()
        }

        // 处理颜色格式变更：更新输出格式设置
        ColorToolMessage::ColorFormatChanged(fmt) => {
            app.color_tool_format = fmt;
            Task::none()
        }

        // 处理 HEX 输入框内容变更：更新输入缓冲区
        ColorToolMessage::HexInputChanged(s) => {
            app.color_hex_input = s;
            Task::none()
        }

        // 处理 HEX 输入验证：解析并更新颜色，显示结果通知
        ColorToolMessage::HexValidate => {
            if let Some(c) = parse_color(app.color_hex_input.trim()) {
                // 解析成功：更新颜色并显示成功通知
                app.color_tool_color = c;
                sync_inputs_from_color(app);
                app.color_notification = Some("已更新颜色".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            } else {
                // 解析失败：显示错误通知
                app.color_notification = Some("HEX 格式错误".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            }
        }

        // 处理 RGB 输入框内容变更：更新输入缓冲区
        ColorToolMessage::RgbInputChanged(s) => {
            app.color_rgb_input = s;
            Task::none()
        }

        // 处理 RGB 输入验证：解析 CSS 颜色格式（rgb/rgba）并更新颜色
        ColorToolMessage::RgbValidate => {
            if let Some(c) = parse_css_color(app.color_rgb_input.trim()) {
                // 解析成功：更新颜色并显示成功通知
                app.color_tool_color = c;
                sync_inputs_from_color(app);
                app.color_notification = Some("已更新颜色".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            } else {
                // 解析失败：显示错误通知
                app.color_notification = Some("RGB/A 格式错误".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            }
        }

        // 处理 HSL 输入框内容变更：更新输入缓冲区
        ColorToolMessage::HslInputChanged(s) => {
            app.color_hsl_input = s;
            Task::none()
        }

        // 处理 HSL 输入验证：解析 HSL/HSLA 格式并转换为 RGBA
        ColorToolMessage::HslValidate => match parse_hsl(&app.color_hsl_input) {
            Some((h, s, l, a)) => {
                // 解析成功：将 HSL 转换为 RGBA 并更新颜色
                let c = hsla_to_rgba(h, s, l, a);
                app.color_tool_color = c;
                sync_inputs_from_color(app);
                app.color_notification = Some("已更新颜色".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            }
            None => {
                // 解析失败：显示错误通知
                app.color_notification = Some("HSL/A 格式错误".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            }
        },

        // 处理 HSV 输入框内容变更：更新输入缓冲区
        ColorToolMessage::HsvInputChanged(s) => {
            app.color_hsv_input = s;
            Task::none()
        }

        // 处理 HSV 输入验证：解析 HSV/HSVA 格式并转换为 RGBA
        ColorToolMessage::HsvValidate => match parse_hsv(&app.color_hsv_input) {
            Some((h, s, v, a)) => {
                // 解析成功：将 HSV 转换为 RGBA 并更新颜色
                let hsv = Hsv { h, s, v };
                let mut c = hsv.to_color();
                c.a = a; // 单独设置 alpha 通道
                app.color_tool_color = c;
                sync_inputs_from_color(app);
                app.color_notification = Some("已更新颜色".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            }
            None => {
                // 解析失败：显示错误通知
                app.color_notification = Some("HSV/A 格式错误".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                )
            }
        },

        // 处理复制操作：将文本写入剪贴板并显示通知
        ColorToolMessage::Copy(text) => {
            app.color_notification = Some("已复制结果".to_string());
            // 批量执行剪贴板写入和延迟清除通知
            Task::batch(vec![
                iced::clipboard::write(text),
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::ColorTool(ColorToolMessage::ClearNotification),
                ),
            ])
        }

        // 处理清除通知：移除当前显示的通知消息
        ColorToolMessage::ClearNotification => {
            app.color_notification = None;
            Task::none()
        }
    }
}

fn sync_inputs_from_color(app: &mut App) {
    let color = app.color_tool_color;
    let (h, s, l, a) = rgba_to_hsla(color);
    let hsv = Hsv::from_color(color);

    app.color_hex_input = format_rgba_to_hex(color.r, color.g, color.b, color.a);
    app.color_rgb_input = format_rgba_to_css(color.r, color.g, color.b, color.a);
    app.color_hsl_input = format_hsla(h, s, l, a);
    app.color_hsv_input = format_hsva(hsv.h, hsv.s, hsv.v, color.a);
}

/// 解析 HSL/HSLA 颜色字符串
///
/// 支持的格式：
/// - `hsl(h, s%, l%)` - HSL 格式，alpha 默认为 1.0
/// - `hsla(h, s%, l%, a)` - HSLA 格式，包含 alpha 通道
///
/// # 参数
///
/// * `input` - 要解析的 HSL/HSLA 字符串
///
/// # 返回值
///
/// 解析成功返回 `Some((h, s, l, a))`，其中：
/// - `h`: 色相值，范围 [0, 360)
/// - `s`: 饱和度，范围 [0, 1]
/// - `l`: 亮度，范围 [0, 1]
/// - `a`: 透明度，范围 [0, 1]
///
/// 解析失败返回 `None`
///
/// # 示例
///
/// ```ignore
/// let result = parse_hsl("hsl(180, 50%, 75%)");
/// assert_eq!(result, Some((180.0, 0.5, 0.75, 1.0)));
///
/// let result = parse_hsl("hsla(120, 100%, 50%, 0.5)");
/// assert_eq!(result, Some((120.0, 1.0, 0.5, 0.5)));
/// ```
fn parse_hsl(input: &str) -> Option<(f32, f32, f32, f32)> {
    let s = input.trim();

    // 验证基本格式：必须以 hsl( 或 hsla( 开头，以 ) 结尾
    if !(s.starts_with("hsl(") || s.starts_with("hsla(")) || !s.ends_with(')') {
        return None;
    }

    // 判断是否为 HSLA 格式（包含 alpha 通道）
    let is_a = s.starts_with("hsla(");

    // 提取括号内的内容
    let content = if is_a { &s[5..s.len() - 1] } else { &s[4..s.len() - 1] };

    // 按逗号分割参数
    let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();

    // 验证参数数量：HSL 需要 3 个参数，HSLA 需要 4 个参数
    if (!is_a && parts.len() != 3) || (is_a && parts.len() != 4) {
        return None;
    }

    // 解析色相值（H）
    let h = parts[0].parse::<f32>().ok()?;

    // 解析饱和度（S）：去除百分号并归一化到 [0, 1]
    let s = parts[1].trim_end_matches('%').parse::<f32>().ok()? / 100.0;

    // 解析亮度（L）：去除百分号并归一化到 [0, 1]
    let l = parts[2].trim_end_matches('%').parse::<f32>().ok()? / 100.0;

    // 解析透明度（A）：仅 HSLA 格式有此参数
    let a = if is_a { parts[3].parse::<f32>().ok()? } else { 1.0 };

    // 返回归一化并限制在有效范围内的值
    Some((normalize_h(h), s.clamp(0.0, 1.0), l.clamp(0.0, 1.0), a.clamp(0.0, 1.0)))
}

/// 解析 HSV/HSVA 颜色字符串
///
/// 支持的格式：
/// - `hsv(h, s%, v%)` - HSV 格式，alpha 默认为 1.0
/// - `hsva(h, s%, v%, a)` - HSVA 格式，包含 alpha 通道
///
/// # 参数
///
/// * `input` - 要解析的 HSV/HSVA 字符串
///
/// # 返回值
///
/// 解析成功返回 `Some((h, s, v, a))`，其中：
/// - `h`: 色相值，范围 [0, 360)
/// - `s`: 饱和度，范围 [0, 1]
/// - `v`: 明度值，范围 [0, 1]
/// - `a`: 透明度，范围 [0, 1]
///
/// 解析失败返回 `None`
///
/// # 示例
///
/// ```ignore
/// let result = parse_hsv("hsv(180, 50%, 75%)");
/// assert_eq!(result, Some((180.0, 0.5, 0.75, 1.0)));
///
/// let result = parse_hsv("hsva(120, 100%, 50%, 0.5)");
/// assert_eq!(result, Some((120.0, 1.0, 0.5, 0.5)));
/// ```
fn parse_hsv(input: &str) -> Option<(f32, f32, f32, f32)> {
    let s = input.trim();

    // 验证基本格式：必须以 hsv( 或 hsva( 开头，以 ) 结尾
    if !(s.starts_with("hsv(") || s.starts_with("hsva(")) || !s.ends_with(')') {
        return None;
    }

    // 判断是否为 HSVA 格式（包含 alpha 通道）
    let is_a = s.starts_with("hsva(");

    // 提取括号内的内容
    let content = if is_a { &s[5..s.len() - 1] } else { &s[4..s.len() - 1] };

    // 按逗号分割参数
    let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();

    // 验证参数数量：HSV 需要 3 个参数，HSVA 需要 4 个参数
    if (!is_a && parts.len() != 3) || (is_a && parts.len() != 4) {
        return None;
    }

    // 解析色相值（H）
    let h = parts[0].parse::<f32>().ok()?;

    // 解析饱和度（S）：去除百分号并归一化到 [0, 1]
    let s = parts[1].trim_end_matches('%').parse::<f32>().ok()? / 100.0;

    // 解析明度值（V）：去除百分号并归一化到 [0, 1]
    let v = parts[2].trim_end_matches('%').parse::<f32>().ok()? / 100.0;

    // 解析透明度（A）：仅 HSVA 格式有此参数
    let a = if is_a { parts[3].parse::<f32>().ok()? } else { 1.0 };

    // 返回归一化并限制在有效范围内的值
    Some((normalize_h(h), s.clamp(0.0, 1.0), v.clamp(0.0, 1.0), a.clamp(0.0, 1.0)))
}

/// 归一化色相值
///
/// 将色相值归一化到 [0, 360) 范围内。
/// 支持负值和超出范围的值。
///
/// # 参数
///
/// * `h` - 原始色相值
///
/// # 返回值
///
/// 返回归一化后的色相值，范围 [0, 360)
///
/// # 示例
///
/// ```ignore
/// assert_eq!(normalize_h(0.0), 0.0);
/// assert_eq!(normalize_h(360.0), 0.0);
/// assert_eq!(normalize_h(720.0), 0.0);
/// assert_eq!(normalize_h(-90.0), 270.0);
/// assert_eq!(normalize_h(180.0), 180.0);
/// ```
fn normalize_h(h: f32) -> f32 {
    if h < 0.0 {
        // 负值：加上 360 度使其变为正值
        h + 360.0
    } else if h >= 360.0 {
        // 超出范围：取模归一化
        h % 360.0
    } else {
        // 已在有效范围内：直接返回
        h
    }
}

fn format_hsla(h: f32, s: f32, l: f32, a: f32) -> String {
    let h_int = h.round();
    let s_pct = (s * 100.0).round();
    let l_pct = (l * 100.0).round();
    format!("hsla({h_int}, {s_pct}%, {l_pct}%, {:.2})", a)
}

fn format_hsva(h: f32, s: f32, v: f32, a: f32) -> String {
    let h_int = h.round();
    let s_pct = (s * 100.0).round();
    let v_pct = (v * 100.0).round();
    format!("hsva({h_int}, {s_pct}%, {v_pct}%, {:.2})", a)
}
#[cfg(test)]
#[path = "color_tool_tests.rs"]
mod color_tool_tests;
