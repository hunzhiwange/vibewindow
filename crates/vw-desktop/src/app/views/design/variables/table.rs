//! 设计变量面板模块，负责变量集合、主题模式和值编辑界面的拆分实现。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::font::Font;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, scrollable, svg, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::components::overlays::PointBelowOverlay;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{ColorPickerTarget, VariableDef};
use crate::app::views::design::state::DesignState;

use super::menus::{render_delete_confirm, render_move_targets_menu, render_variable_menu, render_variant_menu};
use super::styles::{ACTION_COL_WIDTH, NAME_COL_WIDTH, TABLE_GAP, VARIANT_COL_WIDTH, variable_value_input_style, variables_palette};
use super::utils::{color_alpha_input_value, color_hex_input_value, direct_variable_value, parse_hex_color, swatch_border_color, update_color_alpha_value, update_color_hex_value, variable_belongs_to_collection};

/// 渲染对应界面。
///
/// # 参数
/// - `state`: 当前视图构建所需的状态、配置或消息。
/// - `current_collection`: 当前视图构建所需的状态、配置或消息。
/// - `theme_modes`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_variable_table<'a>(
    state: &'a DesignState,
    current_collection: &str,
    theme_modes: &[String],
    label_font: Font,
) -> Element<'a, Message> {
    let mut table_content = column![].spacing(10).width(Length::Shrink);

    if state.doc.variables.is_empty() {
        table_content = table_content.push(
            container(text("还没有变量，点击左下角新增变量开始。").size(12).style(
                move |theme: &Theme| {
                    let palette = variables_palette(theme);
                    iced::widget::text::Style { color: Some(palette.subtitle) }
                },
            ))
            .padding([18, 12])
            .style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                container::Style {
                    background: Some(Background::Color(palette.name_bg)),
                    border: Border { radius: 12.0.into(), width: 1.0, color: palette.name_border },
                    ..Default::default()
                }
            }),
        );
    } else {
        let mut names: Vec<_> = state.doc.variables.keys().collect();
        names.sort();
        for name in names {
            if let Some(def) = state.doc.variables.get(name)
                && variable_belongs_to_collection(def, current_collection) {
                    table_content = table_content.push(render_variable_row(
                        state,
                        name.as_str(),
                        def,
                        theme_modes,
                        label_font,
                    ));
                }
        }
    }

    column![
        render_table_header(state, theme_modes, label_font),
        scrollable(table_content).height(Length::Fill).direction(Direction::Both {
            vertical: Scrollbar::new().width(4).scroller_width(4),
            horizontal: Scrollbar::new().width(4).scroller_width(4),
        })
    ]
    .spacing(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// 执行本模块的界面辅助逻辑。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn divider<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(move |theme: &Theme| {
            let palette = variables_palette(theme);
            container::Style {
                background: Some(Background::Color(palette.row_divider)),
                ..Default::default()
            }
        })
        .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `kind`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_kind_glyph(kind: &str) -> Element<'static, Message> {
    match kind.to_ascii_lowercase().as_str() {
        "color" => container(Space::new().width(Length::Fixed(9.0)).height(Length::Fixed(9.0)))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgba8(148, 163, 184, 0.92))),
                border: Border { radius: 999.0.into(), width: 0.0, color: Color::TRANSPARENT },
                ..Default::default()
            })
            .into(),
        "number" => text("#").size(11).into(),
        _ => text("T").size(11).into(),
    }
}

fn render_table_header<'a>(
    state: &'a DesignState,
    theme_modes: &[String],
    label_font: Font,
) -> Element<'a, Message> {
    let mut head = row![render_name_header(label_font), render_default_variant_header(label_font)]
        .spacing(TABLE_GAP)
        .align_y(Alignment::Center);

    for mode in theme_modes {
        head = head.push(render_variant_header(
            mode,
            state.active_variable_theme_menu.as_ref(),
            state.confirm_delete_variable_theme.as_ref(),
            label_font,
        ));
    }

    head = head
        .push(render_add_theme_column_button())
        .push(Space::new().width(Length::Fixed(ACTION_COL_WIDTH)));

    column![head, divider()].spacing(10).into()
}

fn render_name_header<'a>(label_font: Font) -> Element<'a, Message> {
    container(text("Name").size(11).font(label_font).style(move |theme: &Theme| {
        let palette = variables_palette(theme);
        iced::widget::text::Style { color: Some(palette.header_text) }
    }))
    .width(Length::Fixed(NAME_COL_WIDTH))
    .padding([0, 10])
    .into()
}

fn render_default_variant_header<'a>(label_font: Font) -> Element<'a, Message> {
    container(text("Default").size(11).font(label_font).style(move |theme: &Theme| {
        let palette = variables_palette(theme);
        iced::widget::text::Style { color: Some(palette.header_text) }
    }))
    .width(Length::Fixed(VARIANT_COL_WIDTH))
    .padding([0, 10])
    .into()
}

fn render_variant_header<'a>(
    label: &str,
    active_menu: Option<&String>,
    delete_target: Option<&String>,
    label_font: Font,
) -> Element<'a, Message> {
    let menu_open = active_menu.is_some_and(|value| value.eq_ignore_ascii_case(label));
    let confirm_delete = delete_target.is_some_and(|value| value.eq_ignore_ascii_case(label));

    let trigger = button(
        row![
            text(label.to_string()).size(11).font(label_font).style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                iced::widget::text::Style { color: Some(palette.header_text) }
            }),
            svg(assets::get_icon(Icon::ChevronDown)).width(10).height(10).style(
                move |theme: &Theme, _| {
                    let palette = variables_palette(theme);
                    svg::Style { color: Some(palette.header_text.scale_alpha(0.85)) }
                }
            )
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(move |theme: &Theme, status| {
        let palette = variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered || menu_open || confirm_delete {
                    palette.menu_hover_bg
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            text_color: palette.header_text,
            border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::ToggleVariableThemeMenu(label.to_string())));

    let host = container(trigger).width(Length::Fixed(VARIANT_COL_WIDTH)).padding([0, 2]);

    if menu_open || confirm_delete {
        let overlay = if confirm_delete {
            render_delete_confirm(
                "删除 Variant？",
                format!("{label} 列中的变量值会一并删除。"),
                Message::Design(DesignMessage::CancelDeleteVariableTheme),
                Message::Design(DesignMessage::ConfirmDeleteVariableTheme),
                label_font,
            )
        } else {
            render_variant_menu(label.to_string(), label_font)
        };
        PointBelowOverlay::new(host, overlay)
            .show(true)
            .gap(28.0)
            .on_close(Message::Design(DesignMessage::CloseVariableThemeMenu))
            .into()
    } else {
        host.into()
    }
}

fn render_add_theme_column_button<'a>() -> Element<'a, Message> {
    button(svg(assets::get_icon(Icon::Plus)).width(12).height(12).style(move |theme: &Theme, _| {
        let palette = variables_palette(theme);
        svg::Style { color: Some(palette.header_text) }
    }))
    .padding([6, 8])
    .style(move |theme: &Theme, status| {
        let palette = variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered { palette.menu_hover_bg } else { Color::TRANSPARENT }.into(),
            ),
            text_color: palette.header_text,
            border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::AddVariableTheme))
    .into()
}

fn render_variable_row<'a>(
    state: &'a DesignState,
    name: &'a str,
    def: &'a VariableDef,
    theme_modes: &[String],
    label_font: Font,
) -> Element<'a, Message> {
    let mut row_cells =
        row![render_name_cell(name, def)].spacing(TABLE_GAP).align_y(Alignment::Center);

    row_cells = row_cells.push(render_value_cell(name, def, None));
    for mode in theme_modes {
        row_cells = row_cells.push(render_value_cell(name, def, Some(mode.as_str())));
    }
    row_cells = row_cells.push(render_variable_menu_button(state, name, theme_modes, label_font));

    column![row_cells, divider()].spacing(10).width(Length::Shrink).into()
}

fn render_name_cell<'a>(name: &'a str, def: &'a VariableDef) -> Element<'a, Message> {
    container(
        row![
            render_kind_glyph(&def.kind),
            text(name).size(12).style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                iced::widget::text::Style { color: Some(palette.name_text) }
            })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Length::Fixed(NAME_COL_WIDTH))
    .padding([8, 10])
    .style(move |theme: &Theme| {
        let palette = variables_palette(theme);
        container::Style {
            background: Some(Background::Color(palette.name_bg)),
            border: Border { radius: 8.0.into(), width: 1.0, color: palette.name_border },
            ..Default::default()
        }
    })
    .into()
}

fn render_value_cell<'a>(
    name: &'a str,
    def: &'a VariableDef,
    mode: Option<&str>,
) -> Element<'a, Message> {
    let value = direct_variable_value(def, mode);
    let mode_owned = mode.map(ToString::to_string);
    let name_owned = name.to_string();

    let cell: Element<'a, Message> = if def.kind.eq_ignore_ascii_case("color") {
        let swatch = parse_hex_color(&value).unwrap_or(Color::TRANSPARENT);
        let hex_value = color_hex_input_value(&value);
        let alpha_value = color_alpha_input_value(&value);
        container(
            row![
                button(render_value_preview(&def.kind, &value))
                    .width(Length::Fixed(30.0))
                    .height(Length::Fixed(30.0))
                    .padding(0)
                    .style(move |_theme: &Theme, _status| button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            radius: 8.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT
                        },
                        ..Default::default()
                    })
                    .on_press(Message::Design(DesignMessage::OpenColorPicker(
                        swatch,
                        ColorPickerTarget::VariableValue {
                            variable_name: name_owned.clone(),
                            mode: mode_owned.clone(),
                        },
                        None,
                    ))),
                text_input("#000000", &hex_value)
                    .on_input({
                        let current_value = value.clone();
                        let name = name_owned.clone();
                        let mode = mode_owned.clone();
                        move |new_hex| {
                            Message::Design(DesignMessage::VariableValueChanged(
                                name.clone(),
                                mode.clone(),
                                update_color_hex_value(&current_value, &new_hex),
                            ))
                        }
                    })
                    .padding([8, 10])
                    .size(12)
                    .style(variable_value_input_style)
                    .width(Length::FillPortion(2)),
                container(Space::new().width(Length::Fixed(1.0)).height(Length::Fill)).style(
                    move |theme: &Theme| {
                        let palette = variables_palette(theme);
                        container::Style {
                            background: Some(Background::Color(palette.row_divider)),
                            ..Default::default()
                        }
                    }
                ),
                text_input("100", &alpha_value)
                    .on_input({
                        let current_value = value.clone();
                        let name = name_owned.clone();
                        let mode = mode_owned.clone();
                        move |new_alpha| {
                            Message::Design(DesignMessage::VariableValueChanged(
                                name.clone(),
                                mode.clone(),
                                update_color_alpha_value(&current_value, &new_alpha),
                            ))
                        }
                    })
                    .padding([8, 10])
                    .size(12)
                    .style(variable_value_input_style)
                    .width(Length::Fixed(64.0)),
                text("%").size(11).style(move |theme: &Theme| {
                    let palette = variables_palette(theme);
                    iced::widget::text::Style { color: Some(palette.subtitle) }
                })
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([4, 8])
        .style(move |theme: &Theme| {
            let palette = variables_palette(theme);
            container::Style {
                background: Some(Background::Color(palette.cell_bg)),
                border: Border { radius: 10.0.into(), width: 1.0, color: palette.cell_border },
                ..Default::default()
            }
        })
        .into()
    } else {
        text_input(if def.kind.eq_ignore_ascii_case("number") { "0" } else { "输入值" }, &value)
            .on_input(move |new_value| {
                Message::Design(DesignMessage::VariableValueChanged(
                    name_owned.clone(),
                    mode_owned.clone(),
                    new_value,
                ))
            })
            .padding([8, 10])
            .size(12)
            .style(variable_value_input_style)
            .width(Length::Fixed(VARIANT_COL_WIDTH))
            .into()
    };

    if def.kind.eq_ignore_ascii_case("color") {
        container(cell).width(Length::Fixed(VARIANT_COL_WIDTH)).into()
    } else {
        cell
    }
}

fn render_variable_menu_button<'a>(
    state: &'a DesignState,
    name: &'a str,
    theme_modes: &[String],
    label_font: Font,
) -> Element<'a, Message> {
    let menu_open = state.active_variable_menu.as_deref() == Some(name);
    let move_targets_open = state.variable_move_target_picker.as_deref() == Some(name);
    let confirm_delete = state.confirm_delete_variable.as_deref() == Some(name);

    let trigger = button(
        container(svg(assets::get_icon(Icon::DotsThreeVertical)).width(12).height(12).style(
            move |theme: &Theme, _| {
                let palette = variables_palette(theme);
                svg::Style { color: Some(palette.menu_text.scale_alpha(0.9)) }
            },
        ))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .width(Length::Fixed(ACTION_COL_WIDTH))
    .height(Length::Fixed(32.0))
    .padding(0)
    .style(move |theme: &Theme, status| {
        let palette = variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered || menu_open || move_targets_open || confirm_delete {
                    palette.menu_hover_bg
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            text_color: palette.menu_text,
            border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::ToggleVariableMenu(name.to_string())));

    let host = container(trigger).width(Length::Fixed(ACTION_COL_WIDTH));
    if menu_open || move_targets_open || confirm_delete {
        let overlay = if confirm_delete {
            render_delete_confirm(
                "删除变量？",
                format!("{name} 会从变量表中移除。"),
                Message::Design(DesignMessage::CancelDeleteVariable),
                Message::Design(DesignMessage::ConfirmDeleteVariable),
                label_font,
            )
        } else if move_targets_open {
            render_move_targets_menu(name.to_string(), theme_modes, label_font)
        } else {
            render_variable_menu(name.to_string(), label_font)
        };
        PointBelowOverlay::new(host, overlay)
            .show(true)
            .gap(28.0)
            .on_close(Message::Design(DesignMessage::CloseVariableMenu))
            .into()
    } else {
        host.into()
    }
}

fn render_value_preview(kind: &str, value: &str) -> Element<'static, Message> {
    match kind.to_ascii_lowercase().as_str() {
        "color" => {
            let swatch = parse_hex_color(value).unwrap_or(Color::TRANSPARENT);
            container(Space::new().width(Length::Fixed(12.0)).height(Length::Fixed(12.0)))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(swatch)),
                    border: Border {
                        radius: 3.0.into(),
                        width: 1.0,
                        color: swatch_border_color(swatch),
                    },
                    ..Default::default()
                })
                .into()
        }
        "number" => {
            container(text("#").size(12)).width(Length::Fixed(12.0)).center_x(Length::Fill).into()
        }
        _ => container(text("T").size(11)).width(Length::Fixed(12.0)).center_x(Length::Fill).into(),
    }
}
#[cfg(test)]
#[path = "table_tests.rs"]
mod table_tests;
