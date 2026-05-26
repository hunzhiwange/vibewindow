//! 文件视图中的列表项与按钮构建。

use std::collections::HashMap;
use std::path::Path;

use iced::widget::{Space, button, column, container, mouse_area, row, text};
use iced::{Alignment, Background, Border, Element, Length, Theme};

use crate::app::assets::Icon;
use crate::app::components::chat_panel::utils::{
    change_pills, icon_svg, relative_to_project_root, resolve_path, truncate_chars,
    weak_file_button_style,
};
use crate::app::{App, Message, message};

use super::super::diff_utils::file_preview;
use super::super::tool_parse::tool_input_path;
use super::super::types::ChangeFile;

pub(crate) fn build_file_list_column<'a>(
    app: &'a App,
    render_state: &super::FileListState,
    view_ctx: &super::FilesViewContext<'a>,
    changes_by_path: &HashMap<String, ChangeFile>,
) -> iced::widget::Column<'a, Message> {
    let mut column_view = column![].spacing(6);

    for (index, (display, abs)) in render_state.items_for_display.iter().enumerate() {
        if render_state.truncated_middle && index == (render_state.max_items / 2) {
            column_view = column_view.push(
                container(text("…").size(14).style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().secondary.base.text.scale_alpha(0.9)),
                }))
                .padding([2, 6])
                .width(Length::Fill),
            );
        }

        let open_content: Element<'a, Message> = if render_state.is_search {
            let label = truncate_chars(display, 80);
            row![text(label).size(14).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().secondary.base.text.scale_alpha(0.85)),
            })]
            .into()
        } else {
            let file_name = file_row_label(app, view_ctx, display, abs);
            let light = matches!(view_ctx.verb, "读取" | "编辑");
            tool_file_left(view_ctx.verb, file_name, light, view_ctx.read_range.clone())
        };

        let open_button = button(open_content)
            .padding([6, 10])
            .width(Length::Fill)
            .style(weak_file_button_style)
            .on_press(Message::Preview(message::PreviewMessage::Open(abs.clone())));

        if view_ctx.is_edit_like && !render_state.is_search
            && let Some(rel) = relative_to_project_root(app, abs) {
                if let Some(change) = changes_by_path.get(rel.as_str()) {
                    let preview_abs = Some(abs.as_str());
                    let meta_pill: Element<'a, Message> =
                        change_pills(change.additions, change.deletions);
                    let title = format!("{}  +{}-{}", rel, change.additions, change.deletions);
                    let file_name = file_row_label(app, view_ctx, display, abs);
                    let light = matches!(view_ctx.verb, "读取" | "编辑");
                    let left =
                        tool_file_left(view_ctx.verb, file_name, light, view_ctx.read_range.clone());

                    let diff_row = row![
                        left,
                        preview_eye_button_slot(
                            app,
                            view_ctx.msg_idx,
                            view_ctx.tool_idx,
                            preview_abs,
                        ),
                        container(Space::new()).width(Length::Fill),
                        meta_pill,
                        view_changes_pill()
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center);

                    let diff_button: Element<'a, Message> = button(diff_row)
                        .padding([6, 10])
                        .width(Length::Fill)
                        .style(weak_file_button_style)
                        .on_press(Message::Git(message::GitMessage::OpenChatTextDiff {
                            title,
                            file: rel.clone(),
                            before: change.before.clone(),
                            after: change.after.clone(),
                        }))
                        .into();
                    let diff_button = wrap_tool_file_hover(
                        view_ctx.msg_idx,
                        view_ctx.tool_idx,
                        preview_abs,
                        diff_button,
                    );
                    column_view = column_view.push(diff_button);
                    continue;
                }
            }

        if let Some((title, file, after)) = fallback_edit_diff_payload(app, view_ctx, display, abs) {
            let preview_abs = Some(abs.as_str());
            let light = matches!(view_ctx.verb, "读取" | "编辑");
            let left = tool_file_left(
                view_ctx.verb,
                file_row_label(app, view_ctx, display, abs),
                light,
                view_ctx.read_range.clone(),
            );

            let diff_row = row![
                left,
                preview_eye_button_slot(
                    app,
                    view_ctx.msg_idx,
                    view_ctx.tool_idx,
                    preview_abs,
                ),
                container(Space::new()).width(Length::Fill),
                view_changes_pill()
            ]
            .spacing(10)
            .align_y(Alignment::Center);

            let diff_button: Element<'a, Message> = button(diff_row)
                .padding([6, 10])
                .width(Length::Fill)
                .style(weak_file_button_style)
                .on_press(Message::Git(message::GitMessage::OpenChatTextDiff {
                    title,
                    file,
                    before: String::new(),
                    after,
                }))
                .into();
            let diff_button = wrap_tool_file_hover(
                view_ctx.msg_idx,
                view_ctx.tool_idx,
                preview_abs,
                diff_button,
            );
            column_view = column_view.push(diff_button);
            continue;
        }

        column_view = column_view.push(open_button);
    }

    if render_state.tail_omitted > 0 {
        column_view = column_view.push(
            container(
                text(format!("… 省略{}条", render_state.tail_omitted))
                    .size(14)
                    .style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().secondary.base.text.scale_alpha(0.8)),
                    }),
            )
            .padding([2, 6])
            .width(Length::Fill),
        );
    }

    if !render_state.filter_query.is_empty() && render_state.is_empty_filtered {
        column_view = column_view.push(
            container(text("没有匹配的文件").size(14).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().secondary.base.text.scale_alpha(0.65)),
                }
            }))
            .padding([2, 6])
            .width(Length::Fill),
        );
    } else if render_state.total_items == 0 {
        column_view = column_view.push(
            container(
                text(if render_state.is_search {
                    "未返回可定位的文件结果"
                } else {
                    "没有可展示的文件"
                })
                .size(14)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().secondary.base.text.scale_alpha(0.65)),
                }),
            )
            .padding([2, 6])
            .width(Length::Fill),
        );
    }

    column_view
}

pub(super) fn file_row_label(
    app: &App,
    view_ctx: &super::FilesViewContext<'_>,
    display: &str,
    abs: &str,
) -> String {
    if view_ctx.is_edit_like {
        return relative_to_project_root(app, abs).unwrap_or_else(|| display.to_string());
    }

    Path::new(display)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(display)
        .to_string()
}

pub(super) fn fallback_edit_diff_payload(
    app: &App,
    view_ctx: &super::FilesViewContext<'_>,
    display: &str,
    abs: &str,
) -> Option<(String, String, String)> {
    if !view_ctx.is_edit_like {
        return None;
    }

    let input_path = tool_input_path(&view_ctx.input).and_then(|path| resolve_path(app, &path))?;
    if input_path != abs {
        return None;
    }

    let after = file_preview(&view_ctx.tool_name, &view_ctx.input, &view_ctx.output)?;
    let after = after.trim().to_string();
    if after.is_empty() {
        return None;
    }

    let file = relative_to_project_root(app, abs).unwrap_or_else(|| display.to_string());
    let title = format!("{}  写入内容", file);
    Some((title, file, after))
}

fn view_changes_pill<'a>() -> Element<'a, Message> {
    container(text("查看变更").size(14).style(|theme: &Theme| {
        let is_dark =
            theme.palette().background.r + theme.palette().background.g + theme.palette().background.b
                < 1.5;
        iced::widget::text::Style {
            color: Some(if is_dark {
                theme.palette().text.scale_alpha(0.88)
            } else {
                theme.extended_palette().secondary.base.text
            }),
        }
    }))
    .padding([2, 8])
    .style(|theme: &Theme| {
        let ext = theme.extended_palette();
        let is_dark =
            theme.palette().background.r + theme.palette().background.g + theme.palette().background.b
                < 1.5;
        iced::widget::container::Style {
            background: Some(Background::Color(if is_dark {
                ext.background.weak.color.scale_alpha(0.24)
            } else {
                ext.background.base.color.scale_alpha(0.76)
            })),
            border: Border {
                width: 1.0,
                color: ext.background.strong.color.scale_alpha(if is_dark { 0.48 } else { 0.75 }),
                radius: 999.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn preview_eye_button<'a>(abs: String) -> Element<'a, Message> {
    let eye = icon_svg(Icon::Eye).width(Length::Fixed(12.0)).height(Length::Fixed(12.0)).style(
        |theme: &Theme, _status| {
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;
            iced::widget::svg::Style {
                color: Some(if is_dark {
                    theme.palette().text.scale_alpha(0.9)
                } else {
                    theme.extended_palette().secondary.base.text
                }),
            }
        },
    );

    mouse_area(container(eye).padding([4, 6]).style(|theme: &Theme| {
        let ext = theme.extended_palette();
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;
        iced::widget::container::Style {
            background: Some(Background::Color(if is_dark {
                ext.background.weak.color.scale_alpha(0.32)
            } else {
                ext.background.base.color.scale_alpha(0.95)
            })),
            border: Border {
                width: 1.0,
                color: ext.background.strong.color.scale_alpha(if is_dark { 0.5 } else { 0.75 }),
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }))
    .on_press(Message::Preview(message::PreviewMessage::Open(abs)))
    .into()
}

fn preview_eye_button_slot<'a>(
    app: &App,
    msg_idx: usize,
    tool_idx: usize,
    abs: Option<&str>,
) -> Element<'a, Message> {
    let Some(abs) = abs else {
        return Space::new().into();
    };

    let hover_key = tool_file_hover_key(msg_idx, tool_idx, abs);
    if app.chat_tool_file_hovered.as_deref() == Some(hover_key.as_str()) {
        preview_eye_button(abs.to_string())
    } else {
        Space::new().width(Length::Fixed(24.0)).into()
    }
}

fn wrap_tool_file_hover<'a>(
    msg_idx: usize,
    tool_idx: usize,
    abs: Option<&str>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let Some(abs) = abs else {
        return content;
    };

    mouse_area(content)
        .on_enter(Message::Chat(message::ChatMessage::ToolFileHover(
            tool_file_hover_key(msg_idx, tool_idx, abs),
        )))
        .on_exit(Message::Chat(message::ChatMessage::ToolFileHoverLeave))
        .into()
}

pub(super) fn tool_file_hover_key(msg_idx: usize, tool_idx: usize, abs: &str) -> String {
    format!("{msg_idx}:{tool_idx}:{abs}")
}

fn tool_file_left<'a>(
    verb: &'static str,
    file_name: String,
    light: bool,
    read_range: Option<String>,
) -> Element<'a, Message> {
    let verb_text = text(verb).size(14).style(move |theme: &Theme| {
        let is_dark =
            theme.palette().background.r + theme.palette().background.g + theme.palette().background.b
                < 1.5;
        iced::widget::text::Style {
            color: Some(if light {
                if is_dark {
                    theme.palette().text.scale_alpha(0.76)
                } else {
                    theme.extended_palette().secondary.base.text.scale_alpha(0.65)
                }
            } else {
                theme.extended_palette().secondary.base.text
            }),
        }
    });

    let file_text = text(file_name).size(14).style(move |theme: &Theme| {
        let is_dark =
            theme.palette().background.r + theme.palette().background.g + theme.palette().background.b
                < 1.5;
        iced::widget::text::Style {
            color: Some(if light {
                if is_dark {
                    theme.palette().text.scale_alpha(0.9)
                } else {
                    theme.extended_palette().secondary.base.text.scale_alpha(0.65)
                }
            } else {
                theme.palette().text
            }),
        }
    });

    let mut row_view = row![verb_text, file_text].spacing(8).align_y(Alignment::Center);

    if let Some(range) = read_range {
        row_view = row_view.push(container(Space::new()).width(Length::Fill)).push(
            text(range).size(14).style(|theme: &Theme| {
                let is_dark = theme.palette().background.r
                    + theme.palette().background.g
                    + theme.palette().background.b
                    < 1.5;
                iced::widget::text::Style {
                    color: Some(if is_dark {
                        theme.palette().text.scale_alpha(0.72)
                    } else {
                        theme.extended_palette().secondary.base.text.scale_alpha(0.65)
                    }),
                }
            }),
        );
    }

    row_view.into()
}
