//! 大文件工具视图模块，负责承载大文件查看入口与相关状态展示。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::message::large_file_tool::{LargeFileCategory, LargeFileToolMessage, format_bytes};
use crate::app::{App, Message};
use iced::widget::{
    Space, button, checkbox, column, container, progress_bar, row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

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
pub fn view(app: &App) -> Element<'_, Message> {
    let spinner = ["◐", "◓", "◑", "◒"][app.large_file_animation_frame % 4];
    let report = app.large_file_report.as_ref();
    let total_files = report.map(|value| value.total_files).unwrap_or(0);
    let total_bytes = report.map(|value| value.total_bytes).unwrap_or(0);
    let category_count = report.map(|value| value.categories.len()).unwrap_or(0);
    let selected_count = app.large_file_selected_entries.len();
    let visible_categories: Vec<&LargeFileCategory> = report
        .map(|value| {
            value
                .categories
                .iter()
                .filter(|category| {
                    app.large_file_active_filter == "all"
                        || category.id == app.large_file_active_filter
                })
                .collect()
        })
        .unwrap_or_default();

    let header = row![
        text("大文件查找工具").size(20),
        Space::new().width(Length::Fill),
        if let Some(notification) = &app.large_file_notification {
            text(notification).size(14).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().success.base.color),
            })
        } else {
            text("").size(14)
        }
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center);

    let hero_title = if app.large_file_scanning {
        format!("{spinner} 正在扫描 50MB 以上的大文件")
    } else if app.large_file_deleting {
        "正在删除已选中的大文件".to_string()
    } else if app.large_file_scanned {
        format!("共发现 {} 个大文件", total_files)
    } else {
        "按目录扫描 50MB 以上的大文件".to_string()
    };

    let hero_subtitle = if app.large_file_scanning {
        format!(
            "{} · 已处理 {}/{} · 已命中 {} 个候选文件",
            app.large_file_progress_label,
            app.large_file_processed_files,
            app.large_file_total_files,
            total_files
        )
    } else if app.large_file_deleting {
        format!("已选择 {} 个文件，删除完成后会自动从列表中移除。", selected_count)
    } else if app.large_file_scanned {
        format!(
            "扫描目录：{}，共命中 {} 个文件，总占用 {}。",
            report.map(|value| value.root.as_str()).unwrap_or(app.large_file_root.as_str()),
            total_files,
            format_bytes(total_bytes)
        )
    } else {
        "默认扫描当前用户目录，也可以先切换到指定目录再开始。".to_string()
    };

    let root_input = text_input("输入要扫描的目录", &app.large_file_root)
        .on_input(|value| Message::LargeFileTool(LargeFileToolMessage::RootChanged(value)))
        .padding([12, 14])
        .size(16)
        .width(Length::Fill);

    let choose_button =
        button(text("选择目录").size(15)).padding([12, 18]).style(secondary_button_style);
    let choose_button = if app.large_file_scanning || app.large_file_deleting {
        choose_button
    } else {
        choose_button.on_press(Message::LargeFileTool(LargeFileToolMessage::PickRoot))
    };

    let primary_button = button(
        text(if app.large_file_scanning {
            format!("扫描中 {spinner}")
        } else {
            "开始扫描".to_string()
        })
        .size(20),
    )
    .padding([18, 26])
    .width(Length::Shrink)
    .style(primary_button_style);
    let primary_button = if app.large_file_scanning || app.large_file_deleting {
        primary_button
    } else {
        primary_button.on_press(Message::LargeFileTool(LargeFileToolMessage::Scan))
    };

    let cancel_button: Element<'_, Message> = if app.large_file_scanning {
        button(text("取消扫描").size(15))
            .padding([12, 18])
            .style(danger_button_style)
            .on_press(Message::LargeFileTool(LargeFileToolMessage::CancelScan))
            .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let controls = row![root_input, choose_button, primary_button, cancel_button]
        .spacing(12)
        .align_y(Alignment::Center);

    let stats = row![
        stat_card("扫描阈值", "50MB+"),
        stat_card("分类数", category_count.to_string()),
        stat_card("命中文件", total_files.to_string()),
        stat_card("总占用", format_bytes(total_bytes)),
        stat_card("已选文件", selected_count.to_string())
    ]
    .spacing(12)
    .width(Length::Fill)
    .align_y(Alignment::Center);

    let filter_row = row![
        filter_button(app, "all", "全部"),
        filter_button(app, "giga", "1GB+"),
        filter_button(app, "500m", "500MB-1GB"),
        filter_button(app, "100m", "100MB-500MB"),
        filter_button(app, "50m", "50MB-100MB")
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let progress_panel = container(
        column![
            row![
                text(&app.large_file_progress_label).size(13),
                Space::new().width(Length::Fill),
                text(format!("{:.0}%", app.large_file_progress_value * 100.0)).size(13)
            ]
            .align_y(Alignment::Center),
            container(progress_bar(0.0..=1.0, app.large_file_progress_value)).height(8),
            text(if app.large_file_current_path.is_empty() {
                "等待开始扫描".to_string()
            } else {
                app.large_file_current_path.clone()
            })
            .size(13)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().secondary.strong.color),
            })
        ]
        .spacing(10),
    )
    .padding([14, 16])
    .style(card_style);

    let selection_toolbar = row![
        button(text("全选当前结果").size(14))
            .padding([10, 14])
            .style(secondary_button_style)
            .on_press_maybe(
                (!app.large_file_scanning
                    && !app.large_file_deleting
                    && !visible_categories.is_empty())
                .then_some(Message::LargeFileTool(LargeFileToolMessage::SelectVisibleEntries))
            ),
        button(text("清空选择").size(14))
            .padding([10, 14])
            .style(secondary_button_style)
            .on_press_maybe(
                (!app.large_file_scanning && !app.large_file_deleting && selected_count > 0)
                    .then_some(Message::LargeFileTool(LargeFileToolMessage::ClearSelection))
            ),
        button(text(format!("删除所选 ({selected_count})")).size(14))
            .padding([10, 14])
            .style(danger_button_style)
            .on_press_maybe(
                (!app.large_file_scanning && !app.large_file_deleting && selected_count > 0)
                    .then_some(Message::LargeFileTool(LargeFileToolMessage::DeleteSelected))
            ),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let hero = container(
        column![
            text(hero_title).size(28),
            text(hero_subtitle).size(14).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().secondary.strong.color),
            }),
            controls,
            progress_panel,
            stats,
            filter_row,
            selection_toolbar
        ]
        .spacing(16),
    )
    .width(Length::Fill)
    .padding(24)
    .style(hero_style);

    let content = if app.large_file_scanning {
        scanning_view(app, spinner)
    } else if app.large_file_scanned && total_files == 0 {
        empty_view("当前目录下没有发现 50MB 以上的大文件")
    } else if app.large_file_scanned && visible_categories.is_empty() {
        empty_view("当前筛选分类下没有匹配结果")
    } else if app.large_file_scanned {
        let mut list = column![].spacing(16).width(Length::Fill);
        for category in visible_categories {
            list = list.push(category_card(app, category));
        }
        scrollable(list).height(Length::Fill).into()
    } else {
        empty_view("准备好目录后点击“开始扫描”即可生成分类结果")
    };

    container(column![header, hero, content].spacing(18))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding { top: 16.0, right: 20.0, bottom: 20.0, left: 20.0 })
        .into()
}

fn stat_card(label: impl Into<String>, value: impl Into<String>) -> Element<'static, Message> {
    let label = label.into();
    let value = value.into();

    container(
        column![
            text(label).size(12).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().secondary.strong.color),
            }),
            text(value).size(22)
        ]
        .spacing(6),
    )
    .width(Length::Fill)
    .padding(16)
    .style(card_style)
    .into()
}

fn filter_button<'a>(app: &App, id: &'a str, label: &'a str) -> Element<'a, Message> {
    let active = app.large_file_active_filter == id;
    let button = button(text(label).size(14))
        .padding([8, 14])
        .style(move |theme: &Theme, status| filter_button_style(theme, status, active));

    if app.large_file_scanning || app.large_file_deleting {
        button.into()
    } else {
        button
            .on_press(Message::LargeFileTool(LargeFileToolMessage::SelectFilter(id.to_string())))
            .into()
    }
}

fn scanning_view<'a>(app: &'a App, spinner: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(format!("{spinner} 正在扫描，请稍候")).size(22),
            text("扫描过程中会持续切换当前路径，并在后台并行统计文件大小。").size(14),
            text(format!(
                "当前路径：{}",
                if app.large_file_current_path.is_empty() {
                    "等待扫描".to_string()
                } else {
                    app.large_file_current_path.clone()
                }
            ))
            .size(13)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().secondary.strong.color),
            })
        ]
        .spacing(10)
        .width(Length::Fill)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(28)
    .style(card_style)
    .into()
}

fn empty_view<'a>(message: &'a str) -> Element<'a, Message> {
    container(column![text(message).size(20)].width(Length::Fill).align_x(Alignment::Center))
        .width(Length::Fill)
        .padding(28)
        .style(card_style)
        .into()
}

fn category_card<'a>(app: &'a App, category: &'a LargeFileCategory) -> Element<'a, Message> {
    let mut files = column![].spacing(10).width(Length::Fill);

    for file in &category.files {
        let selected = app.large_file_selected_entries.contains(&file.path);
        let path = file.path.clone();
        files = files.push(
            container(
                row![
                    checkbox(selected).label("").on_toggle(move |selected| {
                        Message::LargeFileTool(LargeFileToolMessage::ToggleEntrySelection {
                            path: path.clone(),
                            selected,
                        })
                    }),
                    column![
                        row![
                            text(&file.name).size(16),
                            Space::new().width(Length::Fill),
                            text(format_bytes(file.size_bytes)).size(14).style(|theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(theme.extended_palette().primary.strong.color),
                                }
                            })
                        ]
                        .align_y(Alignment::Center),
                        text(&file.path).size(13).style(|theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(theme.extended_palette().secondary.strong.color),
                            }
                        }),
                        text(&file.parent).size(12).style(|theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(theme.extended_palette().secondary.base.color),
                            }
                        })
                    ]
                    .spacing(6)
                    .width(Length::Fill)
                ]
                .spacing(12)
                .align_y(Alignment::Start)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .padding(14)
            .style(sub_card_style),
        );
    }

    container(
        column![
            row![
                column![
                    text(&category.title).size(22),
                    text(&category.subtitle).size(13).style(|theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.extended_palette().secondary.strong.color),
                        }
                    })
                ]
                .spacing(6),
                Space::new().width(Length::Fill),
                column![
                    text(format!("{} 个文件", category.files.len())).size(14),
                    text(format_bytes(category.total_bytes)).size(16).style(|theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.extended_palette().primary.strong.color),
                        }
                    })
                ]
                .spacing(4)
                .align_x(Alignment::End)
            ]
            .align_y(Alignment::Center),
            files
        ]
        .spacing(16)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding(20)
    .style(card_style)
    .into()
}

fn hero_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba8(77, 100, 255, 0.10))),
        border: Border {
            radius: 24.0.into(),
            width: 1.0,
            color: theme.extended_palette().primary.weak.color,
        },
        ..Default::default()
    }
}

fn card_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(theme.extended_palette().background.weak.color)),
        border: Border {
            radius: 20.0.into(),
            width: 1.0,
            color: theme.extended_palette().background.strong.color,
        },
        ..Default::default()
    }
}

fn sub_card_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(theme.extended_palette().background.base.color)),
        border: Border {
            radius: 16.0.into(),
            width: 1.0,
            color: theme.extended_palette().background.strong.color,
        },
        ..Default::default()
    }
}

fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let pressed = matches!(status, button::Status::Pressed);
    let disabled = matches!(status, button::Status::Disabled);
    let background = if disabled {
        Color::from_rgba8(107, 124, 255, 0.45)
    } else if pressed {
        Color::from_rgb8(0x4F, 0x63, 0xE8)
    } else if hovered {
        Color::from_rgb8(0x7A, 0x89, 0xFF)
    } else {
        Color::from_rgb8(0x6B, 0x7C, 0xFF)
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: if disabled { theme.palette().text.scale_alpha(0.7) } else { Color::WHITE },
        border: Border { radius: 18.0.into(), width: 1.0, color: Color::TRANSPARENT },
        ..Default::default()
    }
}

fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let pressed = matches!(status, button::Status::Pressed);
    let disabled = matches!(status, button::Status::Disabled);
    let background = if disabled {
        theme.extended_palette().background.weak.color.scale_alpha(0.55)
    } else if pressed {
        theme.extended_palette().background.strong.color
    } else if hovered {
        theme.extended_palette().background.weak.color
    } else {
        theme.extended_palette().background.base.color
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: theme.palette().text,
        border: Border {
            radius: 16.0.into(),
            width: 1.0,
            color: theme.extended_palette().background.strong.color,
        },
        ..Default::default()
    }
}

fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let pressed = matches!(status, button::Status::Pressed);
    let disabled = matches!(status, button::Status::Disabled);
    let background = if disabled {
        Color::from_rgba8(225, 91, 100, 0.45)
    } else if pressed {
        Color::from_rgb8(0xC8, 0x45, 0x50)
    } else if hovered {
        Color::from_rgb8(0xEC, 0x71, 0x79)
    } else {
        Color::from_rgb8(0xE1, 0x5B, 0x64)
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: Border { radius: 16.0.into(), width: 1.0, color: Color::TRANSPARENT },
        ..Default::default()
    }
}

fn filter_button_style(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let pressed = matches!(status, button::Status::Pressed);
    let background = if active {
        theme.extended_palette().primary.base.color
    } else if pressed {
        theme.extended_palette().background.strong.color
    } else if hovered {
        theme.extended_palette().background.weak.color
    } else {
        theme.extended_palette().background.base.color
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: if active {
            theme.extended_palette().primary.base.text
        } else {
            theme.palette().text
        },
        border: Border {
            radius: 999.0.into(),
            width: 1.0,
            color: if active {
                theme.extended_palette().primary.base.color
            } else {
                theme.extended_palette().background.strong.color
            },
        },
        ..Default::default()
    }
}
#[cfg(test)]
#[path = "large_file_tool_tests.rs"]
mod large_file_tool_tests;
