//! 纯色填充属性渲染模块
//!
//! 本模块提供纯色填充（Solid Fill）属性的用户界面渲染功能，
//! 包括颜色选择器的渲染和颜色格式转换工具函数。
//!
//! # 主要功能
//!
//! - 渲染纯色填充的颜色选择器界面
//! - 十六进制颜色与 RGBA 格式之间的相互转换
//! - 解析颜色字符串为 iced Color 对象
//!
//! # 模块关系
//!
//! 该模块是 `fill` 模块的子模块，专门处理纯色填充类型的属性编辑。

use iced::widget::column;
use iced::{Color, Element};

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::ColorFormat;
use crate::app::views::design::properties::color_picker;
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};

/// 渲染纯色填充的颜色选择器界面
///
/// 根据提供的颜色值和配置，渲染一个颜色选择器组件，
/// 允许用户选择和编辑纯色填充的颜色。
///
/// # 参数
///
/// * `color_str` - 当前颜色的十六进制字符串表示（如 "#ff0000" 或 "#ff0000ff"）
/// * `index` - 填充项在填充列表中的索引位置
/// * `fills` - 完整的填充项列表
/// * `id` - 目标对象的唯一标识符
/// * `format` - 颜色显示格式（HEX、RGB、HSL 等）
/// * `picking` - 是否处于颜色拾取（吸管工具）模式
///
/// # 返回值
///
/// 返回一个 `Element`，包含颜色选择器的完整界面元素。
///
/// # 示例
///
/// ```ignore
/// let element = render(
///     "#ff0000".to_string(),
///     0,
///     fills,
///     "shape-1".to_string(),
///     ColorFormat::Hex,
///     false,
/// );
/// ```
pub fn render(
    color_str: String,
    _index: usize,
    _fills: Vec<FillItem>,
    _id: String,
    format: ColorFormat,
    picking: bool,
) -> Element<'static, Message> {
    let (r, g, b, a) = parse_hex_to_rgba(&color_str);
    let current_color = Color::from_rgba(r, g, b, a);

    column![color_picker::render_color_picker(
        current_color,
        format,
        picking,
        move |c| Message::Design(DesignMessage::FillPickerColorChange(c)),
        move |f| Message::Design(DesignMessage::FillPickerFormatChange(f)),
        move || Message::Design(DesignMessage::FillPickerEyedropper),
    )]
    .into()
}

/// 更新填充列表中指定索引位置的颜色值
///
/// # 参数
///
/// * `id` - 目标对象的唯一标识符
/// * `fills` - 当前的填充项列表
/// * `index` - 需要更新的填充项索引
/// * `color` - 新的颜色值（十六进制字符串）
///
/// # 返回值
///
/// 返回一个 `Message`，用于触发属性更新操作。
#[allow(dead_code)]
fn update_fill_color(id: String, fills: Vec<FillItem>, index: usize, color: String) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(item) = new_fills.get_mut(index) {
        match item {
            FillItem::Color(c) => *c = color,
            FillItem::Object(FillObject::Solid { color: c, .. }) => *c = color,
            FillItem::Object(FillObject::Color { color: c, .. }) => *c = color,
            _ => {}
        }
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 将十六进制颜色字符串解析为 RGBA 分量
///
/// 将颜色字符串解析为四个 f32 分量（红、绿、蓝、透明度），
/// 每个分量范围为 0.0 到 1.0。
///
/// # 参数
///
/// * `hex` - 十六进制颜色字符串（如 "#ff0000" 或 "#ff0000ff"）
///
/// # 返回值
///
/// 返回包含 (r, g, b, a) 的元组。如果解析失败，返回黑色不透明 (0.0, 0.0, 0.0, 1.0)。
///
/// # 示例
///
/// ```ignore
/// let (r, g, b, a) = parse_hex_to_rgba("#ff000080");
/// assert_eq!(r, 1.0);
/// assert_eq!(g, 0.0);
/// assert_eq!(b, 0.0);
/// assert_eq!(a, 0.5);
/// ```
pub fn parse_hex_to_rgba(hex: &str) -> (f32, f32, f32, f32) {
    if let Some(c) = parse_color(hex) { (c.r, c.g, c.b, c.a) } else { (0.0, 0.0, 0.0, 1.0) }
}

/// 将 RGBA 分量格式化为十六进制颜色字符串
///
/// 将四个颜色分量（红、绿、蓝、透明度）转换为带透明度的
/// 十六进制颜色字符串格式 "#rrggbbaa"。
///
/// # 参数
///
/// * `r` - 红色分量 (0.0 - 1.0)
/// * `g` - 绿色分量 (0.0 - 1.0)
/// * `b` - 蓝色分量 (0.0 - 1.0)
/// * `a` - 透明度分量 (0.0 - 1.0)
///
/// # 返回值
///
/// 返回格式为 "#rrggbbaa" 的十六进制字符串。
///
/// # 示例
///
/// ```ignore
/// let hex = format_rgba_to_hex(1.0, 0.0, 0.0, 0.5);
/// assert_eq!(hex, "#ff000080");
/// ```
pub fn format_rgba_to_hex(r: f32, g: f32, b: f32, a: f32) -> String {
    let r_u8 = (r * 255.0).round() as u8;
    let g_u8 = (g * 255.0).round() as u8;
    let b_u8 = (b * 255.0).round() as u8;
    let a_u8 = (a * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}{:02X}", r_u8, g_u8, b_u8, a_u8)
}

/// 解析颜色字符串为 iced Color 对象
///
/// 支持解析以下格式的十六进制颜色字符串：
/// - 6 位格式：#rrggbb（透明度默认为 255）
/// - 8 位格式：#rrggbbaa（包含透明度）
///
/// # 参数
///
/// * `s` - 颜色字符串，必须以 "#" 开头
///
/// # 返回值
///
/// - 如果解析成功，返回 `Some(Color)`
/// - 如果字符串格式不正确，返回 `None`
///
/// # 示例
///
/// ```ignore
/// // 解析不带透明度的颜色
/// let color = parse_color("#ff0000");
/// assert!(color.is_some());
///
/// // 解析带透明度的颜色
/// let color = parse_color("#ff000080");
/// assert!(color.is_some());
///
/// // 无效格式返回 None
/// let color = parse_color("invalid");
/// assert!(color.is_none());
/// ```
pub fn parse_color(s: &str) -> Option<Color> {
    if s.len() >= 7 && s.starts_with('#') {
        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;
        let a = if s.len() == 9 { u8::from_str_radix(&s[7..9], 16).ok()? } else { 255 };
        Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "solid_tests.rs"]
mod solid_tests;
