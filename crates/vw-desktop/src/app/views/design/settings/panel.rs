//! 设计器设置视图模块，负责设置面板、快捷键说明与缩放控制的界面组合。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, scrollable, svg, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::message::{DesignMessage, ViewMessage};
use crate::app::views::design::state::{
    DesignSettingsTab, DesignState, sanitize_design_generation_parallel_pages,
};
use crate::app::{App, Message};

fn render_design_parallel_pages_setting(state: &DesignState) -> Element<'static, Message> {
    let value = sanitize_design_generation_parallel_pages(state.design_generation_parallel_pages);
    let minus_button = button(
        container(text("-").size(14))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .style(settings_round_icon_button_style)
    .on_press_maybe((value > 1).then_some(Message::Design(
        DesignMessage::DesignGenerationParallelPagesChanged((value - 1).to_string()),
    )));

    let plus_button = button(
        container(text("+").size(14))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .style(settings_round_icon_button_style)
    .on_press(Message::Design(DesignMessage::DesignGenerationParallelPagesChanged(
        (value + 1).to_string(),
    )));

    row![
        text("页面并发").size(12).style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            iced::widget::text::Style {
                color: Some(if is_dark {
                    palette.text.scale_alpha(0.85)
                } else {
                    theme.extended_palette().background.base.text.scale_alpha(0.90)
                }),
            }
        }),
        Space::new().width(Length::Fill),
        minus_button,
        container(text(value.to_string()).size(12)).padding([4, 10]).style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(p.background.weak.color.scale_alpha(0.42))),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
                ..Default::default()
            }
        }),
        plus_button,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

fn render_design_log_settings(app: &App, state: &DesignState) -> Element<'static, Message> {
    let toggle_button = button(
        row![
            text(if state.design_generation_show_all_logs { "折叠" } else { "显示" }).size(11),
            settings_icon(
                if state.design_generation_show_all_logs {
                    Icon::ChevronUp
                } else {
                    Icon::ChevronDown
                },
                10,
            ),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 8])
    .style(settings_pill_button_style)
    .on_press(Message::Design(DesignMessage::DesignGenerationShowAllLogs));

    let log_files_list: Element<'static, Message> = if state.design_generation_show_all_logs {
        let project_path = app
            .project_path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().display().to_string());
        let log_files = if state.design_generation_log_files.is_empty() {
            vec![
                text("暂无历史日志文件")
                    .size(11)
                    .style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().background.base.text.scale_alpha(0.72)),
                    })
                    .into(),
            ]
        } else {
            state
                .design_generation_log_files
                .iter()
                .map(|filename| {
                    let log_path = std::path::Path::new(&project_path)
                        .join(".vibewindow")
                        .join("design")
                        .join("logs")
                        .join(filename)
                        .display()
                        .to_string();
                    let label = filename.clone();
                    button(
                        container(text(label).size(11))
                            .width(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Left),
                    )
                    .padding([4, 8])
                    .width(Length::Fill)
                    .style(|theme: &Theme, _: iced::widget::button::Status| {
                        let p = theme.extended_palette();
                        iced::widget::button::Style {
                            background: Some(Background::Color(
                                p.background.weak.color.scale_alpha(0.24),
                            )),
                            border: Border {
                                width: 1.0,
                                color: p.background.strong.color.scale_alpha(0.42),
                                radius: 4.0.into(),
                            },
                            text_color: p.background.base.text,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::View(ViewMessage::OpenPathInFinder(log_path)))
                    .into()
                })
                .collect::<Vec<Element<'static, Message>>>()
        };

        column![
            scrollable(column(log_files).spacing(4))
                .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
                .height(Length::Fixed(172.0)),
        ]
        .spacing(4)
        .into()
    } else {
        Space::new().height(0).into()
    };

    let expanded_logs: Element<'static, Message> = if state.design_generation_show_all_logs {
        container(log_files_list)
            .width(Length::Fill)
            .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 17.0, left: 0.0 })
            .into()
    } else {
        Space::new().height(0).into()
    };

    column![
        settings_action_row(
            row![text("历史日志文件").size(13), Space::new().width(Length::Fill), toggle_button]
                .align_y(iced::Alignment::Center)
                .into(),
        ),
        expanded_logs,
    ]
    .spacing(0)
    .into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `state`: 当前视图构建所需的状态、配置或消息。
/// - `show`: 当前视图构建所需的状态、配置或消息。
/// - `wheel_zoom`: 当前视图构建所需的状态、配置或消息。
/// - `show_slot_content`: 当前视图构建所需的状态、配置或消息。
/// - `show_slot_overflow`: 当前视图构建所需的状态、配置或消息。
/// - `show_layer_panel`: 当前视图构建所需的状态、配置或消息。
/// - `show_properties_panel`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render_settings_panel(
    app: &App,
    state: &DesignState,
    show: bool,
    wheel_zoom: bool,
    show_slot_content: bool,
    show_slot_overflow: bool,
    show_layer_panel: bool,
    show_properties_panel: bool,
) -> Element<'static, Message> {
    if !show {
        return Space::new().into();
    }

    let mut nav = column![].spacing(6);
    for (label, icon, tab) in [
        ("通用", Icon::Gear, DesignSettingsTab::General),
        ("聊天", Icon::ChatTextFill, DesignSettingsTab::Chat),
    ] {
        nav = nav.push(settings_nav_item(label, icon, app.design_settings_active_tab == tab, tab));
    }

    let sidebar = container(
        column![
            text("设置")
                .size(15)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            nav,
        ]
        .spacing(14),
    )
    .width(Length::Fixed(198.0))
    .padding([20, 14]);

    let (content_icon, content_title) = match app.design_settings_active_tab {
        DesignSettingsTab::General => (Icon::Gear, "通用"),
        DesignSettingsTab::Chat => (Icon::ChatTextFill, "聊天"),
    };

    let header = row![
        row![
            settings_icon(content_icon, 16),
            text(content_title)
                .size(15)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        Space::new().width(Length::Fill),
        button(settings_icon(Icon::X, 16))
            .on_press(Message::Design(DesignMessage::ToggleSettings))
            .style(settings_icon_button_style)
            .padding(6),
    ]
    .align_y(iced::Alignment::Center);

    let general_content: Element<'static, Message> = column![
        settings_toggle_row(
            "显示图层侧边栏",
            show_layer_panel,
            Message::Design(DesignMessage::ToggleLayerPanel),
        ),
        settings_divider(),
        settings_toggle_row(
            "显示属性侧边栏",
            show_properties_panel,
            Message::Design(DesignMessage::TogglePropertiesPanel),
        ),
        settings_divider(),
        settings_toggle_row(
            "使用滚轮缩放",
            wheel_zoom,
            Message::Design(DesignMessage::ToggleMouseWheelZoom(!wheel_zoom)),
        ),
        settings_divider(),
        settings_toggle_row(
            "显示插槽内容",
            show_slot_content,
            Message::Design(DesignMessage::ToggleSlotContent(!show_slot_content)),
        ),
        settings_divider(),
        settings_toggle_row(
            "允许预览插槽溢出内容",
            show_slot_overflow,
            Message::Design(DesignMessage::ToggleSlotOverflow(!show_slot_overflow)),
        ),
    ]
    .into();

    let chat_content: Element<'static, Message> = column![
        settings_action_row(render_design_parallel_pages_setting(state)),
        settings_divider(),
        render_design_log_settings(app, state),
    ]
    .spacing(0)
    .into();

    let content = column![
        header,
        match app.design_settings_active_tab {
            DesignSettingsTab::General => general_content,
            DesignSettingsTab::Chat => chat_content,
        }
    ]
    .spacing(0);

    let panel = row![
        sidebar,
        settings_vertical_divider(),
        container(content).width(Length::Fill).padding(iced::Padding {
            top: 20.0,
            right: 28.0,
            bottom: 18.0,
            left: 28.0
        }),
    ]
    .height(Length::Fill);

    container(panel)
        .width(Length::Fixed(864.0))
        .height(Length::Fixed(608.0))
        .style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            let bg = if is_dark {
                Color::from_rgba8(30, 32, 36, 0.985)
            } else {
                Color::from_rgba8(255, 255, 255, 0.985)
            };
            let border = if is_dark {
                Color::from_rgba8(255, 255, 255, 0.10)
            } else {
                Color::from_rgba8(32, 37, 44, 0.08)
            };
            container::Style {
                background: Some(Background::Color(bg)),
                border: Border { width: 1.0, color: border, radius: 18.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(if is_dark { 0.38 } else { 0.18 }),
                    offset: iced::Vector::new(0.0, 20.0),
                    blur_radius: 40.0,
                },
                ..Default::default()
            }
        })
        .into()
}

fn settings_icon(icon: Icon, size: u16) -> iced::widget::Svg<'static, Theme> {
    svg(assets::get_icon(icon)).width(size as f32).height(size as f32).style(
        |theme: &Theme, _status| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            svg::Style {
                color: Some(if is_dark {
                    Color::from_rgba8(228, 232, 239, 1.0)
                } else {
                    Color::from_rgba8(54, 60, 70, 0.92)
                }),
            }
        },
    )
}

fn settings_nav_item(
    label: &'static str,
    icon: Icon,
    active: bool,
    tab: DesignSettingsTab,
) -> Element<'static, Message> {
    button(
        container(
            row![
                settings_icon(icon, 14),
                text(label).size(13).style(move |theme: &Theme| iced::widget::text::Style {
                    color: Some(if active {
                        let palette = theme.palette();
                        let is_dark =
                            palette.background.r + palette.background.g + palette.background.b
                                < 1.5;
                        if is_dark {
                            Color::from_rgba8(244, 247, 252, 1.0)
                        } else {
                            Color::from_rgba8(34, 39, 46, 1.0)
                        }
                    } else {
                        theme.palette().text.scale_alpha(0.72)
                    }),
                }),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding([9, 12])
        .style(move |theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            container::Style {
                background: Some(
                    if active {
                        if is_dark {
                            Color::from_rgba8(255, 255, 255, 0.08)
                        } else {
                            Color::from_rgba8(242, 244, 248, 1.0)
                        }
                    } else {
                        Color::TRANSPARENT
                    }
                    .into(),
                ),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
                ..Default::default()
            }
        }),
    )
    .on_press(Message::Design(DesignMessage::DesignSettingsSelectTab(tab)))
    .padding(0)
    .style(|_theme: &Theme, _status| button::Style {
        background: None,
        text_color: Color::TRANSPARENT,
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
        shadow: iced::Shadow::default(),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn settings_round_icon_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(p.background.weak.color.scale_alpha(0.38))),
        button::Status::Pressed => {
            Some(Background::Color(p.background.strong.color.scale_alpha(0.38)))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

fn settings_pill_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(p.background.weak.color.scale_alpha(0.36))),
        button::Status::Pressed => {
            Some(Background::Color(p.background.strong.color.scale_alpha(0.36)))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: theme.palette().text,
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
        ..Default::default()
    }
}

fn settings_toggle_row(
    label: &'static str,
    enabled: bool,
    message: Message,
) -> Element<'static, Message> {
    settings_action_row(
        row![
            text(label).size(13).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.palette().text)
            }),
            Space::new().width(Length::Fill),
            settings_switch(enabled, message),
        ]
        .align_y(iced::Alignment::Center)
        .into(),
    )
}

fn settings_action_row(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content).width(Length::Fill).padding([17, 0]).into()
}

fn settings_switch(enabled: bool, message: Message) -> Element<'static, Message> {
    let knob = || {
        container(Space::new().width(Length::Fixed(18.0)).height(Length::Fixed(18.0))).style(
            |_theme: &Theme| container::Style {
                background: Some(Color::WHITE.into()),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.14),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                ..Default::default()
            },
        )
    };

    let track = if enabled {
        row![Space::new().width(Length::Fill), knob()].align_y(iced::Alignment::Center)
    } else {
        row![knob(), Space::new().width(Length::Fill)].align_y(iced::Alignment::Center)
    };

    button(
        container(track).width(Length::Fixed(38.0)).height(Length::Fixed(22.0)).padding(2).style(
            move |_theme: &Theme| container::Style {
                background: Some(
                    if enabled {
                        Color::from_rgba8(105, 201, 88, 1.0)
                    } else {
                        Color::from_rgba8(224, 226, 229, 1.0)
                    }
                    .into(),
                ),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
                ..Default::default()
            },
        ),
    )
    .on_press(message)
    .padding(0)
    .style(|_theme: &Theme, _status| button::Style {
        background: None,
        text_color: Color::TRANSPARENT,
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
        shadow: iced::Shadow::default(),
        ..Default::default()
    })
    .into()
}

fn settings_divider() -> Element<'static, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            container::Style {
                background: Some(
                    if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.08)
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.08)
                    }
                    .into(),
                ),
                ..Default::default()
            }
        })
        .into()
}

fn settings_vertical_divider() -> Element<'static, Message> {
    container(Space::new().width(Length::Fixed(1.0)).height(Length::Fill))
        .style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            container::Style {
                background: Some(
                    if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.08)
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.08)
                    }
                    .into(),
                ),
                ..Default::default()
            }
        })
        .into()
}

fn settings_icon_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Color::from_rgba8(0, 0, 0, 0.06).into()),
        button::Status::Pressed => Some(Color::from_rgba8(0, 0, 0, 0.10).into()),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: Color::TRANSPARENT,
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
        shadow: iced::Shadow::default(),
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "panel_tests.rs"]
mod panel_tests;
