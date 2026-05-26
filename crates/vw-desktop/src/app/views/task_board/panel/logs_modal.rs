//! 任务看板侧栏面板模块，负责任务编辑、执行器选择和日志弹窗界面。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::app::components::system_settings_common::{
    settings_close_button, settings_text_input_style,
};
use crate::app::message::TaskBoardMessage;
use crate::app::task::Task;
use crate::app::{App, Message};

use super::super::common::{button_style_danger, button_style_primary, button_style_secondary};
use super::styles::panel_container_style;

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `task`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
#[allow(dead_code)]
pub fn build_task_logs_modal<'a>(app: &'a App, task: &'a Task) -> Element<'a, Message> {
    let close_btn = settings_close_button(Message::TaskBoard(TaskBoardMessage::CloseTaskLogs));

    let header = row![
        text(&task.id).size(11).style(|theme: &Theme| {
            iced::widget::text::Style { color: Some(theme.extended_palette().background.base.text) }
        }),
        Space::new().width(Length::Fill),
        close_btn,
    ]
    .spacing(8)
    .width(Length::Fill);

    let mut main_col = column![header, Space::new().height(12.0)].spacing(0).width(Length::Fill);

    if !task.subtasks.is_empty() {
        main_col = build_subtasks_section(task, main_col);
    }

    main_col = build_add_subtask_section(app, task, main_col);

    main_col = build_logs_section(task, main_col);

    let content = scrollable(main_col).width(Length::Fill).height(Length::Fill);

    container(content)
        .padding(20)
        .style(|theme: &Theme| {
            let mut style = panel_container_style(theme);
            style.shadow = iced::Shadow {
                color: Color::BLACK.scale_alpha(0.2),
                offset: iced::Vector::new(-4.0, 0.0),
                blur_radius: 16.0,
            };
            style
        })
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
}

fn build_subtasks_section<'a>(
    task: &'a Task,
    mut main_col: iced::widget::Column<'a, Message>,
) -> iced::widget::Column<'a, Message> {
    let subtasks_title = text("子任务")
        .size(13)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });
    main_col = main_col.push(subtasks_title).push(Space::new().height(8.0));

    let mut subtasks_col = column![].spacing(4);
    for (idx, subtask) in task.subtasks.iter().enumerate() {
        let created_secs = subtask.created_at_ms / 1000;
        let created_date = format!(
            "{}-{} {}:{}",
            (created_secs / 86400 / 365 + 1970),
            (created_secs / 86400 % 365) / 30 + 1,
            (created_secs % 86400) / 3600,
            (created_secs % 3600) / 60
        );

        let checkbox_text = if subtask.completed {
            format!("✓ {}", subtask.content)
        } else {
            format!("○ {}", subtask.content)
        };

        let toggle_btn = button(text(checkbox_text).size(11))
            .on_press(Message::TaskBoard(TaskBoardMessage::ToggleSubTaskCompleted {
                task_id: task.id.clone(),
                subtask_id: subtask.id.clone(),
            }))
            .padding([4, 8])
            .style(button_style_secondary);

        let mut subtask_row =
            row![toggle_btn, Space::new().width(4.0)].spacing(4).align_y(Alignment::Center);

        if idx > 0 {
            let up_btn: Element<'_, Message> = button(text("↑").size(10))
                .on_press(Message::TaskBoard(TaskBoardMessage::MoveSubTaskUp {
                    task_id: task.id.clone(),
                    subtask_id: subtask.id.clone(),
                }))
                .padding([2, 6])
                .style(button_style_secondary)
                .into();
            subtask_row = subtask_row.push(up_btn);
        } else {
            let up_btn: Element<'_, Message> = button(text("↑").size(10))
                .padding([2, 6])
                .style(|theme: &Theme, _| {
                    let p = theme.extended_palette();
                    iced::widget::button::Style {
                        background: None,
                        border: iced::Border {
                            radius: 4.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        text_color: p.background.weak.text,
                        ..Default::default()
                    }
                })
                .into();
            subtask_row = subtask_row.push(up_btn);
        }

        if idx < task.subtasks.len() - 1 {
            let down_btn: Element<'_, Message> = button(text("↓").size(10))
                .on_press(Message::TaskBoard(TaskBoardMessage::MoveSubTaskDown {
                    task_id: task.id.clone(),
                    subtask_id: subtask.id.clone(),
                }))
                .padding([2, 6])
                .style(button_style_secondary)
                .into();
            subtask_row = subtask_row.push(down_btn);
        } else {
            let down_btn: Element<'_, Message> = button(text("↓").size(10))
                .padding([2, 6])
                .style(|theme: &Theme, _| {
                    let p = theme.extended_palette();
                    iced::widget::button::Style {
                        background: None,
                        border: iced::Border {
                            radius: 4.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        text_color: p.background.weak.text,
                        ..Default::default()
                    }
                })
                .into();
            subtask_row = subtask_row.push(down_btn);
        }

        let delete_btn: Element<'_, Message> = button(text("×").size(10))
            .on_press(Message::TaskBoard(TaskBoardMessage::RemoveSubTask {
                task_id: task.id.clone(),
                subtask_id: subtask.id.clone(),
            }))
            .padding([2, 6])
            .style(button_style_danger)
            .into();

        subtask_row = subtask_row.push(delete_btn).push(Space::new().width(8.0)).push(
            text(created_date).size(9).style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(Color::from_rgb8(156, 163, 175)),
            }),
        );

        subtasks_col = subtasks_col.push(subtask_row);
    }
    main_col.push(subtasks_col).push(Space::new().height(12.0))
}

fn build_add_subtask_section<'a>(
    app: &'a App,
    task: &'a Task,
    main_col: iced::widget::Column<'a, Message>,
) -> iced::widget::Column<'a, Message> {
    let add_subtask_title = text("添加子任务").size(12);
    let add_subtask_input = text_input("输入子任务内容...", &app.task_board_new_subtask_content)
        .on_input(|v| Message::TaskBoard(TaskBoardMessage::UpdateNewSubtaskContent(v)))
        .padding(8)
        .width(Length::Fill)
        .style(settings_text_input_style);

    let content_for_add = app.task_board_new_subtask_content.clone();
    let task_id_for_add = task.id.clone();
    let add_subtask_btn = button(text("添加").size(11))
        .on_press(Message::TaskBoard(TaskBoardMessage::AddSubTask {
            task_id: task_id_for_add,
            content: content_for_add,
        }))
        .padding([8, 12])
        .style(button_style_primary);

    let add_subtask_row = row![add_subtask_input, add_subtask_btn].spacing(8).width(Length::Fill);

    main_col
        .push(add_subtask_title)
        .push(Space::new().height(4.0))
        .push(add_subtask_row)
        .push(Space::new().height(12.0))
}

fn build_logs_section<'a>(
    task: &'a Task,
    mut main_col: iced::widget::Column<'a, Message>,
) -> iced::widget::Column<'a, Message> {
    let logs_title = text("任务日志")
        .size(13)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });
    main_col = main_col.push(logs_title).push(Space::new().height(8.0));

    let logs_content: String = task
        .logs
        .iter()
        .map(|log| {
            let ts_mins = log.timestamp_ms / 60000;
            let ts_secs = (log.timestamp_ms % 60000) / 1000;
            format!("{}:{} - {}", ts_mins, ts_secs, log.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let logs_widget =
        container(text(logs_content).size(11).wrapping(iced::widget::text::Wrapping::Word))
            .padding([8, 10])
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        p.background.weak.color.scale_alpha(0.35),
                    )),
                    border: iced::Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: p.background.strong.color.scale_alpha(0.4),
                    },
                    ..Default::default()
                }
            })
            .width(Length::Fill);
    let scrollable_widget = scrollable(logs_widget).height(Length::Fill);
    main_col.push(scrollable_widget)
}

#[cfg(test)]
#[path = "logs_modal_tests.rs"]
mod logs_modal_tests;
