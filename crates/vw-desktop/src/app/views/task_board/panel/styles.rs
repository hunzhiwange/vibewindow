//! 任务看板面板样式集合，统一维护按钮、弹层和容器的视觉规则。

use iced::widget::{container, text, text_editor, text_input};
use iced::{Background, Border, Color, Theme};

use crate::app::Message;
use crate::app::components::system_settings_common::{
    round_icon_btn_style, settings_muted_text_style, settings_panel_style,
    settings_text_editor_style, settings_text_input_style,
};

/// 构建或更新 input label 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn input_label(label: &str) -> iced::Element<'_, Message> {
    text(label).size(12).style(settings_muted_text_style).into()
}

/// 构建或更新 input style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    settings_text_input_style(theme, status)
}

/// 构建或更新 editor style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn editor_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    settings_text_editor_style(theme, status)
}

/// 构建或更新 pill button style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn pill_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let p = theme.extended_palette();
    let is_dark = theme.palette().background.r + theme.palette().background.g + theme.palette().background.b
        < 1.5;
    let bg = match status {
        iced::widget::button::Status::Hovered => Some(Background::Color(if is_dark {
            p.background.weak.color.scale_alpha(0.88)
        } else {
            Color::WHITE.scale_alpha(0.96)
        })),
        iced::widget::button::Status::Pressed => {
            Some(Background::Color(p.background.strong.color.scale_alpha(if is_dark { 0.92 } else { 0.28 })))
        }
        _ => Some(Background::Color(if is_dark {
            p.background.base.color.scale_alpha(0.72)
        } else {
            Color::WHITE.scale_alpha(0.84)
        })),
    };
    iced::widget::button::Style {
        background: bg,
        border: Border {
            radius: 14.0.into(),
            width: 1.0,
            color: if is_dark {
                p.background.strong.color.scale_alpha(0.82)
            } else {
                Color::from_rgba8(15, 23, 42, 0.08)
            },
        },
        text_color: theme.palette().text,
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(if is_dark { 0.14 } else { 0.06 }),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

/// 构建或更新 popover style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn popover_style(theme: &Theme) -> container::Style {
    let mut style = settings_panel_style(theme);
    style.border.radius = 18.0.into();
    style
}

/// 构建或更新 tooltip dark style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn tooltip_dark_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(24, 24, 24, 0.96))),
        text_color: Some(Color::WHITE),
        border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(0.40),
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

/// 构建或更新 square icon button style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn square_icon_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
    enabled: bool,
) -> iced::widget::button::Style {
    let mut style = round_icon_btn_style(theme, status);
    if !enabled {
        style.text_color = theme.extended_palette().background.weak.text;
    }
    style.border.radius = 12.0.into();
    style
}

/// 构建或更新 subtask card style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn subtask_card_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(if theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5
        {
            p.background.weak.color.scale_alpha(0.18)
        } else {
            Color::WHITE.scale_alpha(0.82)
        })),
        border: Border {
            width: 1.0,
            color: p.background.strong.color.scale_alpha(0.72),
            radius: 16.0.into(),
        },
        ..Default::default()
    }
}

/// 构建或更新 subtask badge style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn subtask_badge_style(
    theme: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let p = theme.extended_palette();
    iced::widget::button::Style {
        background: Some(Background::Color(p.background.weak.color.scale_alpha(0.55))),
        border: Border {
            width: 1.0,
            color: p.background.strong.color.scale_alpha(0.78),
            radius: 999.0.into(),
        },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

/// 构建或更新 disabled arrow button style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn disabled_arrow_button_style(theme: &Theme) -> iced::widget::button::Style {
    let p = theme.extended_palette();
    iced::widget::button::Style {
        background: None,
        border: Border { radius: 4.0.into(), width: 0.0, color: Color::TRANSPARENT },
        text_color: p.background.weak.text,
        ..Default::default()
    }
}

/// 构建或更新 panel container style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn panel_container_style(theme: &Theme) -> container::Style {
    let mut style = settings_panel_style(theme);
    style.border.radius = Border {
        radius: 22.0.into(),
        ..style.border
    }
    .radius;
    style
}

#[cfg(test)]
#[path = "styles_tests.rs"]
mod styles_tests;
