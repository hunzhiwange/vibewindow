//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, column, container};
use iced::{Element, Length};

use crate::app::message::DesignMessage;
use crate::app::views::design::models::Effect;
use crate::app::views::design::properties::{appearance, color_picker, fill};
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};

use super::shared::design_overlay_surface_shadow;

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `state`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn fill_picker_layers<'a>(app: &'a App, state: &'a DesignState) -> Vec<Element<'a, Message>> {
    let Some(picker) = &app.active_fill_picker else {
        return vec![];
    };

    let mut layers: Vec<Element<'a, Message>> = vec![];

    if !picker.picking {
        let overlay = iced::widget::MouseArea::new(
            container(Space::new()).width(Length::Fill).height(Length::Fill).style(|_| {
                container::Style {
                    background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.01).into()),
                    ..Default::default()
                }
            }),
        )
        .on_press(Message::Design(DesignMessage::CloseFillPicker));
        layers.push(overlay.into());
    }

    let window_w = app.window_size.0;
    let window_h = app.window_size.1;
    let picker_w = 360.0;
    let picker_h = 520.0;
    let gap = 8.0;
    let max_x = (window_w - picker_w).max(0.0);
    let max_y = (window_h - picker_h).max(0.0);

    let mut x = picker.position.x - picker_w - gap;
    if x < 0.0 {
        x = (picker.position.x + gap).min(max_x);
    } else if x > max_x {
        x = max_x;
    }

    let mut y = picker.position.y - picker_h / 2.0;
    y = y.clamp(0.0, max_y);

    let content: Element<'a, Message> = if let Some(el) = state.doc.find_element(&picker.element_id)
    {
        fill::render_popover(
            el,
            picker.fill_index,
            picker.format,
            picker.picking,
            state.pan,
            state.zoom,
        )
    } else {
        column![].into()
    };

    let popover_content = iced::widget::MouseArea::new(
        container(content)
            .style(|theme: &iced::Theme| container::Style {
                background: Some(theme.palette().background.into()),
                border: iced::Border {
                    color: theme.palette().primary,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: design_overlay_surface_shadow(theme, 0.30, 0.18),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 10.0,
                },
                ..Default::default()
            })
            .padding(10)
            .width(Length::Fixed(picker_w - 20.0)),
    )
    .on_press(Message::None);

    layers.push(
        container(popover_content)
            .padding(iced::Padding { top: y, left: x, ..Default::default() })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .into(),
    );

    layers
}

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `state`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn effect_picker_layers<'a>(app: &'a App, state: &'a DesignState) -> Vec<Element<'a, Message>> {
    let Some(picker) = &app.active_effect_picker else {
        return vec![];
    };

    let mut layers: Vec<Element<'a, Message>> = vec![];
    let overlay = iced::widget::MouseArea::new(
        container(Space::new()).width(Length::Fill).height(Length::Fill).style(|_| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.01).into()),
                ..Default::default()
            }
        }),
    )
    .on_press(Message::Design(DesignMessage::CloseEffectPicker));
    layers.push(overlay.into());

    let window_w = app.window_size.0;
    let window_h = app.window_size.1;
    let picker_w = 340.0;
    let mut picker_h = 360.0;
    let gap = 8.0;
    let max_x = (window_w - picker_w).max(0.0);

    let mut x = picker.position.x - picker_w - gap;
    if x < 0.0 {
        x = (picker.position.x + gap).min(max_x);
    } else if x > max_x {
        x = max_x;
    }

    let content: Element<'a, Message> = if let Some(el) = state.doc.find_element(&picker.element_id)
    {
        if let Some(value) = &el.effect {
            let effects: Vec<Effect> = serde_json::from_value::<Vec<Effect>>(value.clone())
                .or_else(|_| serde_json::from_value::<Effect>(value.clone()).map(|e| vec![e]))
                .unwrap_or_default();

            if let Some(effect) = effects.get(picker.effect_index) {
                picker_h = match effect.kind.as_str() {
                    "shadow" => 420.0,
                    "layer_blur" | "background_blur" => 320.0,
                    _ => 360.0,
                };
            }
        }

        appearance::render_popover(el, picker.effect_index)
    } else {
        column![].into()
    };

    let max_y = (window_h - picker_h).max(0.0);
    let mut y = picker.position.y - picker_h / 2.0;
    y = y.clamp(0.0, max_y);

    let popover_content = iced::widget::MouseArea::new(
        container(content)
            .style(|theme: &iced::Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.base.color.into()),
                    border: iced::Border {
                        color: p.background.strong.color,
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: design_overlay_surface_shadow(theme, 0.26, 0.20),
                        offset: iced::Vector::new(0.0, 8.0),
                        blur_radius: 20.0,
                    },
                    ..Default::default()
                }
            })
            .padding(10)
            .width(Length::Fixed(picker_w))
            .height(Length::Fixed(picker_h)),
    )
    .on_press(Message::None);

    layers.push(
        container(popover_content)
            .padding(iced::Padding { top: y, left: x, ..Default::default() })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .into(),
    );

    layers
}

/// 计算颜色表现。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn color_picker_layers<'a>(app: &'a App) -> Vec<Element<'a, Message>> {
    let Some(picker) = &app.active_color_picker else {
        return vec![];
    };

    let mut layers: Vec<Element<'a, Message>> = vec![];

    if !picker.picking {
        let overlay = iced::widget::MouseArea::new(
            container(Space::new()).width(Length::Fill).height(Length::Fill).style(|_| {
                container::Style {
                    background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.01).into()),
                    ..Default::default()
                }
            }),
        )
        .on_press(Message::Design(DesignMessage::CloseColorPicker));
        layers.push(overlay.into());
    }

    let window_w = app.window_size.0;
    let window_h = app.window_size.1;
    let context_toolbar_picker = matches!(
        picker.target,
        crate::app::views::design::models::ColorPickerTarget::ContextFill { .. }
            | crate::app::views::design::models::ColorPickerTarget::ContextBorder { .. }
            | crate::app::views::design::models::ColorPickerTarget::ContextText { .. }
    );
    let picker_w = if context_toolbar_picker { 370.0 } else { 270.0 };
    let picker_h = if context_toolbar_picker { 560.0 } else { 520.0 };
    let gap = 8.0;
    let max_x = (window_w - picker_w).max(0.0);
    let max_y = (window_h - picker_h).max(0.0);

    let mut x = picker.position.x - picker_w - gap;
    if x < 0.0 {
        x = (picker.position.x + gap).min(max_x);
    } else if x > max_x {
        x = max_x;
    }

    let mut y = picker.position.y - picker_h / 2.0;
    y = y.clamp(0.0, max_y);

    let picker_content = iced::widget::MouseArea::new(
        container(color_picker::render_color_picker(
            picker.color,
            picker.format,
            picker.picking,
            |c| Message::Design(DesignMessage::ColorPickerChange(c)),
            |f| Message::Design(DesignMessage::ColorPickerFormatChange(f)),
            || Message::Design(DesignMessage::ColorPickerEyedropper),
        ))
        .style(move |theme: &iced::Theme| {
            if context_toolbar_picker {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.base.color.scale_alpha(0.99).into()),
                    border: iced::Border {
                        color: p.background.strong.color.scale_alpha(0.72),
                        width: 1.0,
                        radius: 24.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: design_overlay_surface_shadow(theme, 0.56, 0.24),
                        offset: iced::Vector::new(0.0, 12.0),
                        blur_radius: 36.0,
                    },
                    ..Default::default()
                }
            } else {
                container::Style {
                    background: Some(theme.palette().background.into()),
                    border: iced::Border {
                        color: theme.palette().primary,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: design_overlay_surface_shadow(theme, 0.30, 0.18),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 10.0,
                    },
                    ..Default::default()
                }
            }
        })
        .padding(if context_toolbar_picker { 14 } else { 10 })
        .width(if context_toolbar_picker { 340 } else { 250 }),
    )
    .on_press(Message::None);

    layers.push(
        container(picker_content)
            .padding(iced::Padding { top: y, left: x, ..Default::default() })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .into(),
    );

    layers
}
#[cfg(test)]
#[path = "overlay_basic_pickers_tests.rs"]
mod overlay_basic_pickers_tests;
