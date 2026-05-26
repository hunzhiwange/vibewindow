//! 图标属性面板模块，负责图标库、图标名称和粗细选项的展示与更新。

use super::utils::prop_section;
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::models::DesignElement;
use iced::widget::{button, column, container, pick_list, row, svg, text};
use iced::{Element, Length, Point, Theme};
use std::fmt;

#[derive(Debug, Clone)]
/// ActiveIconPicker 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub struct ActiveIconPicker {
    pub element_id: String,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IconFamilyOption {
    id: String,
    label: String,
}

impl fmt::Display for IconFamilyOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

fn icon_family_options() -> Vec<IconFamilyOption> {
    assets::named_icon_catalog()
        .iter()
        .map(|entry| IconFamilyOption {
            id: entry.family.clone(),
            label: assets::named_icon_family_label(&entry.family),
        })
        .collect()
}

/// 处理图标属性的展示、选项或值转换，保持图标面板和设计元素字段一致。
pub(crate) fn icon_display_name(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 处理图标属性的展示、选项或值转换，保持图标面板和设计元素字段一致。
pub(crate) fn icon_weight_options_for_family(family: &str) -> Vec<String> {
    match assets::canonical_named_icon_family(family).as_deref() {
        Some("phosphor") => {
            vec!["Thin", "Light", "Regular", "Bold"].into_iter().map(str::to_string).collect()
        }
        _ => Vec::new(),
    }
}

/// 处理图标属性的展示、选项或值转换，保持图标面板和设计元素字段一致。
pub(crate) fn icon_weight_label(value: &Option<serde_json::Value>) -> String {
    let weight = value
        .as_ref()
        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok())))
        .unwrap_or(400);
    match weight {
        ..=200 => "Thin".to_string(),
        201..=350 => "Light".to_string(),
        351..=599 => "Regular".to_string(),
        _ => "Bold".to_string(),
    }
}

/// 处理图标属性的展示、选项或值转换，保持图标面板和设计元素字段一致。
pub(crate) fn icon_weight_value_from_label(label: &str) -> serde_json::Value {
    let value = match label {
        "Thin" => 100,
        "Light" => 300,
        "Bold" => 700,
        _ => 400,
    };
    serde_json::json!(value)
}

fn picker_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let ext = theme.extended_palette();
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let background = if hovered { ext.background.weak.color } else { ext.background.base.color };
    button::Style {
        background: Some(background.into()),
        text_color: theme.palette().text,
        border: iced::Border { color: ext.background.strong.color, width: 1.0, radius: 8.0.into() },
        ..button::Style::default()
    }
}

fn selector_button<'a>(label: String, on_press: Message) -> Element<'a, Message> {
    button(
        row![
            text(label).size(12).width(Length::Fill),
            container(svg(assets::get_icon(Icon::ChevronDown)).width(14).height(14))
                .width(Length::Fixed(18.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        ]
        .align_y(iced::Alignment::Center)
        .spacing(6),
    )
    .padding([7, 10])
    .width(Length::Fill)
    .style(picker_button_style)
    .on_press(on_press)
    .into()
}

/// 渲染对应的设计界面片段。
///
/// 返回 Iced 元素；输入为空或不支持时由调用方保留现有界面兜底。
pub fn render<'a>(element: &'a DesignElement) -> Element<'a, Message> {
    let id = element.id.clone();
    let family = element.icon_font_family.clone().unwrap_or_else(|| "lucide".to_string());
    let icon_name = element.icon_font_name.clone().unwrap_or_else(|| "star".to_string());
    let icon_label = if icon_name.trim().is_empty() {
        "选择图标".to_string()
    } else {
        icon_display_name(&icon_name)
    };
    let family_options = icon_family_options();
    let selected_family = family_options.iter().find(|option| option.id == family).cloned();
    let weight_options = icon_weight_options_for_family(&family);
    let current_weight_label = icon_weight_label(&element.weight);
    let selected_weight = if weight_options.contains(&current_weight_label) {
        Some(current_weight_label.clone())
    } else {
        weight_options.first().cloned()
    };

    let mut content = column![
        text("图标")
            .size(12)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        prop_section(
            "图标名称",
            selector_button(
                icon_label,
                Message::Design(DesignMessage::OpenIconPicker(id.clone(), None))
            )
        ),
        prop_section(
            "图标库",
            pick_list(family_options, selected_family, {
                let id = id.clone();
                move |option| {
                    Message::Design(DesignMessage::IconFamilySelected {
                        element_id: id.clone(),
                        family: option.id,
                    })
                }
            })
            .text_size(12)
            .padding(6)
            .width(Length::Fill)
        )
    ]
    .spacing(10);

    if !weight_options.is_empty() {
        content = content.push(prop_section(
            "粗细",
            pick_list(weight_options, selected_weight, {
                let id = id.clone();
                move |label| {
                    Message::Design(DesignMessage::PropertyUpdate(
                        id.clone(),
                        "weight".to_string(),
                        icon_weight_value_from_label(&label),
                    ))
                }
            })
            .text_size(12)
            .padding(6)
            .width(Length::Fill),
        ));
    }

    container(content).width(Length::Fill).into()
}

#[cfg(test)]
#[path = "icon_tests.rs"]
mod icon_tests;
