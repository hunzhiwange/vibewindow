//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::{Color, Theme};

/// OVERLAY_ICON_PICKER_RESULT_LIMIT 定义本视图复用的固定尺寸或配置值。
pub(super) const OVERLAY_ICON_PICKER_RESULT_LIMIT: usize = 50;
/// OVERLAY_ICON_PICKER_REQUIRE_QUERY_THRESHOLD 定义本视图复用的固定尺寸或配置值。
pub(super) const OVERLAY_ICON_PICKER_REQUIRE_QUERY_THRESHOLD: usize = 200;

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回判断结果，供调用方选择分支或样式。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn design_overlay_is_dark(theme: &Theme) -> bool {
    let background = theme.palette().background;
    background.r + background.g + background.b < 1.5
}

/// 计算颜色表现。
///
/// # 参数
/// - `background`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回根据输入和主题计算出的颜色。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn design_overlay_contrast_text_color(background: Color) -> Color {
    if background.r + background.g + background.b > 1.5 { Color::BLACK } else { Color::WHITE }
}

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
/// - `dark_alpha`: 当前视图构建所需的状态、配置或消息。
/// - `light_alpha`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回根据输入和主题计算出的颜色。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn design_overlay_surface_shadow(
    theme: &Theme,
    dark_alpha: f32,
    light_alpha: f32,
) -> Color {
    let alpha = if design_overlay_is_dark(theme) { dark_alpha } else { light_alpha };
    Color::BLACK.scale_alpha(alpha)
}
#[cfg(test)]
#[path = "overlay_shared_tests.rs"]
mod overlay_shared_tests;
