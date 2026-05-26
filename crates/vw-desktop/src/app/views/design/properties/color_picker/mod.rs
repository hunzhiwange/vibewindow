//! 颜色选择器属性模块，负责颜色解析、格式转换和拾色控件渲染。

use crate::app::views::design::models::{ColorFormat, ColorPickerTarget};
use iced::Color;
use iced::Point;

mod conversion;
mod hsv;
mod images;
mod pickers;
mod render_full;
mod render_mini;

pub use conversion::{
    format_rgba_to_css, format_rgba_to_hex, hsla_to_rgba, parse_color, parse_css_color,
    rgba_to_hsla,
};
pub use hsv::Hsv;
pub use pickers::{AlphaPicker, HuePicker, SaturationValuePicker};
pub use render_full::render_color_picker;
pub use render_mini::render_mini_color_picker;

#[derive(Debug, Clone)]
/// ActiveColorPicker 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub struct ActiveColorPicker {
    pub color: Color,
    pub format: ColorFormat,
    pub target: ColorPickerTarget,
    pub position: Point,
    pub picking: bool,
}

#[cfg(test)]
mod tests;
