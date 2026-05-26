//! # 设计规划面板模块
//!
//! 本模块负责设计器右上角的规划面板与 Figma 进度覆盖层。
//! 顶层视图只负责摆放该面板，具体的聊天输入、会话切换、快速菜单与统计展示都在这里维护。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::tooltip::{Position as TooltipPosition, Tooltip};
use iced::widget::{
    Space, button, column, container, mouse_area, opaque, progress_bar, row, scrollable, stack,
    svg, text, text_editor,
};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::components::overlays::AboveOverlay;
use crate::app::message::DesignMessage;
use crate::app::views::design::state::{
    DesignChatRole, DesignGenerationStatus, DesignPlannerCorner, DesignPlannerTab, DesignState,
};
use crate::app::{App, Message};

use super::helpers::{
    design_contrast_text_color, design_editor_style, design_popover_style,
    design_round_icon_button_style, design_soft_popover_style, design_square_icon_button_style,
    design_tooltip_dark_style,
};
use super::selectors::{
    render_design_device_selector, render_design_executor_selector, render_design_model_selector,
    render_design_style_selector, render_design_theme_selector,
};

fn design_planner_panel_width(app: &App) -> f32 {
    let available_width = (app.window_size.0 - 32.0).max(260.0);
    (app.design_planner_panel_width * 0.75).clamp(260.0, available_width)
}

fn design_planner_panel_height(app: &App) -> f32 {
    let available_height = (app.window_size.1 - 32.0).max(320.0);
    (700.0_f32 * 0.75).min(available_height)
}

fn design_planner_header_compact_size(value: f32) -> f32 {
    value * (2.0 / 3.0)
}

fn design_chat_icon_style(alpha: f32) -> impl Fn(&Theme, svg::Status) -> svg::Style + Copy {
    move |theme: &Theme, _status: svg::Status| svg::Style {
        color: Some(theme.palette().text.scale_alpha(alpha)),
    }
}

fn design_chat_action_fill(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    if theme.palette().background.r + theme.palette().background.g + theme.palette().background.b < 1.5 {
        p.background.strong.color.scale_alpha(0.92)
    } else {
        Color::BLACK
    }
}

fn design_chat_role_accent(theme: &Theme, role: DesignChatRole) -> Color {
    let p = theme.extended_palette();
    match role {
        DesignChatRole::User => p.primary.base.color,
        DesignChatRole::Assistant => p.secondary.base.color,
        DesignChatRole::System => p.background.strong.text.scale_alpha(0.86),
    }
}

fn design_send_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = design_chat_action_fill(theme);
    let bg = match status {
        button::Status::Pressed => {
            Color::from_rgba(base.r * 0.92, base.g * 0.92, base.b * 0.92, base.a)
        }
        button::Status::Hovered => {
            Color::from_rgba(base.r * 0.96, base.g * 0.96, base.b * 0.96, base.a)
        }
        _ => base,
    };
    button::Style {
        background: Some(bg.into()),
        border: Border { radius: 6.0.into(), width: 1.0, color: Color::TRANSPARENT },
        text_color: design_contrast_text_color(base),
        ..Default::default()
    }
}

fn design_prompt_container_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            theme.palette().background.r,
            theme.palette().background.g,
            theme.palette().background.b,
            0.98,
        ))),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: p.background.strong.color.scale_alpha(0.35),
        },
        ..Default::default()
    }
}

fn truncate_with_ellipsis(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    let kept = max_chars.saturating_sub(1);
    let mut result = value.chars().take(kept).collect::<String>();
    result.push('…');
    result
}

fn design_corner_preview(corner: DesignPlannerCorner) -> Element<'static, Message> {
    let active_index = match corner {
        DesignPlannerCorner::TopLeft => 0,
        DesignPlannerCorner::TopRight => 1,
        DesignPlannerCorner::BottomLeft => 2,
        DesignPlannerCorner::BottomRight => 3,
    };

    let cell = move |index: usize| {
        container(Space::new().width(Length::Fixed(4.0)).height(Length::Fixed(4.0))).style(
            move |theme: &Theme| {
                let p = theme.extended_palette();
                let is_active = index == active_index;
                container::Style {
                    background: Some(Background::Color(if is_active {
                        theme.palette().text.scale_alpha(0.88)
                    } else {
                        Color::TRANSPARENT
                    })),
                    border: Border {
                        radius: 999.0.into(),
                        width: if is_active { 0.0 } else { 1.0 },
                        color: if is_active {
                            Color::TRANSPARENT
                        } else {
                            p.background.strong.color.scale_alpha(0.55)
                        },
                    },
                    ..Default::default()
                }
            },
        )
    };

    container(
        column![
            row![cell(0), cell(1)].spacing(3).align_y(iced::Alignment::Center),
            row![cell(2), cell(3)].spacing(3).align_y(iced::Alignment::Center),
        ]
        .spacing(3)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fixed(14.0))
    .height(Length::Fixed(14.0))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

pub(super) fn render_figma_progress_overlay<'a>(state: &'a DesignState) -> Element<'a, Message> {
    let Some(progress) = state.figma_progress.as_ref() else {
        return Space::new().into();
    };

    let progress_value = progress.progress.clamp(0.0, 1.0);
    let summary = if progress.total > 0 {
        format!(
            "{} / {} · {}%",
            progress.current.min(progress.total),
            progress.total,
            progress.percentage()
        )
    } else {
        format!("{}%", progress.percentage())
    };

    let card = container(
        column![
            text(progress.stage.title()).size(20),
            text(&progress.detail).size(14).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.palette().text.scale_alpha(0.72))
            }),
            container(progress_bar(0.0..=1.0, progress_value))
                .width(Length::Fixed(320.0))
                .height(Length::Fixed(8.0)),
            row![
                text(summary).size(13),
                Space::new().width(Length::Fill),
                text(if progress_value < 1.0 { "处理中…" } else { "即将完成…" }).size(13)
            ]
            .align_y(iced::Alignment::Center)
        ]
        .spacing(14),
    )
    .width(Length::Fixed(420.0))
    .padding([20, 22])
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(theme.palette().background.scale_alpha(0.98))),
            border: Border {
                width: 1.0,
                color: palette.background.strong.color,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.22),
                offset: iced::Vector::new(0.0, 10.0),
                blur_radius: 28.0,
            },
            ..Default::default()
        }
    });

    stack![
        opaque(container(Space::new().width(Length::Fill).height(Length::Fill)).style(|_| {
            container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.28))),
                ..Default::default()
            }
        })),
        opaque(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        )
    ]
    .into()
}

pub(super) fn render_design_planner_panel_overlay<'a>(
    app: &'a App,
    state: &'a DesignState,
) -> Element<'a, Message> {
    let panel_width = design_planner_panel_width(app);
    let collapsed_chevron_size = design_planner_header_compact_size(10.0);
    let collapsed_title_size = design_planner_header_compact_size(16.0);
    let collapsed_action_icon_size = design_planner_header_compact_size(12.0);
    let collapsed_action_button_size = design_planner_header_compact_size(30.0);
    let collapsed_horizontal_padding = design_planner_header_compact_size(12.0);
    let collapsed_vertical_padding = design_planner_header_compact_size(6.0);
    let (align_x, align_y, padding) = match app.design_planner_corner {
        DesignPlannerCorner::TopLeft => (
            iced::alignment::Horizontal::Left,
            iced::alignment::Vertical::Top,
            iced::Padding { top: 16.0, right: 16.0, bottom: 16.0, left: 16.0 },
        ),
        DesignPlannerCorner::TopRight => (
            iced::alignment::Horizontal::Right,
            iced::alignment::Vertical::Top,
            iced::Padding { top: 16.0, right: 16.0, bottom: 16.0, left: 16.0 },
        ),
        DesignPlannerCorner::BottomLeft => (
            iced::alignment::Horizontal::Left,
            iced::alignment::Vertical::Bottom,
            iced::Padding { top: 16.0, right: 16.0, bottom: 16.0, left: 16.0 },
        ),
        DesignPlannerCorner::BottomRight => (
            iced::alignment::Horizontal::Right,
            iced::alignment::Vertical::Bottom,
            iced::Padding { top: 16.0, right: 16.0, bottom: 16.0, left: 16.0 },
        ),
    };

    if !app.show_design_planner_panel {
        let collapsed = button(
            row![
                svg(assets::get_icon(Icon::ChevronUp))
                    .width(Length::Fixed(collapsed_chevron_size))
                    .height(Length::Fixed(collapsed_chevron_size))
                    .style(|theme: &Theme, _| svg::Style {
                        color: Some(theme.extended_palette().background.base.text.scale_alpha(0.82)),
                    }),
                text("使用氛围视窗进行设计...").size(collapsed_title_size).style(|theme: &Theme| {
                    iced::widget::text::Style {
                        color: Some(theme.extended_palette().background.base.text.scale_alpha(0.92)),
                    }
                }),
                Space::new().width(Length::Fill),
                container(
                    svg(assets::get_icon(Icon::ArrowUp))
                        .width(Length::Fixed(collapsed_action_icon_size))
                        .height(Length::Fixed(collapsed_action_icon_size))
                        .style(|theme: &Theme, _| svg::Style {
                            color: Some(theme.extended_palette().background.strong.text),
                        })
                )
                .width(Length::Fixed(collapsed_action_button_size))
                .height(Length::Fixed(collapsed_action_button_size))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .style(|theme: &Theme| {
                    let p = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(p.background.strong.color)),
                        border: Border {
                            radius: design_planner_header_compact_size(15.0).into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }
                })
            ]
            .spacing(design_planner_header_compact_size(8.0))
            .align_y(iced::Alignment::Center),
        )
        .padding([collapsed_vertical_padding, collapsed_horizontal_padding])
        .width(Length::Fixed(panel_width))
        .style(|theme: &Theme, status: button::Status| {
            let p = theme.extended_palette();
            let bg = match status {
                button::Status::Hovered => p.background.base.color.scale_alpha(0.92),
                button::Status::Pressed => p.background.strong.color.scale_alpha(0.98),
                _ => p.background.base.color.scale_alpha(0.98),
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: p.background.base.text,
                border: Border {
                    width: 1.0,
                    color: p.background.strong.color,
                    radius: design_planner_header_compact_size(22.0).into(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::ToggleDesignPlannerPanelCollapsed));

        return container(collapsed)
            .padding(padding)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(align_x)
            .align_y(align_y)
            .into();
    }

    let panel = render_design_planner_panel(app, state);
    container(panel)
        .padding(padding)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(align_x)
        .align_y(align_y)
        .into()
}

fn render_design_planner_panel<'a>(app: &'a App, state: &'a DesignState) -> Element<'a, Message> {
    let panel_width = design_planner_panel_width(app);
    let panel_height = design_planner_panel_height(app);
    let compact = panel_width <= 280.0;
    let header_button_size = design_planner_header_compact_size(28.0);
    let header_icon_size = design_planner_header_compact_size(12.0);
    let quick_grid_size = design_planner_header_compact_size(20.0);
    let quick_dot_size = design_planner_header_compact_size(5.0);
    let header_spacing = design_planner_header_compact_size(8.0);
    let title_size = if compact {
        design_planner_header_compact_size(16.0)
    } else {
        design_planner_header_compact_size(18.0)
    };
    let page_count = state.design_generation_pages.len();
    let module_count = state.design_generation_pages.iter().map(|page| page.modules.len()).sum::<usize>();
    let generated_count = state
        .design_generation_pages
        .iter()
        .flat_map(|page| page.modules.iter())
        .filter(|module| {
            matches!(
                module.status,
                DesignGenerationStatus::Generated | DesignGenerationStatus::Aggregated
            )
        })
        .count();
    let failed_count = state
        .design_generation_pages
        .iter()
        .flat_map(|page| page.modules.iter())
        .filter(|module| matches!(module.status, DesignGenerationStatus::Failed))
        .count();
    let active_session_title = state
        .design_chat_sessions
        .get(state.design_chat_active_session)
        .map(|session| session.title.clone())
        .unwrap_or_else(|| "New Chat".to_string());
    let compact_title = truncate_with_ellipsis(&active_session_title, 16);

    let corner_button = |corner: DesignPlannerCorner| {
        button(
            container(design_corner_preview(corner))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(header_button_size))
        .height(Length::Fixed(header_button_size))
        .padding(0)
        .style(move |theme: &Theme, status: button::Status| {
            let p = theme.extended_palette();
            let active = app.design_planner_corner == corner;
            let bg = if active {
                p.primary.base.color.scale_alpha(0.25)
            } else {
                match status {
                    button::Status::Hovered => p.background.weak.color.scale_alpha(0.45),
                    button::Status::Pressed => p.background.strong.color.scale_alpha(0.45),
                    _ => Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: 999.0.into(),
                    width: if active { 1.0 } else { 0.0 },
                    color: if active { p.primary.base.color } else { Color::TRANSPARENT },
                },
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::DesignPlannerSetCorner(corner)))
    };

    let planner_quick_menu = container(
        column![
            row![
                Space::new().width(Length::Fill),
                corner_button(DesignPlannerCorner::TopLeft),
                corner_button(DesignPlannerCorner::TopRight),
                Space::new().width(Length::Fill),
            ]
            .spacing(4),
            row![
                Space::new().width(Length::Fill),
                corner_button(DesignPlannerCorner::BottomLeft),
                corner_button(DesignPlannerCorner::BottomRight),
                Space::new().width(Length::Fill),
            ]
            .spacing(4),
        ]
        .spacing(8),
    )
    .padding(8)
    .width(Length::Fixed(92.0))
    .style(design_soft_popover_style);

    let quick_button = button(
        container(
            column![
                row![
                    container(
                        Space::new()
                            .width(Length::Fixed(quick_dot_size))
                            .height(Length::Fixed(quick_dot_size)),
                    )
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(theme.palette().text.scale_alpha(0.62))),
                        border: Border {
                            radius: 999.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }),
                    container(
                        Space::new()
                            .width(Length::Fixed(quick_dot_size))
                            .height(Length::Fixed(quick_dot_size)),
                    )
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(theme.palette().text.scale_alpha(0.62))),
                        border: Border {
                            radius: 999.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(design_planner_header_compact_size(4.0))
                .align_y(iced::Alignment::Center),
                row![
                    container(
                        Space::new()
                            .width(Length::Fixed(quick_dot_size))
                            .height(Length::Fixed(quick_dot_size)),
                    )
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(theme.palette().text.scale_alpha(0.62))),
                        border: Border {
                            radius: 999.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }),
                    container(
                        Space::new()
                            .width(Length::Fixed(quick_dot_size))
                            .height(Length::Fixed(quick_dot_size)),
                    )
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(theme.palette().text.scale_alpha(0.62))),
                        border: Border {
                            radius: 999.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(design_planner_header_compact_size(4.0))
                .align_y(iced::Alignment::Center),
            ]
            .spacing(design_planner_header_compact_size(4.0))
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(quick_grid_size))
        .height(Length::Fixed(quick_grid_size))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .style(design_round_icon_button_style)
    .on_press(Message::Design(DesignMessage::OpenDesignPlannerQuickMenu));

    let new_chat_button = {
        let btn = button(
            container(
                svg::Svg::<Theme>::new(assets::get_icon(Icon::Plus))
                    .width(Length::Fixed(header_icon_size))
                    .height(Length::Fixed(header_icon_size))
                    .style(|theme: &Theme, _| svg::Style {
                        color: Some(theme.palette().text.scale_alpha(0.68)),
                    }),
            )
            .width(Length::Fixed(header_button_size))
            .height(Length::Fixed(header_button_size))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .style(design_round_icon_button_style)
        .on_press(Message::Design(DesignMessage::DesignPlannerNewChatSession));
        let tip = container(text("新建会话").size(12)).style(design_tooltip_dark_style).padding([6, 8]);
        Tooltip::new(btn, tip, TooltipPosition::Bottom).gap(8)
    };

    let planner_header = row![
        button(
            svg(assets::get_icon(Icon::ChevronDown))
                .width(Length::Fixed(header_icon_size))
                .height(Length::Fixed(header_icon_size))
                .style(design_chat_icon_style(0.82)),
        )
        .on_press(Message::Design(DesignMessage::ToggleDesignPlannerPanelCollapsed))
        .padding(design_planner_header_compact_size(6.0))
        .style(design_square_icon_button_style),
        text(compact_title).size(title_size),
        Space::new().width(Length::Fill),
        new_chat_button,
        AboveOverlay::new(quick_button, planner_quick_menu)
            .show(state.design_planner_quick_menu_open)
            .gap(6.0)
            .on_close(Message::Design(DesignMessage::CloseDesignPlannerQuickMenu)),
    ]
    .spacing(header_spacing)
    .align_y(iced::Alignment::Center);

    let tab_button = |label: &'static str, tab: DesignPlannerTab| {
        let active = state.design_planner_active_tab == tab;
        button(text(label).size(12))
            .padding([4, 10])
            .style(move |theme: &Theme, status: button::Status| {
                let p = theme.extended_palette();
                let bg = if active {
                    p.primary.base.color.scale_alpha(0.22)
                } else {
                    match status {
                        button::Status::Hovered => p.background.weak.color.scale_alpha(0.35),
                        button::Status::Pressed => p.background.strong.color.scale_alpha(0.35),
                        _ => Color::TRANSPARENT,
                    }
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    border: Border {
                        radius: 999.0.into(),
                        width: if active { 1.0 } else { 0.0 },
                        color: if active { p.primary.base.color } else { Color::TRANSPARENT },
                    },
                    ..Default::default()
                }
            })
            .on_press(Message::Design(DesignMessage::DesignPlannerSelectTab(tab)))
    };

    let sessions = state
        .design_chat_sessions
        .iter()
        .enumerate()
        .map(|(index, session)| {
            let active = index == state.design_chat_active_session;
            button(text(session.title.as_str()).size(11))
                .padding([4, 10])
                .style(move |theme: &Theme, status: button::Status| {
                    let p = theme.extended_palette();
                    let bg = if active {
                        p.primary.base.color.scale_alpha(0.18)
                    } else {
                        match status {
                            button::Status::Hovered => p.background.weak.color.scale_alpha(0.35),
                            button::Status::Pressed => p.background.strong.color.scale_alpha(0.35),
                            _ => Color::TRANSPARENT,
                        }
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        border: Border {
                            radius: 999.0.into(),
                            width: if active { 1.0 } else { 0.0 },
                            color: if active { p.primary.base.color } else { Color::TRANSPARENT },
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::Design(DesignMessage::DesignPlannerSelectChatSession(index)))
                .into()
        })
        .collect::<Vec<Element<'a, Message>>>();

    let build_prompt_editor = || {
        let line_count = state.design_chat_input.text().lines().count().max(1);
        let visible_lines = line_count.clamp(if compact { 2 } else { 3 }, if compact { 5 } else { 6 }) as f32;
        let editor_height = 17.0 * visible_lines + 10.0;
        text_editor(&state.design_chat_input)
            .placeholder("简要描述网站需求，例如布局、模块和响应式要求")
            .height(editor_height)
            .padding(8)
            .style(design_editor_style)
            .size(if compact { 12 } else { 13 })
            .on_action(|action| {
                Message::Design(DesignMessage::DesignGenerationPromptAction(action))
            })
    };

    let build_submit_button = || -> Element<'a, Message> {
        button(
            container(
                svg::Svg::<Theme>::new(assets::get_icon(Icon::ArrowUp))
                    .width(Length::Fixed(18.0))
                    .height(Length::Fixed(18.0))
                    .style(|theme: &Theme, _| svg::Style {
                        color: Some(design_contrast_text_color(design_chat_action_fill(theme))),
                    }),
            )
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .style(design_send_button_style)
        .on_press_maybe(
            (!state.design_generation_loading)
                .then_some(Message::Design(DesignMessage::DesignGenerationSubmit)),
        )
        .into()
    };

    let build_cancel_button = || -> Element<'a, Message> {
        let cancel_size = match (state.design_generation_anim_frame / 2) % 3 {
            0 => 10.0,
            1 => 11.0,
            _ => 12.0,
        };
        let cancel_square = container(
            Space::new().width(Length::Fixed(cancel_size)).height(Length::Fixed(cancel_size)),
        )
        .style(|theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(design_contrast_text_color(
                design_chat_action_fill(theme),
            ))),
            border: Border { radius: 2.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        });
        button(
            container(cancel_square)
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .style(design_send_button_style)
        .on_press(if state.design_generation_loading {
            Message::Design(DesignMessage::DesignGenerationCancel)
        } else {
            Message::Design(DesignMessage::DesignGenerationClearChatSelection)
        })
        .into()
    };

    let build_chat_list = || {
        state
            .design_chat_messages
            .iter()
            .enumerate()
            .map(|(index, message)| {
                let (label, role) = match message.role {
                    DesignChatRole::User => ("你", DesignChatRole::User),
                    DesignChatRole::Assistant => ("AI", DesignChatRole::Assistant),
                    DesignChatRole::System => ("系统", DesignChatRole::System),
                };
                let accent = role;
                let is_step_message = matches!(message.role, DesignChatRole::Assistant)
                    && (message.content.starts_with("Calling tool:")
                        || message.content.starts_with("Step failed:"));
                let copy_button = button(text("复制").size(10))
                    .padding([2, 6])
                    .style(move |theme: &Theme, _: button::Status| button::Style {
                        background: Some(Background::Color(
                            design_chat_role_accent(theme, accent).scale_alpha(0.14),
                        )),
                        border: Border {
                            radius: 4.0.into(),
                            width: 1.0,
                            color: design_chat_role_accent(theme, accent).scale_alpha(0.26),
                        },
                        text_color: theme.extended_palette().background.base.text,
                        ..Default::default()
                    })
                    .on_press(Message::Design(DesignMessage::DesignGenerationCopyChatMessage(index)));

                if is_step_message {
                    let step_ok = !message.content.starts_with("Step failed:");
                    container(
                        row![
                            container(text(if step_ok { "✓" } else { "!" }).size(12).style(
                                move |theme: &Theme| {
                                    let p = theme.extended_palette();
                                    iced::widget::text::Style {
                                        color: Some(if step_ok {
                                            p.success.base.color
                                        } else {
                                            p.danger.base.color
                                        }),
                                    }
                                }
                            ))
                            .width(Length::Fixed(20.0))
                            .align_x(iced::alignment::Horizontal::Center),
                            text(&message.content).size(13),
                            Space::new().width(Length::Fill),
                            copy_button,
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding([9, 10])
                    .style(move |theme: &Theme| {
                        let p = theme.extended_palette();
                        let tone = if step_ok { p.success.base.color } else { p.danger.base.color };
                        container::Style {
                            background: Some(Background::Color(tone.scale_alpha(0.10))),
                            border: Border {
                                width: 1.0,
                                color: tone.scale_alpha(0.36),
                                radius: 10.0.into(),
                            },
                            text_color: Some(p.background.base.text),
                            ..Default::default()
                        }
                    })
                    .into()
                } else {
                    let selected = state.design_chat_selected_message == Some(index);
                    mouse_area(
                        container(
                            column![
                                row![
                                    text(label).size(11).style(move |theme: &Theme| {
                                        iced::widget::text::Style {
                                            color: Some(design_chat_role_accent(theme, accent)),
                                        }
                                    }),
                                    Space::new().width(Length::Fill),
                                    copy_button,
                                ]
                                .spacing(4)
                                .align_y(iced::Alignment::Center),
                                text(&message.content).size(12),
                            ]
                            .spacing(4),
                        )
                        .padding(10)
                        .style(move |theme: &Theme| {
                            let p = theme.extended_palette();
                            let accent = design_chat_role_accent(theme, accent);
                            let background = if selected {
                                accent.scale_alpha(0.14)
                            } else if matches!(message.role, DesignChatRole::User) {
                                theme.palette().primary.scale_alpha(0.10)
                            } else {
                                p.background.weak.color.scale_alpha(0.55)
                            };
                            container::Style {
                                background: Some(Background::Color(background)),
                                border: Border {
                                    width: if selected { 1.5 } else { 1.0 },
                                    color: if selected {
                                        accent.scale_alpha(0.65)
                                    } else {
                                        accent.scale_alpha(0.25)
                                    },
                                    radius: 12.0.into(),
                                },
                                ..Default::default()
                            }
                        }),
                    )
                    .on_press(Message::Design(DesignMessage::DesignGenerationSelectChatMessage(index)))
                    .into()
                }
            })
            .collect::<Vec<Element<'a, Message>>>()
    };

    let examples = row![
        container(text("公用事业公司的技术仪表盘").size(if compact { 10 } else { 11 }))
            .padding([5, 10])
            .style(design_popover_style),
        container(text("咖啡店深色粗体网站，点击开始自动生成").size(if compact { 10 } else { 11 }))
            .padding([5, 10])
            .style(design_popover_style),
    ]
    .spacing(6)
    .wrap();

    let metrics = row![
        container(text(format!("{} 页面", page_count)).size(if compact { 10 } else { 11 }))
            .padding([6, 10])
            .style(design_popover_style),
        container(text(format!("{} 模块", module_count)).size(if compact { 10 } else { 11 }))
            .padding([6, 10])
            .style(design_popover_style),
        container(text(format!("{} 已生成", generated_count)).size(if compact { 10 } else { 11 }))
            .padding([6, 10])
            .style(design_popover_style),
        container(text(format!("{} 失败", failed_count)).size(if compact { 10 } else { 11 }))
            .padding([6, 10])
            .style(design_popover_style),
    ]
    .spacing(6)
    .wrap();

    let build_planner_tabs = || {
        row![
            tab_button("对话", DesignPlannerTab::Chat),
            tab_button("页面", DesignPlannerTab::Tools)
        ]
        .spacing(8)
    };

    let build_settings_controls = || -> Element<'a, Message> {
        container(
            row![
                render_design_theme_selector(state),
                render_design_style_selector(state),
                render_design_device_selector(state)
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(30.0))
        .style(|_theme: &Theme| iced::widget::container::Style { ..Default::default() })
        .into()
    };

    let build_footer_controls = || -> Element<'a, Message> {
        row![
            container(
                row![
                    render_design_model_selector(app, state),
                    render_design_executor_selector(app, state)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fixed(30.0))
            .style(|_theme: &Theme| iced::widget::container::Style { ..Default::default() }),
            if state.design_generation_loading || state.design_chat_selected_message.is_some() {
                build_cancel_button()
            } else {
                build_submit_button()
            }
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    };

    let build_input_bar = || {
        container(
            column![build_settings_controls(), build_prompt_editor(), build_footer_controls()]
                .spacing(6),
        )
        .padding(if compact { 6 } else { 8 })
        .style(design_prompt_container_style)
    };

    let chat_tab_content: Element<'a, Message> = column![
        build_planner_tabs(),
        scrollable(row(sessions).spacing(6))
            .direction(Direction::Horizontal(Scrollbar::new().width(4).scroller_width(4)))
            .height(Length::Fixed(if compact { 30.0 } else { 34.0 })),
        text(state.design_generation_theme.description()).size(11).style(|theme: &Theme| {
            iced::widget::text::Style {
                color: Some(theme.extended_palette().background.base.text.scale_alpha(0.72)),
            }
        }),
        examples,
        scrollable(column(build_chat_list()).spacing(8))
            .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
            .height(Length::Fill),
        build_input_bar(),
    ]
    .spacing(10)
    .into();

    let tools_tab_content: Element<'a, Message> =
        column![build_planner_tabs(), metrics].spacing(10).into();

    let body = column![
        planner_header,
        if state.design_planner_active_tab == DesignPlannerTab::Chat {
            chat_tab_content
        } else {
            tools_tab_content
        }
    ]
    .spacing(if compact { 8 } else { 10 })
    .padding(if compact { 10 } else { 12 });

    container(body)
        .width(Length::Fixed(panel_width))
        .height(Length::Fixed(panel_height))
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(p.background.base.color.scale_alpha(0.99))),
                border: Border {
                    width: 1.0,
                    color: p.background.strong.color,
                    radius: 16.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.18),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            }
        })
        .into()
}
#[cfg(test)]
#[path = "planner_tests.rs"]
mod planner_tests;
