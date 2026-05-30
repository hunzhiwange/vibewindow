//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, button, column, container, mouse_area, text};
use iced::{Background, Border, Color, Element, Length};

use super::shared::design_overlay_surface_shadow;
use crate::app::message::DesignMessage;
use crate::app::message::design::CanvasContextMenuAction;
use crate::app::views::design::canvas::get_element_screen_bounds;
use crate::app::views::design::state::DesignState;
use crate::app::views::design::toolbar;
use crate::app::{App, Message};

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
pub fn context_toolbar_layers<'a>(
    app: &'a App,
    state: &'a DesignState,
) -> Vec<Element<'a, Message>> {
    let mut layers = vec![];

    if let Some(tb) = toolbar::render_context_toolbar(state)
        && let Some(id) = &state.selected_element_id
        && let Some(rect) = get_element_screen_bounds(&state.doc, id, state.pan, state.zoom)
    {
        let selected_element = state.doc.find_element(id);
        let is_text_selected = selected_element.is_some_and(|el| {
            matches!(el.kind.as_str(), "text" | "Typography")
                || el.kind.eq_ignore_ascii_case("sticky_note")
        });
        let selected_font =
            selected_element.and_then(|el| el.font_family.as_deref()).unwrap_or("Inter");
        let (panel_w, panel_h): (f32, f32) = match state.context_popover {
            Some(crate::app::views::design::state::ContextPopoverType::Shape) => (256.0, 258.0),
            Some(crate::app::views::design::state::ContextPopoverType::Fill) => (332.0, 170.0),
            Some(crate::app::views::design::state::ContextPopoverType::Border) => (332.0, 170.0),
            Some(crate::app::views::design::state::ContextPopoverType::TextColor) => {
                (toolbar::text_context_panel_width(selected_font, true), 140.0)
            }
            Some(crate::app::views::design::state::ContextPopoverType::ToolbarBrush) => {
                (164.0, 44.0)
            }
            Some(crate::app::views::design::state::ContextPopoverType::ToolbarShape) => {
                (164.0, 44.0)
            }
            Some(crate::app::views::design::state::ContextPopoverType::ToolbarIcon) => {
                (164.0, 44.0)
            }
            None if is_text_selected => {
                (toolbar::text_context_panel_width(selected_font, false), 44.0)
            }
            None => (164.0, 44.0),
        };
        let canvas_left_offset: f32 =
            if app.show_layer_panel { app.layer_panel_width + 8.0 } else { 0.0 };
        let max_x = (app.window_size.0 - panel_w).max(0.0);
        let target_left = canvas_left_offset + rect.x + (rect.width * 0.5) - (panel_w * 0.5);
        let left = target_left.clamp(0.0, max_x);
        let top = (rect.y - panel_h - 12.0).max(8.0);
        let content = container(tb)
            .padding(iced::Padding { top, left, ..Default::default() })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top);
        layers.push(content.into());
    }

    layers
}

/// 构建菜单界面。
///
/// # 参数
/// - `state`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn canvas_context_menu_layers<'a>(state: &'a DesignState) -> Vec<Element<'a, Message>> {
    let Some(anchor) = state.canvas_context_menu_anchor else {
        return vec![];
    };

    let has_selection = !state.selected_element_ids.is_empty();
    let menu_button = |label: &'static str, action: CanvasContextMenuAction| {
        button(container(text(label).size(13)).width(Length::Fill).padding([4, 10]))
            .width(Length::Fill)
            .on_press(Message::Design(DesignMessage::CanvasContextMenuAction(action)))
            .style(|theme: &iced::Theme, status| {
                let p = theme.extended_palette();
                let bg = match status {
                    button::Status::Hovered => p.background.weak.color.scale_alpha(0.72),
                    button::Status::Pressed => p.background.strong.color.scale_alpha(0.78),
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: theme.palette().text,
                    border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
                    ..Default::default()
                }
            })
    };

    let mut items = column![menu_button("粘贴", CanvasContextMenuAction::Paste),].spacing(3);

    if has_selection {
        items = column![
            menu_button("剪切", CanvasContextMenuAction::Cut),
            menu_button("复制", CanvasContextMenuAction::Copy),
            menu_button("粘贴", CanvasContextMenuAction::Paste),
            menu_button("上移一层", CanvasContextMenuAction::MoveUp),
            menu_button("下移一层", CanvasContextMenuAction::MoveDown),
            menu_button("删除", CanvasContextMenuAction::Delete),
        ]
        .spacing(3);
    }

    let menu_panel =
        container(items).padding(6).width(Length::Fixed(158.0)).style(|theme: &iced::Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(p.background.base.color.scale_alpha(0.98))),
                border: Border {
                    radius: 10.0.into(),
                    width: 1.0,
                    color: p.background.strong.color.scale_alpha(0.72),
                },
                shadow: iced::Shadow {
                    color: design_overlay_surface_shadow(theme, 0.52, 0.28),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            }
        });

    let bg_layer: Element<'a, Message> =
        mouse_area(container(Space::new()).width(Length::Fill).height(Length::Fill))
            .on_press(Message::Design(DesignMessage::CanvasContextMenuClose))
            .into();

    let menu_layer: Element<'a, Message> = container(menu_panel)
        .padding(iced::Padding { top: anchor.y, left: anchor.x, ..Default::default() })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .align_y(iced::alignment::Vertical::Top)
        .into();

    vec![bg_layer, menu_layer]
}
#[cfg(test)]
#[path = "overlay_context_tests.rs"]
mod overlay_context_tests;
