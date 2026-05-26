//! 设计变量面板模块，负责变量集合、主题模式和值编辑界面的拆分实现。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::font::{Font, Weight};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, scrollable, svg, text, text_input};
use iced::{Alignment, Element, Length, Theme};

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::state::DesignState;

mod collections;
mod menus;
mod styles;
mod table;
mod utils;

use collections::{current_variable_collection_name, render_add_collection_button, render_collection_tab};
use menus::{action_button, ghost_button, render_variable_footer};
use styles::{PANEL_HEIGHT, PANEL_WIDTH, THEME_TABS_SCROLL_HEIGHT, backdrop_style, panel_surface_style, variable_text_input_style};
use table::render_variable_table;

/// 渲染对应界面。
///
/// # 参数
/// - `show`: 当前视图构建所需的状态、配置或消息。
/// - `state`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render_variables_panel<'a>(show: bool, state: &'a DesignState) -> Element<'a, Message> {
    if !show {
        return Space::new().into();
    }

    let label_font = Font { weight: Weight::Medium, ..Default::default() };
    let collection_names = state.doc.variable_collection_names();
    let theme_modes = state.doc.variable_theme_modes();
    let current_collection = current_variable_collection_name(state, &collection_names);

    let close_btn = button(
        container(svg(assets::get_icon(Icon::X)).width(14).height(14).style(
            move |theme: &Theme, _status| {
                let palette = styles::variables_palette(theme);
                svg::Style { color: Some(palette.subtitle) }
            },
        ))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .width(Length::Fixed(28.0))
    .height(Length::Fixed(28.0))
    .padding(0)
    .style(move |theme: &Theme, status| {
        let palette = styles::variables_palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                if hovered { palette.menu_hover_bg } else { iced::Color::TRANSPARENT }.into(),
            ),
            text_color: palette.subtitle,
            border: iced::Border {
                radius: 8.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::ToggleVariables));

    let mut theme_tabs = row![].spacing(6).align_y(Alignment::Center);
    for name in &collection_names {
        theme_tabs = theme_tabs.push(render_collection_tab(
            name.clone(),
            name.eq_ignore_ascii_case(&current_collection),
            state.active_variable_collection_menu.as_ref(),
            state.confirm_delete_variable_collection.as_ref(),
            label_font,
        ));
    }

    theme_tabs = theme_tabs.push(render_add_collection_button());

    let theme_row = row![
        scrollable(theme_tabs)
            .direction(Direction::Horizontal(Scrollbar::new().width(4).scroller_width(4)))
            .width(Length::Fill)
            .height(Length::Fixed(THEME_TABS_SCROLL_HEIGHT)),
        close_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let mut content = column![theme_row].spacing(14).width(Length::Fill).height(Length::Fill);

    if let Some(collection_name) = state.renaming_variable_collection.as_deref() {
        content = content.push(render_collection_rename_row(collection_name, state, label_font));
    }

    if let Some(theme_name) = state.renaming_variable_theme.as_deref() {
        content = content.push(render_variant_rename_row(theme_name, state, label_font));
    }

    if let Some(variable_name) = state.renaming_variable.as_deref() {
        content = content.push(render_variable_rename_row(variable_name, state, label_font));
    }

    content = content.push(
        container(render_variable_table(state, &current_collection, &theme_modes, label_font))
            .width(Length::Fill)
            .height(Length::Fill),
    );
    content = content.push(render_variable_footer(state.show_add_variable_menu, label_font));

    let panel = container(content)
        .width(Length::Fixed(PANEL_WIDTH))
        .height(Length::Fixed(PANEL_HEIGHT))
        .padding(18)
        .style(panel_surface_style);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(backdrop_style)
        .into()
}

fn render_collection_rename_row<'a>(
    current: &'a str,
    state: &'a DesignState,
    label_font: Font,
) -> Element<'a, Message> {
    container(
        row![
            text(format!("重命名主题: {current}")).size(12).font(label_font).style(
                move |theme: &Theme| {
                    let palette = styles::variables_palette(theme);
                    iced::widget::text::Style { color: Some(palette.subtitle) }
                }
            ),
            text_input("输入主题名称", &state.variable_collection_rename_value)
                .on_input(|value| Message::Design(DesignMessage::VariableCollectionRenameChanged(
                    value
                )))
                .on_submit(Message::Design(DesignMessage::SubmitVariableCollectionRename))
                .padding([8, 10])
                .size(12)
                .style(variable_text_input_style)
                .width(Length::Fixed(160.0)),
            action_button(
                "确定",
                label_font,
                false,
                Message::Design(DesignMessage::SubmitVariableCollectionRename)
            ),
            ghost_button(
                "取消",
                label_font,
                Message::Design(DesignMessage::CancelVariableCollectionRename)
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([6, 0])
    .into()
}

fn render_variant_rename_row<'a>(
    current: &'a str,
    state: &'a DesignState,
    label_font: Font,
) -> Element<'a, Message> {
    container(
        row![
            text(format!("重命名 variant: {current}")).size(12).font(label_font).style(
                move |theme: &Theme| {
                    let palette = styles::variables_palette(theme);
                    iced::widget::text::Style { color: Some(palette.subtitle) }
                }
            ),
            text_input("输入 variant 名称", &state.variable_theme_rename_value)
                .on_input(|value| Message::Design(DesignMessage::VariableThemeRenameChanged(value)))
                .on_submit(Message::Design(DesignMessage::SubmitVariableThemeRename))
                .padding([8, 10])
                .size(12)
                .style(variable_text_input_style)
                .width(Length::Fixed(160.0)),
            action_button(
                "确定",
                label_font,
                false,
                Message::Design(DesignMessage::SubmitVariableThemeRename)
            ),
            ghost_button(
                "取消",
                label_font,
                Message::Design(DesignMessage::CancelVariableThemeRename)
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([6, 0])
    .into()
}

fn render_variable_rename_row<'a>(
    current: &'a str,
    state: &'a DesignState,
    label_font: Font,
) -> Element<'a, Message> {
    container(
        row![
            text(format!("重命名变量: {current}")).size(12).font(label_font).style(
                move |theme: &Theme| {
                    let palette = styles::variables_palette(theme);
                    iced::widget::text::Style { color: Some(palette.subtitle) }
                }
            ),
            text_input("输入变量名称", &state.variable_rename_value)
                .on_input(|value| Message::Design(DesignMessage::VariableRenameChanged(value)))
                .on_submit(Message::Design(DesignMessage::SubmitVariableRename))
                .padding([8, 10])
                .size(12)
                .style(variable_text_input_style)
                .width(Length::Fixed(160.0)),
            action_button(
                "确定",
                label_font,
                false,
                Message::Design(DesignMessage::SubmitVariableRename)
            ),
            ghost_button("取消", label_font, Message::Design(DesignMessage::CancelVariableRename)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([6, 0])
    .into()
}
#[cfg(test)]
mod tests;
