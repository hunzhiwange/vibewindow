//! # 设计视图样式辅助模块
//!
//! 本模块抽离设计器视图内部多个子模块共享的样式函数与颜色判断逻辑。
//! 顶层视图只保留布局编排，具体视觉细节由这里统一维护，避免多个子模块重复定义。

use iced::widget::{button, container, text_editor, text_input};
use iced::{Background, Border, Color, Theme};

pub(super) fn design_input_style(theme: &Theme, _status: text_input::Status) -> text_input::Style {
    let p = theme.extended_palette();
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border { width: 1.0, color: p.background.strong.color, radius: 10.0.into() },
        icon: theme.palette().text,
        placeholder: p.background.base.text.scale_alpha(0.7),
        value: theme.palette().text,
        selection: theme.palette().primary,
    }
}

pub(super) fn design_editor_style(
    theme: &Theme,
    _status: text_editor::Status,
) -> text_editor::Style {
    let p = theme.extended_palette();
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    let placeholder = if is_dark {
        palette.text.scale_alpha(0.42)
    } else {
        p.secondary.base.text.scale_alpha(0.56)
    };
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
        value: theme.palette().text,
        selection: theme.palette().primary,
        placeholder,
    }
}

pub(super) fn design_popover_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            theme.palette().background.r,
            theme.palette().background.g,
            theme.palette().background.b,
            0.98,
        ))),
        border: Border { radius: 8.0.into(), width: 1.0, color: p.background.strong.color },
        ..Default::default()
    }
}

pub(super) fn design_soft_popover_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            theme.palette().background.r,
            theme.palette().background.g,
            theme.palette().background.b,
            0.98,
        ))),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: p.background.strong.color.scale_alpha(0.38),
        },
        ..Default::default()
    }
}

pub(super) fn design_tooltip_dark_style(_theme: &Theme) -> container::Style {
    let theme = _theme;
    let p = theme.extended_palette();
    let background = if design_is_dark(theme) {
        p.background.strong.color.scale_alpha(0.96)
    } else {
        Color::from_rgba(
            p.background.base.color.r,
            p.background.base.color.g,
            p.background.base.color.b,
            0.96,
        )
    };
    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(p.background.base.text),
        border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
        shadow: iced::Shadow {
            color: if design_is_dark(theme) {
                Color::BLACK.scale_alpha(0.34)
            } else {
                p.background.strong.color.scale_alpha(0.18)
            },
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

pub(super) fn design_pill_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => {
            Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
        }
        button::Status::Pressed => Some(Background::Color(p.background.strong.color)),
        _ => None,
    };
    button::Style {
        background: bg,
        border: Border { radius: 10.0.into(), width: 0.0, color: Color::TRANSPARENT },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

pub(super) fn design_square_icon_button_style(
    theme: &Theme,
    status: button::Status,
) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => {
            Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
        }
        button::Status::Pressed => {
            Some(Background::Color(p.background.strong.color.scale_alpha(0.45)))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        border: Border { radius: 6.0.into(), width: 0.0, color: Color::TRANSPARENT },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

pub(super) fn design_round_icon_button_style(
    theme: &Theme,
    status: button::Status,
) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => {
            Some(Background::Color(p.background.weak.color.scale_alpha(0.28)))
        }
        button::Status::Pressed => {
            Some(Background::Color(p.background.strong.color.scale_alpha(0.36)))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        border: Border { radius: 999.0.into(), width: 0.0, color: Color::TRANSPARENT },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

pub(super) fn design_is_dark(theme: &Theme) -> bool {
    let background = theme.palette().background;
    background.r + background.g + background.b < 1.5
}

pub(super) fn design_contrast_text_color(background: Color) -> Color {
    if background.r + background.g + background.b > 1.5 { Color::BLACK } else { Color::WHITE }
}
#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;
