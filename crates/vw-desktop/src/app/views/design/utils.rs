//! 设计器通用样式模块，提供透明输入框和文本编辑器的外观工具。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{text_editor, text_input};
use iced::{Background, Border, Color, Theme};

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
/// - `_status`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn transparent_input_style(theme: &Theme, _status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();
    text_input::Style {
        background: Color::TRANSPARENT.into(),
        border: iced::Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
        icon: Color::TRANSPARENT,
        placeholder: palette.text,
        value: palette.text,
        selection: palette.primary,
    }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `color`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn transparent_editor_style(color: Color) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
        value: color,
        selection: Color::from_rgba(0.0, 0.5, 1.0, 0.3),
        placeholder: color,
    }
}
#[cfg(test)]
#[path = "utils_tests.rs"]
mod utils_tests;
