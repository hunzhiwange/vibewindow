//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, button, column, container, scrollable, text, text_input};
use iced::{Element, Length};
use serde_json::Value;

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::find_element_by_id;
use crate::app::views::design::canvas::tailwind::get_tailwind_classes;
use crate::app::views::design::properties::{
    group_tailwind_class, split_class_tokens, typography::available_system_fonts,
};
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};

use super::shared::{design_overlay_contrast_text_color, design_overlay_surface_shadow};

/// 处理字体相关状态。
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
pub fn font_picker_layers<'a>(app: &'a App, state: &'a DesignState) -> Vec<Element<'a, Message>> {
    let Some(picker) = &app.active_font_picker else {
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
    .on_press(Message::Design(DesignMessage::CloseFontPicker));
    layers.push(overlay.into());

    let window_w = app.window_size.0;
    let window_h = app.window_size.1;
    let picker_w = 320.0;
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

    let current_family = state
        .doc
        .find_element(&picker.element_id)
        .and_then(|el| el.font_family.clone())
        .unwrap_or_default();

    let needle = app.font_filter_query.to_ascii_lowercase();
    let fonts = available_system_fonts();
    let filtered: Vec<String> = if needle.is_empty() {
        fonts
    } else {
        fonts.into_iter().filter(|f| f.to_ascii_lowercase().contains(&needle)).collect()
    };

    let input_style = |theme: &iced::Theme, status: text_input::Status| {
        let palette = theme.palette();
        let extended = theme.extended_palette();
        let focused = matches!(status, text_input::Status::Focused { .. });
        let border_color = if focused { palette.primary } else { extended.background.strong.color };
        let bg =
            if focused { extended.background.weak.color } else { extended.background.base.color };
        text_input::Style {
            background: iced::Background::Color(bg),
            border: iced::Border { width: 1.0, color: border_color, radius: 8.0.into() },
            icon: palette.text.scale_alpha(0.5),
            placeholder: palette.text.scale_alpha(0.55),
            value: palette.text,
            selection: palette.primary.scale_alpha(0.30),
        }
    };

    let mut list = column![].spacing(0);
    for family in filtered {
        let is_selected = family == current_family;
        let element_id = picker.element_id.clone();
        let label = family.clone();
        list = list.push(
            button(container(text(label).size(14)).width(Length::Fill).padding([8, 10]))
                .on_press(Message::Design(DesignMessage::FontPickerSelect(element_id, family)))
                .style(move |theme: &iced::Theme, status| {
                    let ext = theme.extended_palette();
                    let bg = if is_selected {
                        theme.palette().primary
                    } else if status == button::Status::Hovered {
                        ext.background.weak.color
                    } else {
                        ext.background.base.color
                    };
                    button::Style {
                        background: Some(bg.into()),
                        text_color: if is_selected {
                            design_overlay_contrast_text_color(bg)
                        } else {
                            theme.palette().text
                        },
                        border: iced::Border {
                            color: ext.background.strong.color,
                            width: 0.0,
                            radius: 0.0.into(),
                        },
                        ..button::Style::default()
                    }
                })
                .padding(0),
        );
    }

    let content = iced::widget::MouseArea::new(
        container(
            column![
                text_input("Search fonts...", &app.font_filter_query)
                    .on_input(|s| Message::Design(DesignMessage::SetFontFilter(s)))
                    .style(input_style)
                    .padding(6)
                    .size(12),
                container(scrollable(list).height(Length::Fill))
                    .style(|theme: &iced::Theme| {
                        let ext = theme.extended_palette();
                        container::Style {
                            background: Some(ext.background.base.color.into()),
                            border: iced::Border {
                                color: ext.background.strong.color,
                                width: 1.0,
                                radius: 10.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .height(Length::Fill)
            ]
            .spacing(10),
        )
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                border: iced::Border {
                    color: palette.background.strong.color,
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
        .width(Length::Fixed(picker_w)),
    )
    .on_press(Message::None);

    layers.push(
        container(content)
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
pub fn tailwind_class_picker_layers<'a>(
    app: &'a App,
    state: &'a DesignState,
) -> Vec<Element<'a, Message>> {
    let Some(picker) = &app.active_tailwind_class_picker else {
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
    .on_press(Message::Design(DesignMessage::CloseTailwindClassPicker));
    layers.push(overlay.into());

    let window_w = app.window_size.0;
    let window_h = app.window_size.1;
    let picker_w = 360.0;
    let picker_h = 560.0;
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

    let current_class = find_element_by_id(&state.doc.children, &picker.element_id)
        .and_then(|el| el.class.clone())
        .unwrap_or_default();
    let tokens = split_class_tokens(&current_class);

    let needle = app.tailwind_filter_query.trim().to_ascii_lowercase();
    let classes = get_tailwind_classes();
    let filtered: Vec<String> = if needle.is_empty() {
        classes
    } else {
        classes.into_iter().filter(|c| c.to_ascii_lowercase().contains(&needle)).collect()
    };

    let input_style = |theme: &iced::Theme, status: text_input::Status| {
        let palette = theme.palette();
        let extended = theme.extended_palette();
        let focused = matches!(status, text_input::Status::Focused { .. });
        let border_color = if focused { palette.primary } else { extended.background.strong.color };
        let bg =
            if focused { extended.background.weak.color } else { extended.background.base.color };
        text_input::Style {
            background: iced::Background::Color(bg),
            border: iced::Border { width: 1.0, color: border_color, radius: 8.0.into() },
            icon: palette.text.scale_alpha(0.5),
            placeholder: palette.text.scale_alpha(0.55),
            value: palette.text,
            selection: palette.primary.scale_alpha(0.30),
        }
    };

    let mut list = column![].spacing(0);
    let mut current_group: Option<&'static str> = None;
    for class_name in filtered {
        let group = group_tailwind_class(&class_name);
        if current_group != Some(group) {
            current_group = Some(group);
            list = list.push(
                container(text(group).size(11).style(iced::widget::text::secondary))
                    .width(Length::Fill)
                    .padding(iced::Padding { top: 10.0, right: 10.0, bottom: 6.0, left: 10.0 }),
            );
        }

        let is_selected = tokens.iter().any(|t| t == &class_name);
        let element_id = picker.element_id.clone();
        let new_value = if is_selected {
            tokens.iter().filter(|t| *t != &class_name).cloned().collect::<Vec<_>>().join(" ")
        } else {
            let mut next = tokens.clone();
            next.push(class_name.clone());
            next.join(" ")
        };
        let msg = Message::Design(DesignMessage::PropertyUpdate(
            element_id,
            "class".to_string(),
            if new_value.trim().is_empty() { Value::Null } else { Value::String(new_value) },
        ));

        let label = class_name.clone();
        list = list.push(
            button(container(text(label).size(14)).width(Length::Fill).padding([8, 10]))
                .on_press(msg)
                .style(move |theme: &iced::Theme, status| {
                    let ext = theme.extended_palette();
                    let bg = if is_selected {
                        theme.palette().primary
                    } else if status == button::Status::Hovered {
                        ext.background.weak.color
                    } else {
                        ext.background.base.color
                    };
                    button::Style {
                        background: Some(bg.into()),
                        text_color: if is_selected {
                            design_overlay_contrast_text_color(bg)
                        } else {
                            theme.palette().text
                        },
                        border: iced::Border {
                            color: ext.background.strong.color,
                            width: 0.0,
                            radius: 0.0.into(),
                        },
                        ..button::Style::default()
                    }
                })
                .padding(0),
        );
    }

    let content = iced::widget::MouseArea::new(
        container(
            column![
                text_input("搜索 Tailwind 类...", &app.tailwind_filter_query)
                    .on_input(|s| Message::Design(DesignMessage::SetTailwindFilter(s)))
                    .style(input_style)
                    .padding(6)
                    .size(12),
                container(scrollable(list).height(Length::Fill))
                    .style(|theme: &iced::Theme| {
                        let ext = theme.extended_palette();
                        container::Style {
                            background: Some(ext.background.base.color.into()),
                            border: iced::Border {
                                color: ext.background.strong.color,
                                width: 1.0,
                                radius: 10.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .height(Length::Fill)
            ]
            .spacing(10),
        )
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                border: iced::Border {
                    color: palette.background.strong.color,
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
        .width(Length::Fixed(picker_w)),
    )
    .on_press(Message::None);

    layers.push(
        container(content)
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
#[path = "overlay_font_and_tailwind_tests.rs"]
mod overlay_font_and_tailwind_tests;
