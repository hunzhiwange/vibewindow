//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, container};
use iced::{Element, Length};
use std::borrow::Cow;

use crate::app::{App, Message};

use crate::app::views::design::canvas::DesignCanvas;

/// 渲染对应界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render_canvas(app: &App) -> Element<'_, Message> {
    if let Some(state) = app.active_design_state() {
        let color_picking = app.active_color_picker.as_ref().map(|p| p.picking).unwrap_or(false)
            || app.active_fill_picker.as_ref().map(|p| p.picking).unwrap_or(false);
        let filtered_doc = state.doc.filtered_for_group(state.active_group_id);

        let canvas_widget = DesignCanvas {
            doc: Cow::Owned(filtered_doc),
            cache: &state.canvas_cache,
            pan: state.pan,
            zoom: state.zoom,
            selected_id: state.selected_element_id.as_deref(),
            selected_ids: &state.selected_element_ids,
            selected_fill_index: state.selected_fill_index,
            editing_id: state.editing_id.as_deref(),
            active_tool: state.active_tool,
            brush_color_hex: &state.brush_color_hex,
            brush_width_px: state.brush_width_px,
            toolbar_icon_family: &state.toolbar_icon_family,
            toolbar_icon_name: &state.toolbar_icon_name,
            mouse_wheel_zoom_enabled: app.mouse_wheel_zoom_enabled,
            show_slot_content: app.show_slot_content,
            show_slot_overflow: app.show_slot_overflow,
            color_picking,
            hover_disabled: state.tailwind_inspector_hovered,
        };

        container(iced::widget::canvas(canvas_widget).width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style::default())
            .into()
    } else {
        Space::new().into()
    }
}
#[cfg(test)]
#[path = "canvas_tests.rs"]
mod canvas_tests;
