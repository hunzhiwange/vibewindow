//! 设计变量面板模块，负责变量集合、主题模式和值编辑界面的拆分实现。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::font::{Font, Weight};
use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::components::overlays::AboveOverlay;
use crate::app::message::DesignMessage;
use crate::app::message::design::VariableKindPreset;

use super::styles::{
    VARIABLE_FOOTER_HEIGHT, VARIABLE_MENU_BUTTON_HEIGHT, menu_button_style, menu_surface_style,
    variables_palette,
};
use super::table::divider;

/// 渲染对应界面。
///
/// # 参数
/// - `menu_open`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_variable_footer<'a>(
    menu_open: bool,
    label_font: Font,
) -> Element<'a, Message> {
    column![
        divider(),
        container(
            row![
                render_add_variable_button(menu_open, label_font),
                Space::new().width(Length::Fill)
            ]
            .align_y(Alignment::Center),
        )
        .height(Length::Fixed(VARIABLE_FOOTER_HEIGHT))
        .center_y(Length::Fill)
    ]
    .spacing(0)
    .width(Length::Fill)
    .into()
}

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
/// - `destructive`: 当前视图构建所需的状态、配置或消息。
/// - `message`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn action_button<'a>(
    label: &'static str,
    label_font: Font,
    destructive: bool,
    message: Message,
) -> Element<'a, Message> {
    button(text(label).size(12).font(label_font))
        .padding([7, 10])
        .style(move |theme: &Theme, status| {
            let palette = variables_palette(theme);
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if destructive {
                        if hovered {
                            palette.danger_bg.scale_alpha(0.92)
                        } else {
                            palette.danger_bg
                        }
                    } else if hovered {
                        palette.menu_hover_bg
                    } else {
                        palette.menu_bg
                    }
                    .into(),
                ),
                text_color: if destructive { Color::WHITE } else { palette.menu_text },
                border: Border {
                    radius: 8.0.into(),
                    width: if destructive { 0.0 } else { 1.0 },
                    color: if destructive { Color::TRANSPARENT } else { palette.menu_border },
                },
                ..Default::default()
            }
        })
        .on_press(message)
        .into()
}

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
/// - `message`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn ghost_button<'a>(
    label: &'static str,
    label_font: Font,
    message: Message,
) -> Element<'a, Message> {
    button(text(label).size(12).font(label_font))
        .padding([7, 10])
        .style(move |_theme: &Theme, _status| button::Style {
            background: Some(Color::TRANSPARENT.into()),
            text_color: Color::from_rgba8(113, 113, 122, 1.0),
            border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        })
        .on_press(message)
        .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `title`: 当前视图构建所需的状态、配置或消息。
/// - `description`: 当前视图构建所需的状态、配置或消息。
/// - `cancel`: 当前视图构建所需的状态、配置或消息。
/// - `confirm`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_delete_confirm<'a>(
    title: &'static str,
    description: String,
    cancel: Message,
    confirm: Message,
    label_font: Font,
) -> Element<'a, Message> {
    let title_font = Font { weight: Weight::Bold, ..Default::default() };
    container(
        column![
            text(title).size(13).font(title_font).style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                iced::widget::text::Style { color: Some(palette.title) }
            }),
            text(description).size(11).style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                iced::widget::text::Style { color: Some(palette.subtitle) }
            }),
            row![
                action_button("取消", label_font, false, cancel),
                action_button("确定删除", label_font, true, confirm)
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        ]
        .spacing(10),
    )
    .padding(12)
    .width(Length::Fixed(240.0))
    .style(menu_surface_style)
    .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `icon`: 当前视图构建所需的状态、配置或消息。
/// - `on_press`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
/// - `destructive`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_menu_item<'a>(
    label: String,
    icon: Icon,
    on_press: Message,
    label_font: Font,
    destructive: bool,
) -> Element<'a, Message> {
    button(
        row![
            svg(assets::get_icon(icon)).width(12).height(12).style(move |theme: &Theme, _| {
                let palette = variables_palette(theme);
                svg::Style {
                    color: Some(if destructive { palette.danger_text } else { palette.menu_text }),
                }
            }),
            text(label).size(12).font(label_font).style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                iced::widget::text::Style {
                    color: Some(if destructive { palette.danger_text } else { palette.menu_text }),
                }
            })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([7, 8])
    .style(menu_button_style(destructive))
    .on_press(on_press)
    .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `theme_name`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_collection_menu<'a>(
    theme_name: String,
    label_font: Font,
) -> Element<'a, Message> {
    container(
        column![
            render_menu_item(
                "Rename".to_string(),
                Icon::Pencil,
                Message::Design(DesignMessage::RenameVariableCollectionRequested(
                    theme_name.clone()
                )),
                label_font,
                false,
            ),
            render_menu_item(
                "Copy".to_string(),
                Icon::Copy,
                Message::Design(DesignMessage::DuplicateVariableCollection(theme_name.clone())),
                label_font,
                false,
            ),
            divider(),
            render_menu_item(
                "Delete".to_string(),
                Icon::Trash,
                Message::Design(DesignMessage::RequestDeleteVariableCollection(theme_name)),
                label_font,
                true,
            )
        ]
        .spacing(2),
    )
    .padding(6)
    .width(Length::Fixed(144.0))
    .style(menu_surface_style)
    .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `theme_name`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_variant_menu<'a>(
    theme_name: String,
    label_font: Font,
) -> Element<'a, Message> {
    container(
        column![
            render_menu_item(
                "Rename".to_string(),
                Icon::Pencil,
                Message::Design(DesignMessage::RenameVariableThemeRequested(theme_name.clone())),
                label_font,
                false,
            ),
            render_menu_item(
                "Copy".to_string(),
                Icon::Copy,
                Message::Design(DesignMessage::DuplicateVariableTheme(theme_name.clone())),
                label_font,
                false,
            ),
            divider(),
            render_menu_item(
                "Delete".to_string(),
                Icon::Trash,
                Message::Design(DesignMessage::RequestDeleteVariableTheme(theme_name)),
                label_font,
                true,
            )
        ]
        .spacing(2),
    )
    .padding(6)
    .width(Length::Fixed(144.0))
    .style(menu_surface_style)
    .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `name`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_variable_menu<'a>(name: String, label_font: Font) -> Element<'a, Message> {
    container(
        column![
            render_menu_item(
                "Rename".to_string(),
                Icon::Pencil,
                Message::Design(DesignMessage::RenameVariableRequested(name.clone())),
                label_font,
                false,
            ),
            render_menu_item(
                "Duplicate".to_string(),
                Icon::Copy,
                Message::Design(DesignMessage::DuplicateVariable(name.clone())),
                label_font,
                false,
            ),
            render_menu_item(
                "Move to...".to_string(),
                Icon::ChevronRight,
                Message::Design(DesignMessage::ToggleVariableMoveTargets(name.clone())),
                label_font,
                false,
            ),
            divider(),
            render_menu_item(
                "Delete".to_string(),
                Icon::Trash,
                Message::Design(DesignMessage::RequestDeleteVariable(name)),
                label_font,
                true,
            )
        ]
        .spacing(2),
    )
    .padding(6)
    .width(Length::Fixed(170.0))
    .style(menu_surface_style)
    .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `name`: 当前视图构建所需的状态、配置或消息。
/// - `theme_modes`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_move_targets_menu<'a>(
    name: String,
    theme_modes: &[String],
    label_font: Font,
) -> Element<'a, Message> {
    let mut items = column![
        render_menu_item(
            "Back".to_string(),
            Icon::ChevronLeft,
            Message::Design(DesignMessage::ToggleVariableMoveTargets(name.clone())),
            label_font,
            false,
        ),
        divider()
    ]
    .spacing(2);

    for mode in theme_modes {
        items = items.push(render_menu_item(
            mode.clone(),
            Icon::Columns,
            Message::Design(DesignMessage::MoveVariableTo(name.clone(), Some(mode.clone()))),
            label_font,
            false,
        ));
    }

    container(items).padding(6).width(Length::Fixed(170.0)).style(menu_surface_style).into()
}

fn render_add_variable_button<'a>(menu_open: bool, label_font: Font) -> Element<'a, Message> {
    let button_content = row![
        svg(assets::get_icon(Icon::Plus)).width(12).height(12).style(move |theme: &Theme, _| {
            let palette = variables_palette(theme);
            svg::Style { color: Some(palette.menu_text) }
        }),
        text("新增变量").size(12).font(label_font).style(move |theme: &Theme| {
            let palette = variables_palette(theme);
            iced::widget::text::Style { color: Some(palette.menu_text) }
        }),
        svg(assets::get_icon(Icon::ChevronDown)).width(10).height(10).style(
            move |theme: &Theme, _| {
                let palette = variables_palette(theme);
                svg::Style { color: Some(palette.menu_text.scale_alpha(0.86)) }
            }
        )
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let trigger = button(button_content)
        .height(Length::Fixed(VARIABLE_MENU_BUTTON_HEIGHT))
        .padding([0, 12])
        .style(move |theme: &Theme, status| {
            let palette = variables_palette(theme);
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if menu_open || hovered { palette.menu_hover_bg } else { palette.menu_bg }
                        .into(),
                ),
                text_color: palette.menu_text,
                border: Border { radius: 10.0.into(), width: 1.0, color: palette.menu_border },
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::ToggleAddVariableMenu));

    if menu_open {
        AboveOverlay::new(trigger, render_add_variable_menu(label_font))
            .show(true)
            .gap(8.0)
            .on_close(Message::Design(DesignMessage::CloseAddVariableMenu))
            .into()
    } else {
        trigger.into()
    }
}

fn render_add_variable_menu<'a>(label_font: Font) -> Element<'a, Message> {
    container(
        column![
            render_variable_kind_menu_item(VariableKindPreset::Color, label_font),
            render_variable_kind_menu_item(VariableKindPreset::Number, label_font),
            render_variable_kind_menu_item(VariableKindPreset::String, label_font)
        ]
        .spacing(2),
    )
    .padding(6)
    .width(Length::Fixed(168.0))
    .style(menu_surface_style)
    .into()
}

fn render_variable_kind_menu_item<'a>(
    kind: VariableKindPreset,
    label_font: Font,
) -> Element<'a, Message> {
    button(
        row![
            super::table::render_kind_glyph(kind.as_kind()),
            text(kind.label()).size(12).font(label_font).style(move |theme: &Theme| {
                let palette = variables_palette(theme);
                iced::widget::text::Style { color: Some(palette.menu_text) }
            })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([8, 10])
    .style(menu_button_style(false))
    .on_press(Message::Design(DesignMessage::CreateVariable(kind)))
    .into()
}
#[cfg(test)]
#[path = "menus_tests.rs"]
mod menus_tests;
