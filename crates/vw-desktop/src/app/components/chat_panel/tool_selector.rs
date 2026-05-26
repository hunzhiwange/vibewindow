//! 选择聊天工具输出的具体渲染组件。
//! 模块把工具名称和渲染函数绑定在一处，便于新增视图时保持边界清晰。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::svg;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::app::state::{
    MAIN_AGENT_KEY,
    SessionToolGroup,
    SessionToolSelectorTab,
    tool_display_name,
};
use crate::app::assets::Icon;
use crate::app::components::input_panel::styles::{popover_style, selectable_list_button_style};
use crate::app::{App, Message, message};
use super::utils::icon_svg;

pub(super) const SESSION_SELECTOR_SCROLLBAR_WIDTH: f32 = 4.0;
pub(super) const SESSION_SELECTOR_LIST_MAX_HEIGHT: f32 = 300.0;
const SESSION_SELECTOR_LIST_RIGHT_PADDING: f32 = 5.0;

fn count_badge<'a>(label: String, active: bool) -> Element<'a, Message> {
    container(text(label).size(10))
        .padding([3, 8])
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;
            let color = if active {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            };
            iced::widget::container::Style {
                background: Some(Background::Color(color.scale_alpha(if active {
                    if is_dark { 0.18 } else { 0.10 }
                } else if is_dark {
                    0.32
                } else {
                    0.52
                }))),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
                text_color: Some(if active {
                    palette.primary.base.text
                } else {
                    theme.palette().text.scale_alpha(0.82)
                }),
                ..Default::default()
            }
        })
        .into()
}

fn tab_button(tab: SessionToolSelectorTab, selected: bool) -> Element<'static, Message> {
    button(text(tab.label()).size(12))
        .padding([6, 10])
        .style(move |theme: &Theme, status| selectable_list_button_style(theme, status, selected))
        .on_press(Message::Chat(message::ChatMessage::SessionToolSelectorTabSelected(tab)))
        .into()
}

fn tool_action_button(label: &'static str, action: message::ChatMessage) -> Element<'static, Message> {
    button(text(label).size(11))
        .padding([6, 10])
        .style(|theme: &Theme, status| selectable_list_button_style(theme, status, false))
        .on_press(Message::Chat(action))
        .into()
}

/// 执行 session_tool_selector_popover 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub(crate) fn session_tool_selector_popover(app: &App) -> Element<'_, Message> {
    let runtime = app.current_session_runtime();
    let inventory = app.current_session_tool_inventory();

    let selected_agent_key = runtime.agent.as_deref().unwrap_or(MAIN_AGENT_KEY);
    let available_agents = app
        .agents_settings
        .entries
        .iter()
        .filter(|entry| entry.key == MAIN_AGENT_KEY || entry.enabled || runtime.agent.as_deref() == Some(entry.key.as_str()))
        .collect::<Vec<_>>();
    let summary = if inventory.is_empty() {
        "当前还没有从网关拿到可裁剪的工具清单。".to_string()
    } else {
        format!(
            "当前会话启用 {} / {} 个工具。",
            inventory.effective_tools.len(),
            inventory.base_tools.len(),
        )
    };
    let scope_hint = if inventory.static_filtered {
        "会先应用所选委托 agent 的默认工具范围，再按工具页做精确裁剪。"
    } else {
        "当前只影响本会话，不会改写系统设置里的 agents 配置。"
    };

    let tabs = row![
        tab_button(
            SessionToolSelectorTab::Agent,
            runtime.tool_selector.active_tab() == SessionToolSelectorTab::Agent,
        ),
        tab_button(
            SessionToolSelectorTab::Tools,
            runtime.tool_selector.active_tab() == SessionToolSelectorTab::Tools,
        )
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let body: Element<'_, Message> = match runtime.tool_selector.active_tab() {
        SessionToolSelectorTab::Agent => {
            let mut list = column![].spacing(6);
            let mut has_agents = false;
            for entry in available_agents {
                has_agents = true;
                let selected = selected_agent_key == entry.key;
                let allowlist_text = if entry.allowed_tools.is_empty() {
                    "未声明默认工具范围".to_string()
                } else {
                    format!("默认工具范围 {} 项", entry.allowed_tools.len())
                };
                let subtitle = if entry.model.trim().is_empty() {
                    allowlist_text
                } else {
                    format!("{} / {}", entry.model, allowlist_text)
                };
                let check: Element<'_, Message> = if selected {
                    icon_svg(Icon::Check)
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .style(|theme: &Theme, _status| svg::Style {
                            color: Some(theme.palette().text.scale_alpha(0.92)),
                        })
                        .into()
                } else {
                    Space::new().width(Length::Fixed(14.0)).into()
                };
                let agent_for_press = if entry.key == MAIN_AGENT_KEY {
                    None
                } else {
                    Some(entry.key.clone())
                };
                list = list.push(
                    button(
                        row![
                            column![
                                text(entry.label.clone()).size(13),
                                text(subtitle).size(11).style(|theme: &Theme| {
                                    iced::widget::text::Style {
                                        color: Some(theme.palette().text.scale_alpha(0.62)),
                                    }
                                })
                            ]
                            .spacing(3)
                            .width(Length::Fill),
                            check
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 10])
                    .width(Length::Fill)
                    .style(move |theme: &Theme, status| {
                        selectable_list_button_style(theme, status, selected)
                    })
                    .on_press(Message::Chat(message::ChatMessage::SessionAgentSelected(
                        agent_for_press,
                    ))),
                );
            }
            if !has_agents {
                list = list.push(
                    container(
                        text("当前没有可用于会话切换的 delegate agent。")
                            .size(11)
                            .style(|theme: &Theme| iced::widget::text::Style {
                                color: Some(theme.palette().text.scale_alpha(0.62)),
                            }),
                    )
                    .padding([4, 2]),
                );
            }
            scrollable(container(list).padding(iced::Padding {
                top: 0.0,
                right: SESSION_SELECTOR_LIST_RIGHT_PADDING,
                bottom: 0.0,
                left: 0.0,
            }))
                .direction(
                    Direction::Vertical(
                        Scrollbar::new()
                            .width(SESSION_SELECTOR_SCROLLBAR_WIDTH)
                            .scroller_width(SESSION_SELECTOR_SCROLLBAR_WIDTH),
                    ),
                )
                .height(Length::Fixed(SESSION_SELECTOR_LIST_MAX_HEIGHT))
                .into()
        }
        SessionToolSelectorTab::Tools => {
            if inventory.is_empty() {
                container(
                    text("等待网关返回可用工具列表后，这里会显示可折叠的工具分类与精确勾选。")
                        .size(11)
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.palette().text.scale_alpha(0.62)),
                        }),
                )
                .padding([4, 2])
                .into()
            } else {
                let reset_button = tool_action_button(
                    "恢复默认",
                    message::ChatMessage::SessionToolSelectorReset,
                );
                let select_all_button = tool_action_button(
                    "全选",
                    message::ChatMessage::SessionToolSelectorSelectAll,
                );
                let invert_button = tool_action_button(
                    "反选",
                    message::ChatMessage::SessionToolSelectorInvert,
                );

                let mut groups = column![].spacing(10);
                for group in SessionToolGroup::ALL {
                    let total = inventory.group_count(group);
                    if total == 0 {
                        continue;
                    }
                    let enabled = runtime.tool_selector.group_enabled_tool_count(&inventory.base_tools, group);
                    let collapsed = runtime.tool_selector.is_group_collapsed(group);
                    let group_tools = runtime.tool_selector.available_tools_for_group(&inventory.base_tools, group);
                    let group_title = format!("{} {}", group.label(), group.description());
                    let header = row![
                        button(
                            row![
                                icon_svg(if collapsed { Icon::ChevronRight } else { Icon::ChevronDown })
                                    .width(Length::Fixed(12.0))
                                    .height(Length::Fixed(12.0))
                                    .style(|theme: &Theme, _status| svg::Style {
                                        color: Some(theme.palette().text.scale_alpha(0.72)),
                                    }),
                                text(group_title)
                                    .size(12)
                                    .width(Length::Fill)
                                    .wrapping(iced::widget::text::Wrapping::None)
                                    .style(|theme: &Theme| {
                                        iced::widget::text::Style {
                                            color: Some(theme.palette().text.scale_alpha(0.78)),
                                        }
                                    })
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .padding([0, 0])
                        .style(|_theme: &Theme, _status| iced::widget::button::Style {
                            background: None,
                            border: Border::default(),
                            text_color: Color::TRANSPARENT,
                            ..Default::default()
                        })
                        .on_press(Message::Chat(
                            message::ChatMessage::SessionToolGroupCollapsedToggled(group),
                        )),
                        Space::new().width(Length::Fill),
                        button(count_badge(format!("{} / {}", enabled, total), enabled == total))
                            .padding(0)
                            .style(|_theme: &Theme, _status| iced::widget::button::Style {
                                background: None,
                                border: Border::default(),
                                text_color: Color::TRANSPARENT,
                                ..Default::default()
                            })
                            .on_press(Message::Chat(
                                message::ChatMessage::SessionToolGroupToolsToggled(group),
                            ))
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center);

                    let mut section = column![header].spacing(6);
                    if !collapsed {
                        let mut rows = column![].spacing(4);
                        for tool_id in group_tools {
                            let checked = runtime.tool_selector.is_tool_enabled(&inventory.base_tools, &tool_id);
                            let tool_label = format!("{} · {}", tool_display_name(&tool_id), tool_id);
                            let check: Element<'_, Message> = if checked {
                                icon_svg(Icon::Check)
                                    .width(Length::Fixed(14.0))
                                    .height(Length::Fixed(14.0))
                                    .style(|theme: &Theme, _status| svg::Style {
                                        color: Some(theme.palette().text.scale_alpha(0.92)),
                                    })
                                    .into()
                            } else {
                                Space::new().width(Length::Fixed(14.0)).into()
                            };
                            rows = rows.push(
                                button(
                                    row![
                                        text(tool_label)
                                            .size(12)
                                            .width(Length::Fill)
                                            .wrapping(iced::widget::text::Wrapping::None)
                                            .style(|theme: &Theme| {
                                                iced::widget::text::Style {
                                                    color: Some(theme.palette().text.scale_alpha(0.82)),
                                                }
                                            }),
                                        check
                                    ]
                                    .spacing(10)
                                    .align_y(Alignment::Center),
                                )
                                .padding([7, 10])
                                .width(Length::Fill)
                                .style(move |theme: &Theme, status| {
                                    selectable_list_button_style(theme, status, checked)
                                })
                                .on_press(Message::Chat(message::ChatMessage::SessionToolToggled(
                                    tool_id,
                                ))),
                            );
                        }
                        section = section.push(container(rows).padding(iced::Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: 0.0,
                        }));
                    }
                    groups = groups.push(section);
                }

                column![
                    row![
                        select_all_button,
                        invert_button,
                        reset_button,
                        Space::new().width(Length::Fill),
                        count_badge(
                            format!("{} / {}", inventory.effective_tools.len(), inventory.base_tools.len()),
                            runtime.tool_selector.has_custom_tool_selection() || inventory.static_filtered,
                        )
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    scrollable(container(groups).padding(iced::Padding {
                        top: 0.0,
                        right: SESSION_SELECTOR_LIST_RIGHT_PADDING,
                        bottom: 0.0,
                        left: 0.0,
                    }))
                        .direction(
                            Direction::Vertical(
                                Scrollbar::new()
                                    .width(SESSION_SELECTOR_SCROLLBAR_WIDTH)
                                    .scroller_width(SESSION_SELECTOR_SCROLLBAR_WIDTH),
                            ),
                        )
                        .height(Length::Fixed(SESSION_SELECTOR_LIST_MAX_HEIGHT))
                ]
                .spacing(10)
                .into()
            }
        }
    };

    container(column![
        text("会话控制").size(13),
        text(summary).size(11).style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.palette().text.scale_alpha(0.84)),
        }),
        text(scope_hint).size(10).style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.palette().text.scale_alpha(0.62)),
        }),
        tabs,
        body,
    ]
    .spacing(10))
    .padding([10, 12])
    .width(Length::Fixed(360.0))
    .style(popover_style)
    .into()
}
