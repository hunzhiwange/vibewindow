//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    Space, button, column, container, image, row, scrollable, svg, text, text_input, tooltip,
};
use iced::{Color, Element, Length};

use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::properties::icon::icon_display_name;
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};

use super::shared::{
    OVERLAY_ICON_PICKER_REQUIRE_QUERY_THRESHOLD, OVERLAY_ICON_PICKER_RESULT_LIMIT,
    design_overlay_contrast_text_color, design_overlay_surface_shadow,
};

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
pub fn icon_picker_layers<'a>(app: &'a App, state: &'a DesignState) -> Vec<Element<'a, Message>> {
    let Some(picker) = &app.active_icon_picker else {
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
    .on_press(Message::Design(DesignMessage::CloseIconPicker));
    layers.push(overlay.into());

    let window_w = app.window_size.0;
    let window_h = app.window_size.1;
    let picker_w = 430.0;
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

    let current_element = state.doc.find_element(&picker.element_id);
    let current_family = current_element
        .and_then(|element| element.icon_font_family.as_deref())
        .and_then(assets::canonical_named_icon_family)
        .unwrap_or_else(|| "lucide".to_string());
    let current_icon_name = current_element
        .and_then(|element| element.icon_font_name.clone())
        .unwrap_or_else(|| "star".to_string());
    let current_weight = current_element.and_then(|element| element.weight.clone());
    let catalog = assets::named_icon_catalog();
    let active_family = catalog
        .iter()
        .find(|entry| entry.family == app.icon_picker_family_tab)
        .map(|entry| entry.family.as_str())
        .or_else(|| catalog.first().map(|entry| entry.family.as_str()))
        .unwrap_or("lucide");
    let query = app.icon_picker_filter_query.trim().to_ascii_lowercase();

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

    let family_button_style = |active: bool| {
        move |theme: &iced::Theme, status: button::Status| {
            let ext = theme.extended_palette();
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let background = if active {
                theme.palette().primary
            } else if hovered {
                ext.background.weak.color
            } else {
                ext.background.base.color
            };
            button::Style {
                background: Some(background.into()),
                text_color: if active {
                    design_overlay_contrast_text_color(background)
                } else {
                    theme.palette().text
                },
                border: iced::Border {
                    color: if active {
                        theme.palette().primary
                    } else {
                        ext.background.strong.color
                    },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..button::Style::default()
            }
        }
    };

    let icon_button_style = |active: bool| {
        move |theme: &iced::Theme, status: button::Status| {
            let ext = theme.extended_palette();
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let background = if active {
                theme.palette().primary.scale_alpha(0.18)
            } else if hovered {
                ext.background.weak.color
            } else {
                ext.background.base.color
            };
            button::Style {
                background: Some(background.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    color: if active {
                        theme.palette().primary
                    } else {
                        ext.background.strong.color
                    },
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..button::Style::default()
            }
        }
    };

    let mut family_list = column![].spacing(6);
    for entry in catalog {
        let is_selected = entry.family == active_family;
        family_list = family_list.push(
            button(text(assets::named_icon_family_label(&entry.family)).size(12))
                .padding([6, 10])
                .width(Length::Fill)
                .style(family_button_style(is_selected))
                .on_press(Message::Design(DesignMessage::SetIconPickerFamilyTab(
                    entry.family.clone(),
                ))),
        );
    }

    let active_icons = catalog
        .iter()
        .find(|entry| entry.family == active_family)
        .map(|entry| entry.icons.as_slice())
        .unwrap_or(&[]);
    let filtered_icons = active_icons
        .iter()
        .filter(|name| query.is_empty() || name.contains(&query))
        .cloned()
        .collect::<Vec<_>>();
    let require_query =
        query.is_empty() && active_icons.len() > OVERLAY_ICON_PICKER_REQUIRE_QUERY_THRESHOLD;
    let visible_icons = if require_query {
        Vec::new()
    } else {
        filtered_icons
            .iter()
            .take(OVERLAY_ICON_PICKER_RESULT_LIMIT)
            .cloned()
            .collect::<Vec<_>>()
    };
    let result_summary = if require_query {
        format!("{} 个图标，请先输入名称搜索", active_icons.len())
    } else if filtered_icons.len() > visible_icons.len() {
        format!(
            "显示前 {} / {} 个结果，请继续输入缩小范围",
            visible_icons.len(),
            filtered_icons.len()
        )
    } else {
        format!("{} 个图标", filtered_icons.len())
    };

    let tooltip_style = |theme: &iced::Theme| {
        let palette = theme.palette();
        let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
        container::Style {
            background: Some(
                if is_dark {
                    Color::from_rgba8(32, 35, 39, 0.98)
                } else {
                    Color::from_rgba8(255, 255, 255, 0.98)
                }
                .into(),
            ),
            text_color: Some(theme.palette().text),
            border: iced::Border {
                color: if is_dark {
                    Color::from_rgba8(255, 255, 255, 0.12)
                } else {
                    Color::from_rgba8(0, 0, 0, 0.08)
                },
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.16),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        }
    };

    let mut icon_grid = column![].spacing(8);
    for chunk in visible_icons.chunks(4) {
        let mut card_row = row![].spacing(8);
        for name in chunk {
            let selected = active_family == current_family && name == &current_icon_name;
            let preview_color = if selected {
                Color::from_rgba8(37, 99, 235, 1.0)
            } else {
                Color::from_rgba8(75, 85, 99, 1.0)
            };
            let preview: Element<'a, Message> = if let Some(handle) =
                assets::get_named_icon_image_with_weight(
                    active_family,
                    name,
                    current_weight.as_ref(),
                    preview_color,
                ) {
                image(handle).width(Length::Fixed(20.0)).height(Length::Fixed(20.0)).into()
            } else {
                svg(assets::get_icon(Icon::Star))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .style(move |_theme: &iced::Theme, _status| iced::widget::svg::Style {
                        color: Some(preview_color),
                    })
                    .into()
            };
            let label = icon_display_name(name);
            let item = button(
                container(
                    text(label.clone())
                        .size(10)
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .height(Length::Fixed(46.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .width(Length::Fill)
            .padding([8, 6])
            .style(icon_button_style(selected))
            .on_press(Message::Design(DesignMessage::IconPickerSelect {
                element_id: picker.element_id.clone(),
                family: active_family.to_string(),
                name: name.clone(),
            }));
            let hover_preview = tooltip::Tooltip::new(
                item,
                container(
                    column![
                        container(preview)
                            .width(Length::Fixed(36.0))
                            .height(Length::Fixed(36.0))
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                        text(label).size(11)
                    ]
                    .spacing(6)
                    .align_x(iced::Alignment::Center),
                )
                .padding([8, 10])
                .style(tooltip_style),
                tooltip::Position::Right,
            )
            .gap(8.0);
            card_row = card_row.push(container(hover_preview).width(Length::FillPortion(1)));
        }
        icon_grid = icon_grid.push(card_row);
    }

    if require_query {
        icon_grid = icon_grid.push(
            container(text("图标数量过多，请先输入图标名称搜索").size(12))
                .width(Length::Fill)
                .padding([20, 0])
                .align_x(iced::alignment::Horizontal::Center),
        );
    } else if filtered_icons.is_empty() {
        icon_grid = icon_grid.push(
            container(text("没有匹配图标").size(12))
                .width(Length::Fill)
                .padding([20, 0])
                .align_x(iced::alignment::Horizontal::Center),
        );
    }

    let content = iced::widget::MouseArea::new(
        container(
            column![
                text_input("搜索图标...", &app.icon_picker_filter_query)
                    .on_input(|value| Message::Design(DesignMessage::SetIconPickerFilter(value)))
                    .style(input_style)
                    .padding(6)
                    .size(12),
                text(result_summary).size(11).style(iced::widget::text::secondary),
                row![
                    container(
                        scrollable(family_list)
                            .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
                            .height(Length::Fill)
                    )
                    .width(Length::Fixed(120.0)),
                    scrollable(icon_grid)
                        .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
                        .height(Length::Fill)
                        .width(Length::Fill)
                ]
                .spacing(10)
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
        .width(Length::Fixed(picker_w))
        .height(Length::Fixed(picker_h)),
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
#[path = "overlay_icon_tests.rs"]
mod overlay_icon_tests;
