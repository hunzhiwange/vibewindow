//! 图片导入视图模块，负责渲染图片导入弹窗并把用户输入转换为设计消息。

use iced::widget::{Space, button, column, container, row, svg, text, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::state::{DesignState, ImageImportTarget};
use crate::app::{App, Message};

/// 渲染对应的设计界面片段。
///
/// 返回 Iced 元素；输入为空或不支持时由调用方保留现有界面兜底。
pub fn render_image_import_dialog<'a>(
    _app: &'a App,
    state: &'a DesignState,
) -> Element<'a, Message> {
    let Some(target) = state.image_import_target.as_ref() else {
        return Space::new().into();
    };

    let title = match target {
        ImageImportTarget::Element => "导入图片",
        ImageImportTarget::Fill { .. } => "设置图片填充",
    };
    let hint = match target {
        ImageImportTarget::Element => "支持导入图片或 SVG，并创建新的图片元素",
        ImageImportTarget::Fill { .. } => "支持导入图片或 SVG，并替换当前填充图片",
    };

    let header = row![
        column![
            text(title)
                .size(18)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            text(hint).size(13).style(|theme: &Theme| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                iced::widget::text::Style {
                    color: Some(if is_dark {
                        Color::from_rgba8(190, 194, 201, 1.0)
                    } else {
                        Color::from_rgba8(108, 112, 120, 1.0)
                    }),
                }
            })
        ]
        .spacing(6),
        Space::new().width(Length::Fill),
        button(svg(assets::get_icon(Icon::X)).width(14).height(14))
            .on_press(Message::Design(DesignMessage::CloseImageImportDialog))
            .style(dialog_icon_button_style)
            .padding(6)
    ]
    .align_y(iced::Alignment::Center);

    let input =
        text_input("粘贴 https://、file:///、data:image/... 或 base64", &state.image_import_input)
            .on_input(|value| Message::Design(DesignMessage::ImageImportInputChanged(value)))
            .on_submit(Message::Design(DesignMessage::SubmitImageImport))
            .padding([11, 12])
            .size(13)
            .style(dialog_input_style);

    let error_text: Element<'a, Message> = if let Some(error) = state.image_import_error.as_ref() {
        text(error)
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba8(220, 88, 88, 1.0)),
            })
            .into()
    } else {
        Space::new().height(Length::Fixed(0.0)).into()
    };

    let actions = row![
        button(text("选择文件").size(13))
            .on_press_maybe(
                (!state.image_import_loading)
                    .then_some(Message::Design(DesignMessage::ChooseImageImportFile,))
            )
            .style(dialog_secondary_button_style)
            .padding([10, 14]),
        button(text("粘贴剪贴板").size(13))
            .on_press_maybe(
                (!state.image_import_loading)
                    .then_some(Message::Design(DesignMessage::PasteImageImportInput,))
            )
            .style(dialog_secondary_button_style)
            .padding([10, 14]),
        Space::new().width(Length::Fill),
        button(text(if state.image_import_loading { "导入中..." } else { "确认导入" }).size(13))
            .on_press_maybe(
                (!state.image_import_loading)
                    .then_some(Message::Design(DesignMessage::SubmitImageImport,))
            )
            .style(dialog_primary_button_style)
            .padding([10, 16])
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let panel = container(
        column![
            header,
            text("图片来源")
                .size(13)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            input,
            text("可直接粘贴网络图片 URL、本地路径、file URL、data URL 或 base64 字符串。")
                .size(12)
                .style(|theme: &Theme| {
                    let palette = theme.palette();
                    let is_dark =
                        palette.background.r + palette.background.g + palette.background.b < 1.5;
                    iced::widget::text::Style {
                        color: Some(if is_dark {
                            Color::from_rgba8(156, 160, 168, 1.0)
                        } else {
                            Color::from_rgba8(123, 126, 133, 1.0)
                        }),
                    }
                }),
            error_text,
            actions
        ]
        .spacing(14),
    )
    .padding(20)
    .width(Length::Fixed(560.0))
    .style(dialog_panel_style);

    container(container(panel).center_x(Length::Fill).center_y(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(dialog_backdrop_style)
        .into()
}

fn dialog_backdrop_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    container::Style {
        background: Some(
            if is_dark {
                Color::from_rgba8(0, 0, 0, 0.42)
            } else {
                Color::from_rgba8(17, 24, 39, 0.18)
            }
            .into(),
        ),
        ..Default::default()
    }
}

fn dialog_panel_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    container::Style {
        background: Some(
            if is_dark {
                Color::from_rgba8(36, 38, 42, 0.985)
            } else {
                Color::from_rgba8(248, 248, 249, 0.995)
            }
            .into(),
        ),
        text_color: Some(if is_dark {
            Color::from_rgba8(246, 247, 249, 1.0)
        } else {
            Color::from_rgba8(28, 29, 31, 1.0)
        }),
        border: Border {
            width: 1.0,
            color: if is_dark {
                Color::from_rgba8(255, 255, 255, 0.10)
            } else {
                Color::from_rgba8(0, 0, 0, 0.08)
            },
            radius: 14.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(if is_dark { 0.28 } else { 0.15 }),
            offset: iced::Vector::new(0.0, 16.0),
            blur_radius: 42.0,
        },
        ..Default::default()
    }
}

fn dialog_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    let border_color = match status {
        text_input::Status::Focused { .. } => Color::from_rgba8(59, 130, 246, 0.9),
        _ if is_dark => Color::from_rgba8(255, 255, 255, 0.10),
        _ => Color::from_rgba8(0, 0, 0, 0.08),
    };
    text_input::Style {
        background: Background::Color(if is_dark {
            Color::from_rgba8(28, 30, 34, 1.0)
        } else {
            Color::WHITE
        }),
        border: Border { width: 1.0, color: border_color, radius: 10.0.into() },
        icon: palette.text,
        placeholder: if is_dark {
            Color::from_rgba8(137, 141, 149, 1.0)
        } else {
            Color::from_rgba8(152, 156, 164, 1.0)
        },
        value: palette.text,
        selection: Color::from_rgba8(59, 130, 246, 0.28),
    }
}

fn dialog_icon_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered => Some(Color::from_rgba8(127, 127, 132, 0.12).into()),
            button::Status::Pressed => Some(Color::from_rgba8(127, 127, 132, 0.18).into()),
            _ => None,
        },
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
        ..Default::default()
    }
}

fn dialog_secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    button::Style {
        background: Some(
            match status {
                button::Status::Hovered => {
                    if is_dark {
                        Color::from_rgba8(71, 74, 80, 1.0)
                    } else {
                        Color::from_rgba8(240, 241, 243, 1.0)
                    }
                }
                button::Status::Pressed => {
                    if is_dark {
                        Color::from_rgba8(78, 82, 88, 1.0)
                    } else {
                        Color::from_rgba8(232, 234, 237, 1.0)
                    }
                }
                _ => {
                    if is_dark {
                        Color::from_rgba8(60, 63, 68, 1.0)
                    } else {
                        Color::from_rgba8(244, 245, 246, 1.0)
                    }
                }
            }
            .into(),
        ),
        text_color: palette.text,
        border: Border {
            width: 1.0,
            color: if is_dark {
                Color::from_rgba8(255, 255, 255, 0.08)
            } else {
                Color::from_rgba8(0, 0, 0, 0.06)
            },
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

fn dialog_primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgba8(28, 100, 242, 1.0),
        button::Status::Pressed => Color::from_rgba8(24, 89, 216, 1.0),
        _ => Color::from_rgba8(37, 99, 235, 1.0),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: Color::WHITE,
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "image_import_tests.rs"]
mod image_import_tests;
