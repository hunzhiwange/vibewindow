//! 清理工具视图。
//!
//! 该模块实现桌面端清理工具的界面状态、交互消息和渲染逻辑，负责把扫描结果、安全确认与用户操作组织成可预测的 UI 流程。

use crate::app::message::CleanerToolMessage;
use crate::app::message::cleaner_tool::{
    CleanerScanGroup, CleanerScanItem, format_bytes, selected_scan_totals,
};
use crate::app::{App, Message};
use iced::widget::{
    Space, button, checkbox, column, container, progress_bar, row, scrollable, text, text_editor,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme, Vector};

/// 公开的 view 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn view(app: &App) -> Element<'_, Message> {
    let header = row![
        text("电脑垃圾清理工具").size(20),
        Space::new().width(Length::Fill),
        if let Some(msg) = &app.cleaner_notification {
            text(msg).size(14).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().success.base.color),
            })
        } else {
            text("").size(14)
        }
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center);

    let platform_name = cleaner_platform_name(app);

    let spinner = ["◐", "◓", "◑", "◒"][app.cleaner_animation_frame % 4];
    let (selected_count, selected_bytes) = selected_scan_totals(app);
    let total_found_bytes =
        app.cleaner_scan_report.as_ref().map(|report| report.total_bytes).unwrap_or(0);
    let matched_items =
        app.cleaner_scan_report.as_ref().map(|report| report.matched_items).unwrap_or(0);
    let progress_value = cleaner_progress_value(app);
    let progress_label = cleaner_progress_label(app, selected_count, matched_items);
    let hero_badge_label = if app.cleaner_last_run_completed {
        Some("✓ 清理完成")
    } else if app.cleaner_cancelling {
        Some("正在取消")
    } else if app.cleaner_running {
        Some("执行中")
    } else if app.cleaner_scanning {
        Some("搜索中")
    } else if app.cleaner_scanned {
        Some("可勾选")
    } else {
        None
    };
    let hero_badge: Element<'_, Message> = if let Some(label) = hero_badge_label {
        container(text(label).size(12)).padding([6, 10]).style(completion_badge_style).into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let hero_title = if app.cleaner_cancelling {
        format!("{spinner} 正在取消清理")
    } else if app.cleaner_running {
        format!("{spinner} 正在清理已勾选项目")
    } else if app.cleaner_scanning {
        format!("{spinner} 正在搜索可清理文件")
    } else if app.cleaner_last_run_completed {
        "✓ 清理完成".to_string()
    } else if app.cleaner_scanned {
        format!("共发现可清理文件 {}", format_bytes(total_found_bytes))
    } else {
        "先搜索，再按树状展开勾选".to_string()
    };

    let hero_subtitle = if app.cleaner_cancelling {
        "会在当前步骤结束后停止后续清理。".to_string()
    } else if app.cleaner_running {
        if selected_bytes > 0 {
            format!("本次已选 {} 项，预计清理 {}。", selected_count, format_bytes(selected_bytes))
        } else {
            format!("本次已选 {} 项，正在直接执行清理。", selected_count)
        }
    } else if app.cleaner_scanning {
        "正在按系统垃圾、应用垃圾、上网垃圾三组进行扫描。".to_string()
    } else if app.cleaner_last_run_completed {
        "本轮清理已结束，建议重新搜索确认剩余项目。".to_string()
    } else if app.cleaner_scanned {
        format!(
            "命中 {} 个可清理项，当前已勾选 {} 项，共 {}。",
            matched_items,
            selected_count,
            format_bytes(selected_bytes)
        )
    } else {
        "下载、安装包、上网垃圾及敏感应用默认不勾选。".to_string()
    };

    let primary_label = if app.cleaner_cancelling {
        format!("取消中 {spinner}")
    } else if app.cleaner_running {
        format!("清理中 {spinner}")
    } else if app.cleaner_scanning {
        format!("搜索中 {spinner}")
    } else if app.cleaner_scanned {
        "立即清理".to_string()
    } else {
        "开始搜索".to_string()
    };

    let primary_message =
        if app.cleaner_scanned { CleanerToolMessage::Run } else { CleanerToolMessage::Scan };
    let primary_enabled = !app.cleaner_running && !app.cleaner_scanning;

    let primary_button = button(text(primary_label).size(22))
        .width(Length::Fill)
        .padding([20, 34])
        .style(|theme: &Theme, status| {
            let pressed = matches!(status, button::Status::Pressed);
            let hovered = matches!(status, button::Status::Hovered);
            let disabled = matches!(status, button::Status::Disabled);
            let base = Color::from_rgb8(0x34, 0xD3, 0x99);
            let hover = Color::from_rgb8(0x4B, 0xDA, 0xA6);
            let pressed_bg = Color::from_rgb8(0x26, 0xB8, 0x83);
            let bg = if disabled {
                base.scale_alpha(0.55)
            } else if pressed {
                pressed_bg
            } else if hovered {
                hover
            } else {
                base
            };
            let shadow = iced::Shadow {
                color: Color::from_rgba8(52, 211, 153, if hovered { 0.28 } else { 0.18 }),
                offset: Vector::new(0.0, if hovered { 12.0 } else { 8.0 }),
                blur_radius: if hovered { 24.0 } else { 18.0 },
            };
            let mut style = button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgba8(255, 255, 255, 96.0 / 255.0),
                    radius: 20.0.into(),
                },
                text_color: Color::WHITE,
                shadow,
                ..Default::default()
            };
            if disabled {
                style.text_color = theme.palette().text.scale_alpha(0.65);
            }
            style
        });
    let primary_button = if primary_enabled {
        primary_button.on_press(Message::CleanerTool(primary_message))
    } else {
        primary_button
    };

    let secondary_button =
        button(text(if app.cleaner_scanned { "重新搜索" } else { "清空输出" }).size(15))
            .padding([12, 20])
            .style(secondary_button_style);
    let secondary_button = if app.cleaner_scanned && !app.cleaner_running && !app.cleaner_scanning {
        secondary_button.on_press(Message::CleanerTool(CleanerToolMessage::Scan))
    } else if !app.cleaner_scanned && !app.cleaner_running && !app.cleaner_scanning {
        secondary_button.on_press(Message::CleanerTool(CleanerToolMessage::Clear))
    } else {
        secondary_button
    };

    let cancel_button =
        button(text("取消").size(15)).padding([12, 20]).style(secondary_button_style);
    let cancel_button = if (app.cleaner_running || app.cleaner_scanning) && !app.cleaner_cancelling
    {
        cancel_button.on_press(Message::CleanerTool(CleanerToolMessage::Cancel))
    } else {
        cancel_button
    };

    let toolbar = if app.cleaner_running || app.cleaner_cancelling || app.cleaner_scanning {
        row![secondary_button, cancel_button]
            .spacing(10)
            .width(Length::Fill)
            .align_y(Alignment::Center)
    } else {
        row![secondary_button].spacing(10).width(Length::Fill).align_y(Alignment::Center)
    };

    let selected_status: Element<'_, Message> = if app.cleaner_scanned {
        text(format!("已勾选 {} 项", selected_count))
            .size(13)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().primary.strong.color),
            })
            .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };
    let selected_bytes_view: Element<'_, Message> = if app.cleaner_scanned {
        text(format!("已选择 {}", format_bytes(selected_bytes)))
            .size(13)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().success.strong.color),
            })
            .into()
    } else {
        text("搜索后可展开树形目录并勾选").size(13).into()
    };

    let platform_row = row![
        pill("当前平台", false),
        text(platform_name).size(14),
        selected_status,
        Space::new().width(Length::Fill),
        selected_bytes_view
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let hero = container(
        column![
            row![
                column![
                    row![text(hero_title).size(28), hero_badge]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    text(hero_subtitle).size(13).style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().secondary.strong.color),
                    }),
                    toolbar,
                ]
                .spacing(10)
                .width(Length::FillPortion(7)),
                column![primary_button].width(Length::FillPortion(5)).align_x(Alignment::End)
            ]
            .spacing(24)
            .align_y(Alignment::Center),
            container(
                column![
                    row![
                        text(progress_label).size(13),
                        Space::new().width(Length::Fill),
                        text(format!("{:.0}%", progress_value * 100.0)).size(13)
                    ]
                    .align_y(Alignment::Center),
                    container(progress_bar(0.0..=1.0, progress_value)).height(8)
                ]
                .spacing(10)
            )
            .padding([14, 16])
            .style(progress_card_style),
            row![
                summary_metric_card("扫描命中", format!("{matched_items} 项"), false),
                summary_metric_card("当前勾选", format!("{selected_count} 项"), false),
                summary_metric_card("预计释放", format_bytes(selected_bytes), true),
            ]
            .spacing(12)
        ]
        .spacing(18),
    )
    .padding(24)
    .style(hero_card_style);

    let tree_status: Element<'_, Message> = if app.cleaner_scanned {
        text(format!("共 {}", format_bytes(total_found_bytes)))
            .size(13)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().primary.strong.color),
            })
            .into()
    } else {
        text("未开始").size(13).into()
    };
    let tree_header = row![
        column![
            text("扫描树").size(16),
            text("先看分组，再展开到具体目录。敏感应用默认保持未勾选。").size(12).style(
                |theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().secondary.strong.color),
                }
            )
        ]
        .spacing(4),
        Space::new().width(Length::Fill),
        tree_status
    ]
    .align_y(Alignment::Center);

    let tree_panel = container(column![tree_header, render_scan_tree(app)].spacing(14))
        .padding(16)
        .height(Length::Fill)
        .style(panel_card_style)
        .width(Length::FillPortion(6));

    let clear_button = button(text("清空输出").size(14))
        .padding([10, 18])
        .style(secondary_button_style)
        .on_press(Message::CleanerTool(CleanerToolMessage::Clear));

    let tip = text(match app.open_external_platform {
        Some(crate::app::state::RuntimePlatform::MacOs) => {
            "提示：会直接在当前 macOS 上执行清理；涉及系统目录时会弹出管理员授权窗口。"
        }
        Some(crate::app::state::RuntimePlatform::Windows) => {
            "提示：会直接在当前 Windows 上执行清理；涉及系统目录时会触发 UAC 管理员确认。"
        }
        Some(crate::app::state::RuntimePlatform::Linux) => {
            "提示：当前 gateway 宿主为 Linux，暂不支持直接执行清理。"
        }
        None => "提示：正在通过 gateway 获取宿主平台；扫描会以 gateway 所在系统为准。",
    })
    .size(12)
    .style(|theme: &Theme| iced::widget::text::Style {
        color: Some(theme.extended_palette().secondary.strong.color),
    });

    let status_card = container(
        column![
            row![text("清理说明").size(15), pill("敏感应用默认不勾选", true)]
                .spacing(10)
                .align_y(Alignment::Center),
            text("先搜索、再树状展开勾选、最后执行清理。建议清理前关闭下载任务、IDE 和浏览器。")
                .size(12),
        ]
        .spacing(8),
    )
    .padding(14)
    .style(info_card_style);

    let editor = text_editor(&app.cleaner_output_editor)
        .on_action(|action| Message::CleanerTool(CleanerToolMessage::EditorAction(action)))
        .height(Length::Fill);

    let log_header =
        row![text("日志输出").size(15), Space::new().width(Length::Fill), clear_button,]
            .align_y(Alignment::Center);

    let log_panel = container(column![log_header, editor].spacing(12))
        .padding(16)
        .height(Length::Fill)
        .style(log_panel_style)
        .width(Length::FillPortion(5));

    let content = column![
        header,
        platform_row,
        hero,
        row![tree_panel, log_panel].spacing(16).height(Length::Fill).align_y(Alignment::Start),
        status_card,
        tip,
    ]
    .spacing(12)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.base.color.into()),
                ..Default::default()
            }
        })
        .into()
}

fn cleaner_platform_name(app: &App) -> &'static str {
    match app.open_external_platform {
        Some(crate::app::state::RuntimePlatform::MacOs) => "macOS",
        Some(crate::app::state::RuntimePlatform::Windows) => "Windows",
        Some(crate::app::state::RuntimePlatform::Linux) => "Linux",
        None => "Gateway 宿主",
    }
}

fn render_scan_tree(app: &App) -> Element<'_, Message> {
    let Some(report) = app.cleaner_scan_report.as_ref() else {
        return container(
            column![
                text("尚未搜索").size(18),
                text("点击上方“开始搜索”后，会先扫描系统垃圾和应用垃圾，再按树状展开具体项目。")
                    .size(13)
                    .style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().secondary.strong.color),
                    }),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    };

    let mut tree = column![].spacing(12).width(Length::Fill);
    for group in &report.groups {
        tree = tree.push(render_scan_group(app, group));
    }

    scrollable(
        column![
            row![
                summary_metric_card("分组", report.groups.len().to_string(), false),
                summary_metric_card("命中", report.matched_items.to_string(), false),
                summary_metric_card("总量", format_bytes(report.total_bytes), true),
            ]
            .spacing(12),
            tree
        ]
        .spacing(14),
    )
    .height(Length::Fill)
    .into()
}

fn render_scan_group<'a>(app: &'a App, group: &'a CleanerScanGroup) -> Element<'a, Message> {
    let expanded = app.cleaner_tree_expanded.contains(&group.id);
    let chevron = if expanded { "▾" } else { "▸" };
    let selected_count =
        group.items.iter().filter(|item| item_selected(app, item.id.as_str())).count();

    let header = button(
        row![
            container(text(chevron).size(18))
                .width(28)
                .center_x(Length::Fill)
                .center_y(Length::Shrink),
            column![
                text(&group.title).size(15),
                text(&group.subtitle).size(12).style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().secondary.strong.color),
                }),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![
                text(if group.total_bytes > 0 {
                    format_bytes(group.total_bytes)
                } else {
                    "很干净".to_string()
                })
                .size(14),
                text(format!("已勾选 {}", selected_count)).size(11).style(|theme: &Theme| {
                    iced::widget::text::Style {
                        color: Some(theme.extended_palette().primary.strong.color),
                    }
                }),
            ]
            .spacing(2)
            .align_x(Alignment::End)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([10, 12])
    .style(ghost_button_style)
    .on_press(Message::CleanerTool(CleanerToolMessage::ToggleTreeNode(group.id.clone())));

    let mut content = column![header].spacing(8).width(Length::Fill);
    if expanded {
        for item in &group.items {
            content = content.push(render_scan_item(app, item));
        }
    }

    container(content)
        .padding(10)
        .style(move |theme: &Theme| group_card_style(theme, expanded))
        .into()
}

fn render_scan_item<'a>(app: &'a App, item: &'a CleanerScanItem) -> Element<'a, Message> {
    let expanded = app.cleaner_tree_expanded.contains(&item.id);
    let checked = item_selected(app, item.id.as_str());
    let detail_toggle = button(text(if expanded { "▾" } else { "▸" }).size(16))
        .style(ghost_button_style)
        .padding([4, 8])
        .on_press(Message::CleanerTool(CleanerToolMessage::ToggleTreeNode(item.id.clone())));
    let sensitive_tag: Element<'a, Message> = if item.sensitive {
        pill("敏感应用", true).into()
    } else {
        Space::new().width(Length::Shrink).into()
    };
    let selection_hint: Element<'a, Message> = if checked {
        text("已纳入本次清理")
            .size(11)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().success.strong.color),
            })
            .into()
    } else {
        text("未选择").size(11).into()
    };

    let mut content = column![
        row![
            item_checkbox(app, item.id.as_str()),
            detail_toggle,
            column![
                row![text(&item.title).size(14), sensitive_tag]
                    .spacing(8)
                    .align_y(Alignment::Center),
                text(&item.subtitle).size(12).style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().secondary.strong.color),
                }),
                selection_hint
            ]
            .spacing(4)
            .width(Length::Fill),
            text(if item.total_bytes > 0 {
                format!("共 {}", format_bytes(item.total_bytes))
            } else {
                "很干净".to_string()
            })
            .size(13)
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    ]
    .spacing(8)
    .width(Length::Fill);

    if expanded {
        for detail in &item.details {
            content = content.push(
                row![
                    Space::new().width(Length::Fixed(60.0)),
                    text("•").size(12),
                    column![
                        text(&detail.label).size(12),
                        text(simplify_path(&detail.path)).size(11).style(|theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(theme.extended_palette().secondary.strong.color),
                            }
                        }),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    text(if detail.total_bytes > 0 {
                        format_bytes(detail.total_bytes)
                    } else {
                        "0 B".to_string()
                    })
                    .size(12)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            );
        }
    }

    container(content)
        .padding([10, 12])
        .style(move |theme: &Theme| item_card_style(theme, checked, item.sensitive))
        .into()
}

fn item_checkbox<'a>(app: &'a App, id: &str) -> Element<'a, Message> {
    match id {
        "system_temp" => checkbox(app.cleaner_clear_system_temp)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleSystemTemp(value)))
            .into(),
        "app_cache" => checkbox(app.cleaner_clear_app_cache)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleAppCache(value)))
            .into(),
        "logs" => checkbox(app.cleaner_clear_logs)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleLogs(value)))
            .into(),
        "package_cache" => checkbox(app.cleaner_clear_package_cache)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::TogglePackageCache(value)))
            .into(),
        "downloads" => checkbox(app.cleaner_clear_downloads)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleDownloads(value)))
            .into(),
        "trash" => checkbox(app.cleaner_empty_trash)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleTrash(value)))
            .into(),
        "installers" => checkbox(app.cleaner_clear_installers)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleInstallers(value)))
            .into(),
        "other_apps" => checkbox(app.cleaner_clear_other_apps)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleOtherApps(value)))
            .into(),
        "wechat_work" => checkbox(app.cleaner_clear_wechat_work)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleWeChatWork(value)))
            .into(),
        "wechat" => checkbox(app.cleaner_clear_wechat)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleWeChat(value)))
            .into(),
        "qq" => checkbox(app.cleaner_clear_qq)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleQq(value)))
            .into(),
        "dingtalk" => checkbox(app.cleaner_clear_dingtalk)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleDingTalk(value)))
            .into(),
        "feishu" => checkbox(app.cleaner_clear_feishu)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleFeishu(value)))
            .into(),
        "safari" => checkbox(app.cleaner_clear_safari)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleSafari(value)))
            .into(),
        "chrome" => checkbox(app.cleaner_clear_chrome)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleChrome(value)))
            .into(),
        "edge" => checkbox(app.cleaner_clear_edge)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleEdge(value)))
            .into(),
        "firefox" => checkbox(app.cleaner_clear_firefox)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleFirefox(value)))
            .into(),
        "mail" => checkbox(app.cleaner_clear_mail)
            .label("")
            .on_toggle(|value| Message::CleanerTool(CleanerToolMessage::ToggleMail(value)))
            .into(),
        _ => checkbox(false).label("").into(),
    }
}

fn item_selected(app: &App, id: &str) -> bool {
    match id {
        "system_temp" => app.cleaner_clear_system_temp,
        "app_cache" => app.cleaner_clear_app_cache,
        "logs" => app.cleaner_clear_logs,
        "package_cache" => app.cleaner_clear_package_cache,
        "downloads" => app.cleaner_clear_downloads,
        "trash" => app.cleaner_empty_trash,
        "installers" => app.cleaner_clear_installers,
        "other_apps" => app.cleaner_clear_other_apps,
        "wechat_work" => app.cleaner_clear_wechat_work,
        "wechat" => app.cleaner_clear_wechat,
        "qq" => app.cleaner_clear_qq,
        "dingtalk" => app.cleaner_clear_dingtalk,
        "feishu" => app.cleaner_clear_feishu,
        "safari" => app.cleaner_clear_safari,
        "chrome" => app.cleaner_clear_chrome,
        "edge" => app.cleaner_clear_edge,
        "firefox" => app.cleaner_clear_firefox,
        "mail" => app.cleaner_clear_mail,
        _ => false,
    }
}

fn cleaner_progress_value(app: &App) -> f32 {
    if app.cleaner_cancelling {
        0.95
    } else if app.cleaner_running {
        0.82 + ((app.cleaner_animation_frame % 18) as f32 / 18.0) * 0.15
    } else if app.cleaner_scanning {
        0.18 + ((app.cleaner_animation_frame % 20) as f32 / 20.0) * 0.52
    } else if app.cleaner_last_run_completed {
        1.0
    } else if app.cleaner_scanned {
        0.78
    } else {
        0.0
    }
}

fn cleaner_progress_label(app: &App, selected_count: usize, matched_items: usize) -> String {
    if app.cleaner_cancelling {
        "正在等待当前步骤结束后取消".to_string()
    } else if app.cleaner_running {
        format!("正在处理 {selected_count} 个已勾选项目")
    } else if app.cleaner_scanning {
        "正在扫描系统垃圾、应用垃圾与上网垃圾".to_string()
    } else if app.cleaner_last_run_completed {
        "清理已完成，建议再次搜索确认结果".to_string()
    } else if app.cleaner_scanned {
        format!("已完成搜索，命中 {matched_items} 项")
    } else {
        "尚未开始搜索".to_string()
    }
}

fn simplify_path(path: &str) -> String {
    path.replace("$HOME", "~").replace("%USERPROFILE%", "用户目录")
}

fn pill<'a>(label: &'a str, emphasize: bool) -> iced::widget::Container<'a, Message> {
    container(text(label).size(11)).padding([4, 8]).style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let background =
            if emphasize { palette.danger.weak.color } else { palette.background.weak.color };
        let text_color =
            if emphasize { palette.danger.strong.color } else { palette.background.base.text };
        iced::widget::container::Style {
            background: Some(background.into()),
            border: Border {
                width: 1.0,
                color: text_color.scale_alpha(0.18),
                radius: 999.0.into(),
            },
            text_color: Some(text_color),
            ..Default::default()
        }
    })
}

fn hero_card_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(
            Color::from_rgba8(241, 253, 248, if is_dark_mode(theme) { 0.12 } else { 0.96 }).into(),
        ),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.18),
            radius: 24.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(16, 185, 129, 0.10),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

fn panel_card_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.18),
            radius: 18.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(0, 0, 0, if is_dark_mode(theme) { 0.10 } else { 0.04 }),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

fn log_panel_style(theme: &Theme) -> iced::widget::container::Style {
    let dark_mode = is_dark_mode(theme);
    iced::widget::container::Style {
        background: Some(
            if dark_mode {
                Color::from_rgba8(0xB8, 0xC0, 0xCC, 0.12)
            } else {
                Color::from_rgba8(0xF1, 0xF3, 0xF6, 0.96)
            }
            .into(),
        ),
        border: Border {
            width: 1.0,
            color: theme.extended_palette().background.strong.color.scale_alpha(if dark_mode {
                0.3
            } else {
                0.18
            }),
            radius: 18.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(0, 0, 0, if dark_mode { 0.1 } else { 0.05 }),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

fn info_card_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(
            Color::from_rgba8(255, 244, 227, if is_dark_mode(theme) { 0.14 } else { 0.95 }).into(),
        ),
        border: Border {
            width: 1.0,
            color: theme.extended_palette().background.strong.color.scale_alpha(0.2),
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(245, 158, 11, 0.10),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

fn summary_metric_card<'a>(
    title: &'a str,
    value: String,
    accent: bool,
) -> iced::widget::Container<'a, Message> {
    container(
        column![
            text(title).size(11).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().secondary.strong.color),
            }),
            text(value).size(18)
        ]
        .spacing(6),
    )
    .padding([12, 14])
    .width(Length::Fill)
    .style(move |theme: &Theme| summary_metric_style(theme, accent))
}

fn progress_card_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(
            Color::from_rgba8(255, 255, 255, if is_dark_mode(theme) { 0.06 } else { 0.62 }).into(),
        ),
        border: Border {
            width: 1.0,
            color: palette.primary.strong.color.scale_alpha(0.14),
            radius: 18.0.into(),
        },
        ..Default::default()
    }
}

fn completion_badge_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(palette.success.weak.color.into()),
        border: Border {
            width: 1.0,
            color: palette.success.strong.color.scale_alpha(0.24),
            radius: 999.0.into(),
        },
        text_color: Some(palette.success.strong.color),
        ..Default::default()
    }
}

fn summary_metric_style(theme: &Theme, accent: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let background = if accent {
        palette.primary.weak.color.scale_alpha(if is_dark_mode(theme) { 0.18 } else { 0.72 })
    } else {
        palette.background.base.color.scale_alpha(if is_dark_mode(theme) { 0.16 } else { 0.72 })
    };
    iced::widget::container::Style {
        background: Some(background.into()),
        border: Border {
            width: 1.0,
            color: if accent {
                palette.primary.strong.color.scale_alpha(0.18)
            } else {
                palette.background.strong.color.scale_alpha(0.12)
            },
            radius: 16.0.into(),
        },
        ..Default::default()
    }
}

fn group_card_style(theme: &Theme, expanded: bool) -> iced::widget::container::Style {
    let background = if expanded {
        theme.extended_palette().primary.weak.color.scale_alpha(0.12)
    } else {
        theme.extended_palette().background.base.color.scale_alpha(0.42)
    };
    iced::widget::container::Style {
        background: Some(background.into()),
        border: Border {
            width: 1.0,
            color: if expanded {
                theme.extended_palette().primary.strong.color.scale_alpha(0.18)
            } else {
                theme.extended_palette().background.strong.color.scale_alpha(0.14)
            },
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(0, 0, 0, if expanded { 0.08 } else { 0.03 }),
            offset: Vector::new(0.0, if expanded { 8.0 } else { 4.0 }),
            blur_radius: if expanded { 18.0 } else { 10.0 },
        },
        ..Default::default()
    }
}

fn item_card_style(
    theme: &Theme,
    selected: bool,
    sensitive: bool,
) -> iced::widget::container::Style {
    let background = if selected {
        theme.extended_palette().success.weak.color.scale_alpha(if is_dark_mode(theme) {
            0.14
        } else {
            0.86
        })
    } else if sensitive {
        theme.extended_palette().danger.weak.color.scale_alpha(if is_dark_mode(theme) {
            0.10
        } else {
            0.58
        })
    } else {
        theme.extended_palette().background.base.color.scale_alpha(0.30)
    };
    iced::widget::container::Style {
        background: Some(background.into()),
        border: Border {
            width: 1.0,
            color: if selected {
                theme.extended_palette().success.strong.color.scale_alpha(0.22)
            } else if sensitive {
                theme.extended_palette().danger.strong.color.scale_alpha(0.16)
            } else {
                theme.extended_palette().background.strong.color.scale_alpha(0.12)
            },
            radius: 12.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(0, 0, 0, if selected { 0.05 } else { 0.02 }),
            offset: Vector::new(0.0, if selected { 6.0 } else { 3.0 }),
            blur_radius: if selected { 14.0 } else { 8.0 },
        },
        ..Default::default()
    }
}

fn ghost_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let hovered = matches!(status, button::Status::Hovered);
    let pressed = matches!(status, button::Status::Pressed);
    button::Style {
        background: Some(
            if pressed {
                palette.primary.weak.color.scale_alpha(0.32)
            } else if hovered {
                palette.primary.weak.color.scale_alpha(0.22)
            } else {
                Color::TRANSPARENT
            }
            .into(),
        ),
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 12.0.into() },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let dark_mode = is_dark_mode(theme);
    let disabled = matches!(status, button::Status::Disabled);
    let hovered = matches!(status, button::Status::Hovered);
    let pressed = matches!(status, button::Status::Pressed);
    let base = if dark_mode {
        Color::from_rgba8(0x8E, 0x96, 0xA3, 0.22)
    } else {
        Color::from_rgba8(0xEE, 0xF1, 0xF5, 0.98)
    };
    let bg = if disabled {
        base.scale_alpha(0.55)
    } else if pressed {
        base.scale_alpha(0.88)
    } else if hovered {
        base.scale_alpha(1.0)
    } else {
        base
    };
    button::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(if dark_mode { 0.28 } else { 0.18 }),
            radius: 14.0.into(),
        },
        text_color: palette.background.base.text,
        shadow: iced::Shadow {
            color: Color::from_rgba8(0, 0, 0, if hovered { 0.08 } else { 0.04 }),
            offset: Vector::new(0.0, if hovered { 6.0 } else { 3.0 }),
            blur_radius: if hovered { 14.0 } else { 8.0 },
        },
        ..Default::default()
    }
}

fn is_dark_mode(theme: &Theme) -> bool {
    theme.palette().background.r + theme.palette().background.g + theme.palette().background.b < 1.5
}

#[cfg(test)]
#[path = "cleaner_tool_tests.rs"]
mod cleaner_tool_tests;
