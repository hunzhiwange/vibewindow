//! # 设计生成选择器模块
//!
//! 本模块承载设计生成流程中的弹出选择器，包括 ACP 执行器、主题、风格、设备与模型。
//! 拆分后保留原有消息流与状态结构，只把具体 UI 细节从顶层视图模块移出。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::tooltip::{Position as TooltipPosition, Tooltip};
use iced::widget::{
    Space, button, column, container, mouse_area, row, scrollable, svg, text, text_input, toggler,
};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::components::input_panel::icons::acp_agent_icon;
use crate::app::components::overlays::AboveOverlay;
use crate::app::components::system_settings::SystemTab;
use crate::app::message::{DesignMessage, SettingsMessage, ViewMessage};
use crate::app::task::{TASK_MODEL_AUTO, normalize_task_model_input};
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationTheme, DesignState, DesignStyle,
};
use crate::app::{App, Message};

use super::helpers::{
    design_input_style, design_pill_button_style, design_popover_style,
    design_square_icon_button_style, design_tooltip_dark_style,
};

const ACP_SELECTOR_MAX_HEIGHT: f32 = 240.0;
const ACP_SELECTOR_SCROLLBAR_WIDTH: f32 = 4.0;
const ACP_SELECTOR_LIST_RIGHT_PADDING: f32 = 5.0;

fn design_auto_icon() -> svg::Handle {
    assets::get_icon(Icon::Star)
}

fn design_provider_logo_handle(provider_id: &str) -> svg::Handle {
    assets::get_provider_icon(provider_id)
}

fn design_model_input_placeholder() -> &'static str {
    "auto / provider/model / ACP 模型 ID"
}

fn design_model_input_hint() -> String {
    "通过 ACP 网关将模型 ID 传给所选 ACP 智能体；未出现在列表中的模型也可直接填写".to_string()
}

fn design_theme_icon(theme: DesignGenerationTheme) -> Icon {
    match theme {
        DesignGenerationTheme::Shadcn => Icon::Square,
        DesignGenerationTheme::Nitro => Icon::Speedometer2,
        DesignGenerationTheme::Halo => Icon::Star,
        DesignGenerationTheme::Lunaris => Icon::Code,
    }
}

fn design_style_icon(style: DesignStyle) -> Icon {
    match style {
        DesignStyle::Default => Icon::Circle,
        DesignStyle::Minimalist => Icon::Square,
        DesignStyle::Modern => Icon::SymmetryHorizontal,
        DesignStyle::Business => Icon::Columns,
        DesignStyle::Creative => Icon::Bezier,
        DesignStyle::Retro => Icon::Clock,
        DesignStyle::Tech => Icon::Code,
        DesignStyle::Elegant => Icon::Type,
        DesignStyle::Vibrant => Icon::Palette,
        DesignStyle::Dark => Icon::Circle,
    }
}

fn design_device_icon(device: DesignGenerationDevice) -> Icon {
    match device {
        DesignGenerationDevice::Auto => Icon::Star,
        DesignGenerationDevice::DesktopWeb => Icon::LayoutTextWindow,
        DesignGenerationDevice::MobileApp => Icon::FileText,
        DesignGenerationDevice::Tablet => Icon::Columns,
    }
}

fn design_model_toggle_label(app: &App, auto_model: bool, model: &str) -> String {
    if auto_model {
        return "自动模型".to_string();
    }

    let parsed = if model.contains('/') {
        let parts = model.splitn(2, '/').collect::<Vec<_>>();
        if parts.len() == 2 { Some((parts[0].to_string(), parts[1].to_string())) } else { None }
    } else {
        None
    };

    let display = match parsed.as_ref() {
        Some((provider_id, model_id)) => app
            .model_settings
            .providers
            .iter()
            .find(|p| &p.id == provider_id)
            .and_then(|p| p.models.iter().find(|m| &m.id == model_id))
            .map(|m| m.name.clone()),
        None => app
            .model_settings
            .providers
            .iter()
            .find_map(|p| p.models.iter().find(|m| m.id == model).map(|m| m.name.clone())),
    };

    display.unwrap_or_else(|| {
        parsed.as_ref().map(|(_, model_id)| model_id.clone()).unwrap_or_else(|| model.to_string())
    })
}

fn design_model_icon<'a>(auto_model: bool, app: &'a App, model: &str) -> svg::Svg<'a, Theme> {
    if auto_model {
        svg::Svg::<Theme>::new(design_auto_icon())
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0))
            .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) })
    } else {
        let provider_id = if model.contains('/') {
            model.split('/').next().unwrap_or("agent").to_string()
        } else {
            app.model_settings
                .providers
                .iter()
                .find_map(|p| p.models.iter().any(|m| m.id == model).then_some(p.id.clone()))
                .unwrap_or_else(|| "agent".to_string())
        };
        svg::Svg::<Theme>::new(design_provider_logo_handle(&provider_id))
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0))
            .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) })
    }
}

pub(super) fn render_design_executor_selector<'a>(
    app: &'a App,
    state: &'a DesignState,
) -> Element<'a, Message> {
    let current_agent = app.acp_agent.clone();
    let current_label = current_agent.as_deref().unwrap_or("ACP 智能体").to_string();
    let executor_toggle_btn = button(
        container(
            row![
                acp_agent_icon(current_agent.as_deref().unwrap_or("ACP 智能体"), 14.0),
                text(current_label.clone()).size(12)
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Shrink)
        .height(Length::Fixed(30.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .style(design_pill_button_style)
    .padding([0, 10])
    .on_press(Message::Design(DesignMessage::ToggleDesignGenerationExecutorPopover));
    let executor_tip = container(text(format!("ACP 智能体：{}", current_label)).size(12))
        .style(design_tooltip_dark_style)
        .padding([6, 8]);
    let executor_toggle =
        Tooltip::new(executor_toggle_btn, executor_tip, TooltipPosition::Top).gap(8);

    let mut executor_list = column![].spacing(4);
    let default_selected = current_agent.is_none();
    let default_check: Element<'_, Message> = if default_selected {
        svg::Svg::<Theme>::new(assets::get_icon(Icon::Check))
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0))
            .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().primary) })
            .into()
    } else {
        Space::new().width(Length::Fixed(14.0)).into()
    };

    let default_btn = button(
        row![
            acp_agent_icon("ACP 智能体", 14.0),
            text("ACP 智能体").size(13),
            Space::new().width(Length::Fill),
            default_check
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .style(move |theme: &Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered);
        let p = theme.extended_palette();
        let bg = if hovered {
            Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
        } else if default_selected {
            Some(Background::Color(Color::from_rgba(
                theme.palette().primary.r,
                theme.palette().primary.g,
                theme.palette().primary.b,
                0.10,
            )))
        } else {
            None
        };
        button::Style {
            background: bg,
            border: Border { radius: 6.0.into(), width: 0.0, color: theme.palette().primary },
            text_color: theme.palette().text,
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::DesignGenerationAcpAgentSelected(None)));
    executor_list = executor_list.push(default_btn);

    for agent in app.acp_agents.iter() {
        let agent = agent.as_str().to_owned();
        let selected = current_agent.as_ref() == Some(&agent);
        let check: Element<'_, Message> = if selected {
            svg::Svg::<Theme>::new(assets::get_icon(Icon::Check))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().primary) })
                .into()
        } else {
            Space::new().width(Length::Fixed(14.0)).into()
        };

        let select_btn = button(
            row![
                acp_agent_icon(&agent, 14.0),
                text(agent.clone()).size(13),
                Space::new().width(Length::Fill),
                check
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered);
            let p = theme.extended_palette();
            let bg = if hovered {
                Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
            } else if selected {
                Some(Background::Color(Color::from_rgba(
                    theme.palette().primary.r,
                    theme.palette().primary.g,
                    theme.palette().primary.b,
                    0.10,
                )))
            } else {
                None
            };
            button::Style {
                background: bg,
                border: Border { radius: 6.0.into(), width: 0.0, color: theme.palette().primary },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::DesignGenerationAcpAgentSelected(Some(agent))));
        executor_list = executor_list.push(select_btn);
    }

    let executor_popover = container(
        scrollable(container(executor_list).padding(iced::Padding {
            top: 0.0,
            right: ACP_SELECTOR_LIST_RIGHT_PADDING,
            bottom: 0.0,
            left: 0.0,
        }))
        .id(iced::widget::Id::new("design_executor_selector_scroll"))
        .direction(Direction::Vertical(
            Scrollbar::new()
                .width(ACP_SELECTOR_SCROLLBAR_WIDTH)
                .scroller_width(ACP_SELECTOR_SCROLLBAR_WIDTH),
        ))
        .height(Length::Fixed(ACP_SELECTOR_MAX_HEIGHT)),
    )
    .style(design_popover_style)
    .padding(8)
    .width(Length::Fixed(220.0));

    AboveOverlay::new(executor_toggle, executor_popover)
        .show(state.design_generation_executor_popover)
        .gap(6.0)
        .on_close(Message::Design(DesignMessage::CloseDesignGenerationExecutorPopover))
        .into()
}

pub(super) fn render_design_theme_selector<'a>(state: &'a DesignState) -> Element<'a, Message> {
    let current_theme = state.design_generation_theme;
    let theme_toggle_btn = button(
        container(
            svg::Svg::<Theme>::new(assets::get_icon(design_theme_icon(current_theme)))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .style(design_square_icon_button_style)
    .padding(0)
    .on_press(Message::Design(DesignMessage::ToggleDesignGenerationThemePopover));
    let theme_tip = container(text(format!("主题：{}", current_theme.label())).size(12))
        .style(design_tooltip_dark_style)
        .padding([6, 8]);
    let theme_toggle = Tooltip::new(theme_toggle_btn, theme_tip, TooltipPosition::Top).gap(8);

    let mut theme_list = column![].spacing(4);
    for theme in DesignGenerationTheme::ALL {
        let selected = theme == current_theme;
        let icon = design_theme_icon(theme);
        let check: Element<'_, Message> = if selected {
            svg::Svg::<Theme>::new(assets::get_icon(Icon::Check))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().primary) })
                .into()
        } else {
            Space::new().width(Length::Fixed(14.0)).into()
        };

        let select_btn = button(
            row![
                svg::Svg::<Theme>::new(assets::get_icon(icon))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
                text(theme.label()).size(13),
                Space::new().width(Length::Fill),
                check
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered);
            let p = theme.extended_palette();
            let bg = if hovered {
                Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
            } else if selected {
                Some(Background::Color(Color::from_rgba(
                    theme.palette().primary.r,
                    theme.palette().primary.g,
                    theme.palette().primary.b,
                    0.10,
                )))
            } else {
                None
            };
            button::Style {
                background: bg,
                border: Border { radius: 6.0.into(), width: 0.0, color: theme.palette().primary },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::DesignGenerationThemeSelected(theme)));
        theme_list = theme_list.push(select_btn);
    }

    let theme_popover =
        container(theme_list).style(design_popover_style).padding(8).width(Length::Fixed(180.0));

    AboveOverlay::new(theme_toggle, theme_popover)
        .show(state.design_generation_theme_popover)
        .gap(6.0)
        .on_close(Message::Design(DesignMessage::CloseDesignGenerationThemePopover))
        .into()
}

pub(super) fn render_design_style_selector<'a>(state: &'a DesignState) -> Element<'a, Message> {
    let current_style = state.design_generation_style;
    let style_toggle_btn = button(
        container(
            svg::Svg::<Theme>::new(assets::get_icon(design_style_icon(current_style)))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .style(design_square_icon_button_style)
    .padding(0)
    .on_press(Message::Design(DesignMessage::ToggleDesignGenerationStylePopover));
    let style_tip = container(text(format!("风格：{}", current_style.label())).size(12))
        .style(design_tooltip_dark_style)
        .padding([6, 8]);
    let style_toggle = Tooltip::new(style_toggle_btn, style_tip, TooltipPosition::Top).gap(8);

    let mut style_list = column![].spacing(4);
    for style in DesignStyle::all() {
        let selected = style == current_style;
        let icon = design_style_icon(style);
        let check: Element<'_, Message> = if selected {
            svg::Svg::<Theme>::new(assets::get_icon(Icon::Check))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().primary) })
                .into()
        } else {
            Space::new().width(Length::Fixed(14.0)).into()
        };

        let select_btn = button(
            row![
                svg::Svg::<Theme>::new(assets::get_icon(icon))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
                text(style.label()).size(13),
                Space::new().width(Length::Fill),
                check
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered);
            let p = theme.extended_palette();
            let bg = if hovered {
                Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
            } else if selected {
                Some(Background::Color(Color::from_rgba(
                    theme.palette().primary.r,
                    theme.palette().primary.g,
                    theme.palette().primary.b,
                    0.10,
                )))
            } else {
                None
            };
            button::Style {
                background: bg,
                border: Border { radius: 6.0.into(), width: 0.0, color: theme.palette().primary },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::DesignGenerationStyleSelected(style)));
        style_list = style_list.push(select_btn);
    }

    let style_popover =
        container(style_list).style(design_popover_style).padding(8).width(Length::Fixed(180.0));

    AboveOverlay::new(style_toggle, style_popover)
        .show(state.design_generation_style_popover)
        .gap(6.0)
        .on_close(Message::Design(DesignMessage::CloseDesignGenerationStylePopover))
        .into()
}

pub(super) fn render_design_device_selector<'a>(state: &'a DesignState) -> Element<'a, Message> {
    let current_device = state.design_generation_device;
    let device_toggle_btn = button(
        container(
            svg::Svg::<Theme>::new(assets::get_icon(design_device_icon(current_device)))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .style(design_square_icon_button_style)
    .padding(0)
    .on_press(Message::Design(DesignMessage::ToggleDesignGenerationDevicePopover));
    let device_tip = container(text(format!("网站类型：{}", current_device.label())).size(12))
        .style(design_tooltip_dark_style)
        .padding([6, 8]);
    let device_toggle = Tooltip::new(device_toggle_btn, device_tip, TooltipPosition::Top).gap(8);

    let mut device_list = column![].spacing(4);
    for device in DesignGenerationDevice::ALL {
        let selected = device == current_device;
        let icon = design_device_icon(device);
        let check: Element<'_, Message> = if selected {
            svg::Svg::<Theme>::new(assets::get_icon(Icon::Check))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().primary) })
                .into()
        } else {
            Space::new().width(Length::Fixed(14.0)).into()
        };

        let select_btn = button(
            row![
                svg::Svg::<Theme>::new(assets::get_icon(icon))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
                column![
                    text(device.label()).size(13),
                    text(device.description()).size(11).style(|theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(
                                theme.extended_palette().background.base.text.scale_alpha(0.7),
                            ),
                        }
                    }),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                check
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered);
            let p = theme.extended_palette();
            let bg = if hovered {
                Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
            } else if selected {
                Some(Background::Color(Color::from_rgba(
                    theme.palette().primary.r,
                    theme.palette().primary.g,
                    theme.palette().primary.b,
                    0.10,
                )))
            } else {
                None
            };
            button::Style {
                background: bg,
                border: Border { radius: 6.0.into(), width: 0.0, color: theme.palette().primary },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::DesignGenerationDeviceSelected(device)));
        device_list = device_list.push(select_btn);
    }

    let device_popover =
        container(device_list).style(design_popover_style).padding(8).width(Length::Fixed(240.0));

    AboveOverlay::new(device_toggle, device_popover)
        .show(state.design_generation_device_popover)
        .gap(6.0)
        .on_close(Message::Design(DesignMessage::CloseDesignGenerationDevicePopover))
        .into()
}

pub(super) fn render_design_model_selector<'a>(
    app: &'a App,
    state: &'a DesignState,
) -> Element<'a, Message> {
    let model_value = normalize_task_model_input(&state.design_generation_model);
    let auto_model = model_value == TASK_MODEL_AUTO;
    let model = model_value.as_str();
    let query = app.model_settings.query.trim().to_ascii_lowercase();
    let popup_width = 320.0;

    let model_toggle_btn = button(
        container(design_model_icon(auto_model, app, model))
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(30.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .style(design_square_icon_button_style)
    .padding(0)
    .on_press(Message::Design(DesignMessage::ToggleDesignGenerationModelPopover));
    let model_tip = container(
        text(format!("模型：{}", design_model_toggle_label(app, auto_model, model))).size(12),
    )
    .style(design_tooltip_dark_style)
    .padding([6, 8]);
    let model_toggle = Tooltip::new(model_toggle_btn, model_tip, TooltipPosition::Top).gap(8);

    let matches_query =
        |provider_id: &str, provider_name: &str, model_id: &str, model_name: &str| -> bool {
            if query.is_empty() {
                return true;
            }
            provider_id.to_ascii_lowercase().contains(&query)
                || provider_name.to_ascii_lowercase().contains(&query)
                || model_id.to_ascii_lowercase().contains(&query)
                || model_name.to_ascii_lowercase().contains(&query)
        };
    let is_selected = |current: &str, provider_id: &str, model_id: &str| -> bool {
        if current.contains('/') {
            current == format!("{}/{}", provider_id, model_id)
        } else {
            current == model_id
        }
    };

    let search = text_input("搜索模型…", &app.model_settings.query)
        .on_input(|v| Message::Settings(SettingsMessage::ModelQueryChanged(v)))
        .padding([8, 10])
        .size(13);

    let mut model_list = column![].spacing(8);
    let mut any_models = false;

    if app.model_settings.loading && app.model_settings.providers.is_empty() {
        model_list = model_list.push(text("加载中…").size(13).style(|t: &Theme| {
            iced::widget::text::Style { color: Some(t.extended_palette().background.base.text) }
        }));
    } else {
        for p in &app.model_settings.providers {
            let mut models = p
                .models
                .iter()
                .filter(|m| m.enabled)
                .filter(|m| matches_query(&p.id, &p.name, &m.id, &m.name))
                .collect::<Vec<_>>();
            if models.is_empty() {
                continue;
            }
            models.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));

            let header = row![
                svg::Svg::<Theme>::new(design_provider_logo_handle(&p.id))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
                text(p.name.clone()).size(13),
                container(text("")).width(Length::Fill),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center);

            model_list = model_list.push(container(header).padding([4, 0]));
            any_models = true;

            for m in models {
                let model_key = format!("{}/{}", p.id, m.id);
                let selected = !auto_model && is_selected(model, &p.id, &m.id);
                let selected_badge: Element<'_, Message> = if selected {
                    row![
                        text("已选择").size(11).style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.palette().primary),
                        }),
                        svg::Svg::<Theme>::new(assets::get_icon(Icon::Check))
                            .width(Length::Fixed(14.0))
                            .height(Length::Fixed(14.0))
                            .style(|theme: &Theme, _| svg::Style {
                                color: Some(theme.palette().primary),
                            })
                    ]
                    .spacing(4)
                    .align_y(iced::Alignment::Center)
                    .into()
                } else {
                    Space::new().width(Length::Fixed(44.0)).into()
                };

                let select_btn = button(
                    row![text(m.name.clone()).size(13).width(Length::Fill), selected_badge]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                )
                .padding(iced::Padding { top: 6.0, right: 8.0, bottom: 6.0, left: 8.0 })
                .width(Length::Fill)
                .style(move |theme: &Theme, status: button::Status| {
                    let hovered = matches!(status, button::Status::Hovered);
                    let p = theme.extended_palette();
                    let bg = if hovered {
                        Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
                    } else if selected {
                        Some(Background::Color(Color::from_rgba(
                            theme.palette().primary.r,
                            theme.palette().primary.g,
                            theme.palette().primary.b,
                            0.10,
                        )))
                    } else {
                        None
                    };
                    button::Style {
                        background: bg,
                        border: Border {
                            radius: 6.0.into(),
                            width: 0.0,
                            color: theme.palette().primary,
                        },
                        text_color: theme.palette().text,
                        ..Default::default()
                    }
                })
                .on_press(Message::Design(DesignMessage::DesignGenerationModelSelected(model_key)));

                model_list = model_list.push(select_btn);
                any_models = true;
            }
        }

        if !any_models {
            model_list =
                model_list.push(text("暂无可选模型（请先在系统设置里启用模型）").size(13).style(
                    |t: &Theme| iced::widget::text::Style {
                        color: Some(t.extended_palette().background.base.text),
                    },
                ));
        }
    }

    let manage_btn: Element<'_, Message> = {
        let btn = button(
            container(
                svg::Svg::<Theme>::new(assets::get_icon(Icon::Sliders))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(|theme: &Theme, _| svg::Style { color: Some(theme.palette().text) }),
            )
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .style(design_square_icon_button_style)
        .on_press(Message::View(ViewMessage::OpenSystemSettingsTab(SystemTab::Models)));
        let tip =
            container(text("管理模型").size(12)).style(design_tooltip_dark_style).padding([6, 8]);
        Tooltip::new(btn, tip, TooltipPosition::Right).gap(8).into()
    };

    let providers_btn: Element<'_, Message> = {
        let can_add_provider = !state.design_generation_loading;
        let btn = button(
            container(
                svg::Svg::<Theme>::new(assets::get_icon(Icon::Plus))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(move |theme: &Theme, _| svg::Style {
                        color: Some(if can_add_provider {
                            theme.palette().text
                        } else {
                            theme.extended_palette().background.weak.text
                        }),
                    }),
            )
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .style(design_square_icon_button_style)
        .on_press_maybe(
            can_add_provider
                .then_some(Message::View(ViewMessage::OpenSystemSettingsTab(SystemTab::Providers))),
        );
        let tip =
            container(text("连接供应商").size(12)).style(design_tooltip_dark_style).padding([6, 8]);
        Tooltip::new(btn, tip, TooltipPosition::Right).gap(8).into()
    };

    let boxed_content = column![
        container(
            column![
                row![
                    Tooltip::new(
                        mouse_area(
                            svg::Svg::<Theme>::new(design_auto_icon())
                                .width(Length::Fixed(18.0))
                                .height(Length::Fixed(18.0))
                                .style(|theme: &Theme, _| svg::Style {
                                    color: Some(theme.palette().text),
                                }),
                        )
                        .on_press(Message::Design(
                            DesignMessage::DesignGenerationModelSelected(
                                TASK_MODEL_AUTO.to_string(),
                            )
                        )),
                        container(text("自动模型自动选择可用模型").size(12))
                            .style(design_tooltip_dark_style)
                            .padding([6, 8]),
                        TooltipPosition::Right,
                    )
                    .gap(8),
                    text("自动模型").style(|theme: &Theme| {
                        let palette = theme.palette();
                        let is_dark =
                            palette.background.r + palette.background.g + palette.background.b
                                < 1.5;
                        iced::widget::text::Style {
                            color: Some(if is_dark {
                                palette.text.scale_alpha(0.85)
                            } else {
                                theme.extended_palette().background.base.text.scale_alpha(0.90)
                            }),
                        }
                    }),
                    container(text("")).width(Length::Fill),
                    toggler(auto_model).on_toggle(|_| {
                        Message::Design(DesignMessage::DesignGenerationModelSelected(
                            TASK_MODEL_AUTO.to_string(),
                        ))
                    })
                ]
                .spacing(8),
            ]
            .spacing(10),
        )
        .padding([6, 8]),
        container(
            column![
                text("手工填写模型").size(12).style(|theme: &Theme| {
                    let palette = theme.palette();
                    let is_dark =
                        palette.background.r + palette.background.g + palette.background.b < 1.5;
                    iced::widget::text::Style {
                        color: Some(if is_dark {
                            palette.text.scale_alpha(0.85)
                        } else {
                            theme.extended_palette().background.base.text.scale_alpha(0.90)
                        }),
                    }
                }),
                text_input(design_model_input_placeholder(), model)
                    .on_input(|value| {
                        Message::Design(DesignMessage::DesignGenerationModelChanged(
                            normalize_task_model_input(&value),
                        ))
                    })
                    .padding([8, 10])
                    .style(design_input_style)
                    .size(13),
                text(design_model_input_hint()).size(11).style(|theme: &Theme| {
                    let palette = theme.palette();
                    let is_dark =
                        palette.background.r + palette.background.g + palette.background.b < 1.5;
                    iced::widget::text::Style {
                        color: Some(if is_dark {
                            palette.text.scale_alpha(0.78)
                        } else {
                            theme.extended_palette().background.base.text.scale_alpha(0.82)
                        }),
                    }
                })
            ]
            .spacing(6),
        )
        .padding(iced::Padding { top: 2.0, right: 8.0, bottom: 6.0, left: 8.0 }),
        container(
            column![
                row![search.width(Length::Fill), providers_btn, manage_btn]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                scrollable(container(model_list).padding(iced::Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: 0.0,
                }))
                .id(iced::widget::Id::new("design_generation_model_selector_scroll"))
                .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
                .height(Length::Fixed(260.0)),
            ]
            .spacing(8),
        )
        .padding(iced::Padding { top: 2.0, right: 8.0, bottom: 8.0, left: 8.0 }),
    ]
    .spacing(6);

    let model_pop_content = container(boxed_content)
        .style(design_popover_style)
        .padding(4)
        .width(Length::Fixed(popup_width));

    AboveOverlay::new(model_toggle, model_pop_content)
        .show(state.design_generation_model_popover)
        .gap(6.0)
        .on_close(Message::Design(DesignMessage::CloseDesignGenerationModelPopover))
        .into()
}
#[cfg(test)]
#[path = "selectors_tests.rs"]
mod selectors_tests;
