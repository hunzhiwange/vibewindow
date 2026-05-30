//! 设计变量面板模块，负责变量集合、主题模式和值编辑界面的拆分实现。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::font::Font;
use iced::widget::{button, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::components::overlays::PointBelowOverlay;
use crate::app::message::DesignMessage;
use crate::app::views::design::state::DesignState;

use super::menus::{render_collection_menu, render_delete_confirm};
use super::styles::{
    THEME_ADD_BUTTON_SIZE, THEME_TAB_BUTTON_HEIGHT, THEME_TAB_MENU_BUTTON_SIZE, THEME_TAB_MENU_GAP,
    variables_palette,
};

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `state`: 当前视图构建所需的状态、配置或消息。
/// - `names`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn current_variable_collection_name(state: &DesignState, names: &[String]) -> String {
    state
        .current_variable_collection
        .as_ref()
        .cloned()
        .or_else(|| names.first().cloned())
        .unwrap_or_else(|| "Theme".to_string())
}

/// 渲染对应界面。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `active`: 当前视图构建所需的状态、配置或消息。
/// - `active_menu`: 当前视图构建所需的状态、配置或消息。
/// - `delete_target`: 当前视图构建所需的状态、配置或消息。
/// - `label_font`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_collection_tab<'a>(
    label: String,
    active: bool,
    active_menu: Option<&String>,
    delete_target: Option<&String>,
    label_font: Font,
) -> Element<'a, Message> {
    let menu_open = active_menu.is_some_and(|value| value.eq_ignore_ascii_case(&label));
    let confirm_delete = delete_target.is_some_and(|value| value.eq_ignore_ascii_case(&label));
    let highlighted = active || menu_open || confirm_delete;

    let select_button =
        button(text(label.to_string()).size(12).font(label_font).style(move |theme: &Theme| {
            let palette = variables_palette(theme);
            iced::widget::text::Style {
                color: Some(if highlighted { palette.title } else { palette.menu_text }),
            }
        }))
        .padding([7, 12])
        .height(Length::Fixed(THEME_TAB_BUTTON_HEIGHT))
        .style(move |theme: &Theme, status| {
            let palette = variables_palette(theme);
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if highlighted {
                        palette.name_bg
                    } else if hovered {
                        palette.menu_hover_bg
                    } else {
                        Color::TRANSPARENT
                    }
                    .into(),
                ),
                text_color: if highlighted { palette.title } else { palette.menu_text },
                border: Border {
                    radius: 999.0.into(),
                    width: if highlighted || hovered { 1.0 } else { 0.0 },
                    color: if highlighted { palette.name_border } else { palette.menu_border },
                },
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::SelectVariableCollection(label.clone())));

    let menu_button = button(svg(assets::get_icon(Icon::ChevronDown)).width(8).height(8).style(
        move |theme: &Theme, status| {
            let palette = variables_palette(theme);
            let visible = menu_open || confirm_delete || matches!(status, svg::Status::Hovered);
            svg::Style {
                color: Some(if visible {
                    if highlighted { palette.title } else { palette.menu_text }.scale_alpha(0.62)
                } else {
                    Color::TRANSPARENT
                }),
            }
        },
    ))
    .padding(0)
    .width(Length::Fixed(THEME_TAB_MENU_BUTTON_SIZE))
    .height(Length::Fixed(THEME_TAB_MENU_BUTTON_SIZE))
    .style(move |theme: &Theme, status| {
        let palette = variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered || menu_open || confirm_delete {
                    palette.menu_hover_bg.scale_alpha(0.92)
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            text_color: if highlighted { palette.title } else { palette.menu_text },
            border: Border { radius: 999.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::ToggleVariableCollectionMenu(label.clone())));

    let host = row![select_button, menu_button].spacing(4).align_y(Alignment::Center);

    if menu_open || confirm_delete {
        let overlay = if confirm_delete {
            render_delete_confirm(
                "删除主题？",
                format!("{label} 主题下的变量会一并删除。"),
                Message::Design(DesignMessage::CancelDeleteVariableCollection),
                Message::Design(DesignMessage::ConfirmDeleteVariableCollection),
                label_font,
            )
        } else {
            render_collection_menu(label, label_font)
        };
        PointBelowOverlay::new(host, overlay)
            .show(true)
            .gap(THEME_TAB_MENU_GAP)
            .on_close(Message::Design(DesignMessage::CloseVariableCollectionMenu))
            .into()
    } else {
        host.into()
    }
}

/// 渲染对应界面。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn render_add_collection_button<'a>() -> Element<'a, Message> {
    button(svg(assets::get_icon(Icon::Plus)).width(10).height(10).style(move |theme: &Theme, _| {
        let palette = variables_palette(theme);
        svg::Style { color: Some(palette.menu_text) }
    }))
    .padding(0)
    .width(Length::Fixed(THEME_ADD_BUTTON_SIZE))
    .height(Length::Fixed(THEME_ADD_BUTTON_SIZE))
    .style(move |theme: &Theme, status| {
        let palette = variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered { palette.menu_hover_bg.scale_alpha(0.94) } else { Color::TRANSPARENT }
                    .into(),
            ),
            text_color: palette.menu_text,
            border: Border { radius: 999.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::AddVariableCollection))
    .into()
}
#[cfg(test)]
#[path = "collections_tests.rs"]
mod collections_tests;
