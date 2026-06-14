//! 渲染应用视图中的模态窗口。
//! 本模块只描述模态内容和交互消息，不直接承担持久化策略。

use crate::app::state::ProjectEditTab;
use iced::widget::{button, text};
use iced::{Background, Color, Element, Theme};

use super::super::Message;

/// 模块内可见函数，执行 parse_hex_color 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn parse_hex_color(input: &str) -> Option<Color> {
    let hex = input.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::from_rgb8(red, green, blue))
}

/// 模块内可见函数，执行 format_hex_color 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn format_hex_color(color: Color) -> String {
    let red = (color.r * 255.0).round() as u8;
    let green = (color.g * 255.0).round() as u8;
    let blue = (color.b * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", red, green, blue)
}

/// 模块内可见函数，执行 icon_image_handle 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn icon_image_handle(icon: &str) -> Option<iced::widget::image::Handle> {
    let raw = icon.trim();
    if raw.is_empty() {
        return None;
    }

    let path = raw.strip_prefix("file://").unwrap_or(raw);
    let path =
        path.strip_prefix("//").map(|rest| format!("/{rest}")).unwrap_or_else(|| path.to_string());
    let path = std::path::Path::new(&path);
    if path.exists() { Some(iced::widget::image::Handle::from_path(path)) } else { None }
}

/// 模块内可见函数，执行 tab_button 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn tab_button(
    label: &'static str,
    tab: ProjectEditTab,
    selected: bool,
) -> Element<'static, Message> {
    button(text(label).size(12))
        .on_press(Message::Project(
            crate::app::message::project::ProjectMessage::ProjectEditTabSelected(tab),
        ))
        .padding([6, 12])
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;
            let background = if selected {
                Some(Background::Color(if is_dark {
                    theme.palette().primary.scale_alpha(0.18)
                } else {
                    theme.palette().primary.scale_alpha(0.10)
                }))
            } else if matches!(status, iced::widget::button::Status::Hovered) {
                Some(Background::Color(if is_dark {
                    palette.background.weak.color.scale_alpha(0.84)
                } else {
                    Color::WHITE.scale_alpha(0.92)
                }))
            } else {
                Some(Background::Color(if is_dark {
                    palette.background.base.color.scale_alpha(0.56)
                } else {
                    Color::WHITE.scale_alpha(0.78)
                }))
            };
            iced::widget::button::Style {
                background,
                text_color: if selected { theme.palette().primary } else { theme.palette().text },
                border: iced::Border {
                    radius: 999.0.into(),
                    width: 1.0,
                    color: if selected {
                        theme.palette().primary.scale_alpha(0.45)
                    } else {
                        palette.background.strong.color.scale_alpha(0.62)
                    },
                },
                shadow: if selected {
                    iced::Shadow {
                        color: theme.palette().primary.scale_alpha(if is_dark {
                            0.18
                        } else {
                            0.08
                        }),
                        offset: iced::Vector::new(0.0, 8.0),
                        blur_radius: 18.0,
                    }
                } else {
                    iced::Shadow::default()
                },
                ..Default::default()
            }
        })
        .into()
}
#[cfg(test)]
#[path = "common_tests.rs"]
mod common_tests;
