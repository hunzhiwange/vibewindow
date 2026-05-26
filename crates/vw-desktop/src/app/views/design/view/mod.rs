//! # 设计器视图模块
//!
//! 本模块负责渲染设计器主界面的整体布局，并把具体的大块 UI 逻辑拆到独立子模块。
//!
//! ## 主要职责
//!
//! - 组织设计器主界面的三栏布局与叠层关系
//! - 协调画布、规划面板、覆盖层与 Tailwind 检查器
//! - 在无活动设计文件时渲染空状态入口
//!
//! ## 模块结构
//!
//! - [`canvas`] - 画布渲染子模块，负责绘制设计元素
//! - [`helpers`] - 共享样式与颜色辅助函数
//! - [`overlay`] - 覆盖层子模块，负责各种浮动面板和编辑器
//! - [`planner`] - 设计规划面板与 Figma 进度覆盖层
//! - [`selectors`] - 设计生成相关的弹出选择器
//! - [`tailwind_inspector`] - Tailwind 结构检查器

use iced::widget::{Space, button, column, container, row, stack, text};
use iced::{Background, Color, Element, Length, Theme};

use crate::app::message::{DesignMessage, ViewMessage};
use crate::app::views::design::state::ContextPopoverType;
use crate::app::{App, Message};

use super::image_import::render_image_import_dialog;
use super::layers::render_layers;
use super::properties::render_properties;
use super::settings::{render_settings_panel, render_shortcuts_panel, render_zoom_controls};
use super::sticky_note_create::render_sticky_note_create_dialog;
use super::toolbar::render_toolbar;
use super::variables::render_variables_panel;

mod canvas;
mod helpers;
mod overlay;
mod planner;
mod selectors;
mod tailwind_inspector;

use helpers::design_is_dark;
use planner::{render_design_planner_panel_overlay, render_figma_progress_overlay};
use tailwind_inspector::render_tailwind_inspector_panel;

fn render_empty_state_view<'a>() -> Element<'a, Message> {
    let new_btn_style = |theme: &Theme, status: button::Status| {
        let is_dark = design_is_dark(theme);
        let bg = match status {
            button::Status::Hovered => {
                if is_dark {
                    Color::from_rgb8(96, 165, 250)
                } else {
                    Color::from_rgb8(59, 130, 246)
                }
            }
            button::Status::Pressed => {
                if is_dark {
                    Color::from_rgb8(59, 130, 246)
                } else {
                    Color::from_rgb8(37, 99, 235)
                }
            }
            _ => {
                if is_dark {
                    Color::from_rgb8(72, 149, 239)
                } else {
                    Color::from_rgb8(37, 99, 235)
                }
            }
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: iced::Border { color: bg, width: 0.0, radius: 10.0.into() },
            ..Default::default()
        }
    };

    let open_btn_style = |theme: &Theme, status: button::Status| {
        let ext = theme.extended_palette();
        let bg = match status {
            button::Status::Hovered => ext.background.strong.color,
            button::Status::Pressed => ext.background.strong.color,
            _ => ext.background.weak.color,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: theme.palette().text,
            border: iced::Border {
                color: ext.background.strong.color,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    };

    let content = column![
        text("设计器").size(20).style(|theme: &Theme| {
            iced::widget::text::Style { color: Some(theme.palette().text) }
        }),
        text("新建一个空白设计，或打开已有文件").size(14).style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            iced::widget::text::Style {
                color: Some(if is_dark {
                    palette.text.scale_alpha(0.78)
                } else {
                    theme.extended_palette().background.base.text.scale_alpha(0.82)
                }),
            }
        }),
        button("新建空白")
            .on_press(Message::Design(DesignMessage::New))
            .style(new_btn_style)
            .padding([6, 16]),
        button("打开文件")
            .on_press(Message::Design(DesignMessage::Open))
            .style(open_btn_style)
            .padding([6, 16]),
    ]
    .spacing(12)
    .align_x(iced::alignment::Horizontal::Center);

    let panel = container(content).padding(24).style(|theme: &Theme| {
        let ext = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(ext.background.base.color)),
            border: iced::Border {
                color: ext.background.strong.color,
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.10),
                offset: iced::Vector::new(0.0, 10.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        }
    });

    container(panel).center(Length::Fill).into()
}

/// 渲染设计器的主视图
///
/// 顶层仅负责按状态拼装空状态界面或编辑态布局，细节子面板由各独立模块完成。
pub fn view(app: &App) -> Element<'_, Message> {
    let Some(state) = app.active_design_state() else {
        return render_empty_state_view();
    };

    let layers = render_layers(app);
    let design_planner_overlay = render_design_planner_panel_overlay(app, state);
    let canvas = canvas::render_canvas(app);
    let canvas_overlay = overlay::inline_text_editor_overlay(state);
    let properties = render_properties(app);
    let toolbar = render_toolbar(
        state.active_tool,
        app.show_layer_panel,
        app.show_properties_panel,
        app.show_design_variables,
        state.context_popover == Some(ContextPopoverType::ToolbarBrush),
        state.context_popover == Some(ContextPopoverType::ToolbarShape),
        state.context_popover == Some(ContextPopoverType::ToolbarIcon),
        &state.brush_color_hex,
        state.brush_width_px,
        &state.icon_filter_query,
        &state.toolbar_icon_family,
        &state.toolbar_icon_name,
        &state.toolbar_icon_family_tab,
        app.layer_panel_width,
    );
    let zoom = render_zoom_controls(state.zoom, state.show_zoom_menu);

    let settings_layer = if app.show_design_settings {
        container(render_settings_panel(
            app,
            state,
            true,
            app.mouse_wheel_zoom_enabled,
            app.show_slot_content,
            app.show_slot_overflow,
            app.show_layer_panel,
            app.show_properties_panel,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            container::Style {
                background: Some(
                    if is_dark {
                        Color::from_rgba8(5, 7, 11, 0.58)
                    } else {
                        Color::from_rgba8(17, 24, 39, 0.20)
                    }
                    .into(),
                ),
                ..Default::default()
            }
        })
        .into()
    } else {
        Space::new().into()
    };

    let shortcuts_layer = if app.show_design_shortcuts {
        container(render_shortcuts_panel(true))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into()
    } else {
        Space::new().into()
    };

    let variables_layer = if app.show_design_variables {
        render_variables_panel(true, state)
    } else {
        Space::new().into()
    };

    let tailwind_inspector = render_tailwind_inspector_panel(app, state);

    let center_area: Element<'_, Message> = stack(vec![
        canvas,
        canvas_overlay,
        container(
            column(vec![Space::new().height(10).into(), toolbar])
                .padding(10)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Left),
        )
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .align_y(iced::alignment::Vertical::Top)
        .into(),
        tailwind_inspector,
        container(zoom)
            .padding(iced::Padding { top: 20.0, right: 20.0, bottom: 20.0, left: 80.0 })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom)
            .into(),
        settings_layer,
        variables_layer,
        shortcuts_layer,
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let main_content = row(vec![layers, center_area, properties]).width(Length::Fill).height(Length::Fill);

    let floating_resize_hotzones: Element<'_, Message> = {
        let hotzone_width = 8.0;
        let left_offset = if app.show_layer_panel {
            (app.layer_panel_width - hotzone_width / 2.0).max(0.0)
        } else {
            0.0
        };
        let right_offset = if app.show_properties_panel {
            (app.properties_panel_width - hotzone_width / 2.0).max(0.0)
        } else {
            0.0
        };

        let left_hotzone: Element<'_, Message> = if app.show_layer_panel {
            iced::widget::MouseArea::new(
                Space::new().width(Length::Fixed(hotzone_width)).height(Length::Fill),
            )
            .on_press(Message::View(ViewMessage::LayerPanelDragStarted))
            .on_release(Message::View(ViewMessage::GlobalMouseReleased))
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
            .into()
        } else {
            Space::new().width(0).into()
        };

        let right_hotzone: Element<'_, Message> = if app.show_properties_panel {
            iced::widget::MouseArea::new(
                Space::new().width(Length::Fixed(hotzone_width)).height(Length::Fill),
            )
            .on_press(Message::View(ViewMessage::PropertiesPanelDragStarted))
            .on_release(Message::View(ViewMessage::GlobalMouseReleased))
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
            .into()
        } else {
            Space::new().width(0).into()
        };

        container(row![
            Space::new().width(Length::Fixed(left_offset)),
            left_hotzone,
            Space::new().width(Length::Fill),
            right_hotzone,
            Space::new().width(Length::Fixed(right_offset))
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    };

    let mut final_stack = vec![main_content.into()];
    final_stack.push(floating_resize_hotzones);
    final_stack.extend(overlay::context_toolbar_layers(app, state));
    final_stack.extend(overlay::html_preview_layers(app));
    final_stack.extend(overlay::fill_picker_layers(app, state));
    final_stack.extend(overlay::effect_picker_layers(app, state));
    final_stack.extend(overlay::color_picker_layers(app));
    final_stack.extend(overlay::font_picker_layers(app, state));
    final_stack.extend(overlay::icon_picker_layers(app, state));
    final_stack.extend(overlay::tailwind_class_picker_layers(app, state));
    final_stack.extend(overlay::canvas_context_menu_layers(state));
    final_stack.push(render_image_import_dialog(app, state));
    final_stack.push(render_sticky_note_create_dialog(app, state));
    final_stack.push(design_planner_overlay);
    final_stack.push(render_figma_progress_overlay(state));

    let content = stack(final_stack).width(Length::Fill).height(Length::Fill);
    column![content].width(Length::Fill).height(Length::Fill).spacing(0).into()
}
#[cfg(test)]
mod tests;
