//! 应用内短消息提示的状态模型与渲染控件。
//!
//! 本模块主要负责把应用状态转换为桌面端可渲染的 Iced 控件，并把用户操作映射回上层消息。
//! 注释聚焦调用边界、返回值和错误传播方式，便于后续维护设置页与工具栏行为时快速定位职责。

use crate::app::assets::{self, Icon};
use crate::app::components::system_settings_common::settings_panel_style;
use crate::app::state::ToastKind;
use crate::app::{App, Message};
use iced::widget::svg::{self, Svg};
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme, Vector};

fn is_dark_theme(theme: &Theme) -> bool {
    let palette = theme.palette();
    palette.background.r + palette.background.g + palette.background.b < 1.5
}

fn icon_svg(icon: Icon, size: f32, color: Color) -> Svg<'static> {
    Svg::new(assets::get_icon(icon))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_theme: &Theme, _status| svg::Style { color: Some(color) })
}

/// 构建或处理 `view` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub fn view(app: &App) -> Element<'_, Message> {
    let Some(toast) = &app.active_toast else {
        return container(text("")).width(Length::Fixed(0.0)).height(Length::Fixed(0.0)).into();
    };

    let (icon, accent, title) = match toast.kind {
        ToastKind::Success => (Icon::Check, Color::from_rgb8(0x28, 0x8F, 0x61), "操作已完成"),
        ToastKind::Info => (Icon::QuestionCircle, Color::from_rgb8(0x2A, 0x6F, 0xC2), "提示"),
        ToastKind::Error => (Icon::X, Color::from_rgb8(0xC5, 0x3E, 0x3E), "需要处理"),
    };

    let icon_badge = container(icon_svg(icon, 14.0, accent))
        .width(Length::Fixed(34.0))
        .height(Length::Fixed(34.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(if is_dark_theme(theme) {
                accent.scale_alpha(0.18)
            } else {
                accent.scale_alpha(0.12)
            })),
            border: Border {
                width: 1.0,
                color: accent.scale_alpha(if is_dark_theme(theme) { 0.36 } else { 0.18 }),
                radius: 999.0.into(),
            },
            ..Default::default()
        });

    let content = row![
        icon_badge,
        column![
            text(title).size(11).style(move |_theme: &Theme| iced::widget::text::Style {
                color: Some(accent),
            }),
            text(&toast.message).size(13).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(if is_dark_theme(theme) {
                    theme.extended_palette().background.base.text.scale_alpha(0.94)
                } else {
                    theme.palette().text.scale_alpha(0.88)
                }),
            })
        ]
        .spacing(3)
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(content)
        .padding([12, 14])
        .style(move |theme: &Theme| {
            let mut style = settings_panel_style(theme);
            let is_dark = is_dark_theme(theme);
            style.background = Some(Background::Color(if is_dark {
                theme.extended_palette().background.base.color.scale_alpha(0.97)
            } else {
                Color::WHITE.scale_alpha(0.96)
            }));
            style.border = Border {
                width: 1.0,
                color: accent.scale_alpha(if is_dark { 0.38 } else { 0.16 }),
                radius: 18.0.into(),
            };
            style.shadow = iced::Shadow {
                color: Color::BLACK.scale_alpha(if is_dark { 0.18 } else { 0.08 }),
                offset: Vector::new(0.0, 14.0),
                blur_radius: 26.0,
            };
            style
        })
        .width(Length::Shrink)
        .into()
}
