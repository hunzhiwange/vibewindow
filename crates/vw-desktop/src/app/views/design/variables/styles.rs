//! 设计变量面板模块，负责变量集合、主题模式和值编辑界面的拆分实现。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};

/// VariablesPalette 保存本视图片段需要跨控件传递的轻量状态。
#[derive(Debug, Clone, Copy)]
pub(super) struct VariablesPalette {
    pub(super) backdrop: Color,
    pub(super) panel_bg: Color,
    pub(super) panel_border: Color,
    pub(super) panel_shadow: Color,
    pub(super) title: Color,
    pub(super) subtitle: Color,
    pub(super) header_text: Color,
    pub(super) row_divider: Color,
    pub(super) name_bg: Color,
    pub(super) name_border: Color,
    pub(super) name_text: Color,
    pub(super) cell_bg: Color,
    pub(super) cell_border: Color,
    pub(super) cell_text: Color,
    pub(super) menu_bg: Color,
    pub(super) menu_border: Color,
    pub(super) menu_hover_bg: Color,
    pub(super) menu_text: Color,
    pub(super) danger_text: Color,
    pub(super) danger_bg: Color,
}

/// NAME_COL_WIDTH 定义本视图复用的固定尺寸或配置值。
pub(super) const NAME_COL_WIDTH: f32 = 220.0;
/// VARIANT_COL_WIDTH 定义本视图复用的固定尺寸或配置值。
pub(super) const VARIANT_COL_WIDTH: f32 = 236.0;
/// ACTION_COL_WIDTH 定义本视图复用的固定尺寸或配置值。
pub(super) const ACTION_COL_WIDTH: f32 = 42.0;
/// TABLE_GAP 定义本视图复用的固定尺寸或配置值。
pub(super) const TABLE_GAP: f32 = 12.0;
/// THEME_TABS_SCROLL_HEIGHT 定义本视图复用的固定尺寸或配置值。
pub(super) const THEME_TABS_SCROLL_HEIGHT: f32 = 40.0;
/// THEME_TAB_BUTTON_HEIGHT 定义本视图复用的固定尺寸或配置值。
pub(super) const THEME_TAB_BUTTON_HEIGHT: f32 = 30.0;
/// THEME_TAB_MENU_BUTTON_SIZE 定义本视图复用的固定尺寸或配置值。
pub(super) const THEME_TAB_MENU_BUTTON_SIZE: f32 = 18.0;
/// THEME_ADD_BUTTON_SIZE 定义本视图复用的固定尺寸或配置值。
pub(super) const THEME_ADD_BUTTON_SIZE: f32 = 24.0;
/// THEME_TAB_MENU_GAP 定义本视图复用的固定尺寸或配置值。
pub(super) const THEME_TAB_MENU_GAP: f32 = 38.0;
/// VARIABLE_MENU_BUTTON_HEIGHT 定义本视图复用的固定尺寸或配置值。
pub(super) const VARIABLE_MENU_BUTTON_HEIGHT: f32 = 32.0;
/// VARIABLE_FOOTER_HEIGHT 定义本视图复用的固定尺寸或配置值。
pub(super) const VARIABLE_FOOTER_HEIGHT: f32 = 72.0;
/// PANEL_WIDTH 定义本视图复用的固定尺寸或配置值。
pub(super) const PANEL_WIDTH: f32 = 1080.0;
/// PANEL_HEIGHT 定义本视图复用的固定尺寸或配置值。
pub(super) const PANEL_HEIGHT: f32 = 600.0;

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
/// - `status`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn variable_text_input_style(
    theme: &Theme,
    status: text_input::Status,
) -> text_input::Style {
    let palette = theme.palette();
    let extended = theme.extended_palette();
    let focused = matches!(status, text_input::Status::Focused { .. });
    let border_color = if focused { palette.primary } else { extended.background.strong.color };
    let background =
        if focused { extended.background.weak.color } else { extended.background.base.color };

    text_input::Style {
        background: Background::Color(background),
        border: Border { width: 1.0, color: border_color, radius: 8.0.into() },
        icon: palette.text.scale_alpha(0.5),
        placeholder: palette.text.scale_alpha(0.55),
        value: palette.text,
        selection: palette.primary.scale_alpha(0.30),
    }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
/// - `status`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn variable_value_input_style(
    theme: &Theme,
    status: text_input::Status,
) -> text_input::Style {
    let palette = variables_palette(theme);
    let focused = matches!(status, text_input::Status::Focused { .. });
    text_input::Style {
        background: Background::Color(if focused {
            palette.menu_hover_bg
        } else {
            palette.cell_bg
        }),
        border: Border {
            width: 1.0,
            color: if focused { theme.palette().primary } else { palette.cell_border },
            radius: 8.0.into(),
        },
        icon: palette.cell_text.scale_alpha(0.5),
        placeholder: palette.cell_text.scale_alpha(0.45),
        value: palette.cell_text,
        selection: theme.palette().primary.scale_alpha(0.30),
    }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn backdrop_style(theme: &Theme) -> container::Style {
    let palette = variables_palette(theme);
    container::Style { background: Some(Background::Color(palette.backdrop)), ..Default::default() }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn panel_surface_style(theme: &Theme) -> container::Style {
    let palette = variables_palette(theme);
    container::Style {
        background: Some(Background::Color(palette.panel_bg)),
        border: Border { radius: 18.0.into(), width: 1.0, color: palette.panel_border },
        shadow: iced::Shadow {
            color: palette.panel_shadow.scale_alpha(0.82),
            offset: iced::Vector::new(0.0, 18.0),
            blur_radius: 42.0,
        },
        ..Default::default()
    }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn menu_surface_style(theme: &Theme) -> container::Style {
    let palette = variables_palette(theme);
    container::Style {
        background: Some(Background::Color(palette.menu_bg)),
        border: Border { radius: 10.0.into(), width: 1.0, color: palette.menu_border },
        shadow: iced::Shadow {
            color: palette.panel_shadow.scale_alpha(0.78),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 28.0,
        },
        ..Default::default()
    }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `destructive`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn menu_button_style(
    destructive: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |theme: &Theme, status| {
        let palette = variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered {
                    if destructive {
                        palette.danger_bg.scale_alpha(0.14)
                    } else {
                        palette.menu_hover_bg
                    }
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            text_color: if destructive { palette.danger_text } else { palette.menu_text },
            border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        }
    }
}

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回值遵循函数签名约定，调用方据此继续组装界面或更新状态。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn variables_palette(theme: &Theme) -> VariablesPalette {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    if is_dark {
        VariablesPalette {
            backdrop: Color::from_rgba8(7, 8, 11, 120.0 / 255.0),
            panel_bg: Color::from_rgba8(28, 28, 31, 0.98),
            panel_border: Color::from_rgba8(255, 255, 255, 0.08),
            panel_shadow: Color::BLACK.scale_alpha(0.52),
            title: Color::from_rgba8(250, 250, 251, 1.0),
            subtitle: Color::from_rgba8(161, 161, 170, 1.0),
            header_text: Color::from_rgba8(212, 212, 216, 1.0),
            row_divider: Color::from_rgba8(255, 255, 255, 0.05),
            name_bg: Color::from_rgba8(38, 38, 42, 1.0),
            name_border: Color::from_rgba8(255, 255, 255, 0.05),
            name_text: Color::from_rgba8(244, 244, 245, 1.0),
            cell_bg: Color::from_rgba8(36, 36, 39, 1.0),
            cell_border: Color::from_rgba8(255, 255, 255, 0.06),
            cell_text: Color::from_rgba8(245, 245, 245, 1.0),
            menu_bg: Color::from_rgba8(24, 24, 27, 0.99),
            menu_border: Color::from_rgba8(255, 255, 255, 0.08),
            menu_hover_bg: Color::from_rgba8(255, 255, 255, 0.08),
            menu_text: Color::from_rgba8(244, 244, 245, 1.0),
            danger_text: Color::from_rgba8(248, 113, 113, 1.0),
            danger_bg: Color::from_rgba8(127, 29, 29, 0.92),
        }
    } else {
        VariablesPalette {
            backdrop: Color::from_rgba8(15, 23, 42, 0.08),
            panel_bg: Color::from_rgba8(249, 249, 250, 0.99),
            panel_border: Color::from_rgba8(228, 228, 231, 1.0),
            panel_shadow: Color::BLACK.scale_alpha(0.16),
            title: Color::from_rgba8(39, 39, 42, 1.0),
            subtitle: Color::from_rgba8(113, 113, 122, 1.0),
            header_text: Color::from_rgba8(82, 82, 91, 1.0),
            row_divider: Color::from_rgba8(228, 228, 231, 0.9),
            name_bg: Color::from_rgba8(244, 244, 245, 1.0),
            name_border: Color::from_rgba8(228, 228, 231, 1.0),
            name_text: Color::from_rgba8(39, 39, 42, 1.0),
            cell_bg: Color::from_rgba8(244, 244, 245, 1.0),
            cell_border: Color::from_rgba8(228, 228, 231, 1.0),
            cell_text: Color::from_rgba8(39, 39, 42, 1.0),
            menu_bg: Color::from_rgba8(255, 255, 255, 0.99),
            menu_border: Color::from_rgba8(228, 228, 231, 1.0),
            menu_hover_bg: Color::from_rgba8(244, 244, 245, 1.0),
            menu_text: Color::from_rgba8(39, 39, 42, 1.0),
            danger_text: Color::from_rgba8(185, 28, 28, 1.0),
            danger_bg: Color::from_rgba8(220, 38, 38, 0.92),
        }
    }
}
#[cfg(test)]
#[path = "styles_tests.rs"]
mod styles_tests;
