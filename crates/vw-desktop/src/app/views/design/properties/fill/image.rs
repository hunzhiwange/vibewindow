//! 图片填充属性模块，负责渲染图片填充控制项并发送元素属性更新。

use iced::widget::{Space, button, column, container, pick_list, text, text_input};
use iced::{Background, Border, Color, Element, Length};
use std::fmt;

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::properties::fill::types::{FillItem, FillObject, ImageFill};

use crate::app::views::design::properties::utils::{prop_section, prop_text_input_style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageModeOption {
    label: &'static str,
    value: &'static str,
}

impl fmt::Display for ImageModeOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

/// 渲染对应的设计界面片段。
///
/// 返回 Iced 元素；输入为空或不支持时由调用方保留现有界面兜底。
pub fn render(
    image: ImageFill,
    index: usize,
    fills: Vec<FillItem>,
    id: String,
) -> Element<'static, Message> {
    let mode_options = vec![
        ImageModeOption { label: "铺满宽度", value: "fill_width" },
        ImageModeOption { label: "适应", value: "fit" },
        ImageModeOption { label: "填充", value: "fill" },
        ImageModeOption { label: "拉伸", value: "stretch" },
    ];

    let selected_mode = mode_options.iter().copied().find(|o| o.value == image.mode);

    let url_label =
        if image.url.trim().is_empty() { "未选择图片".to_string() } else { image.url.clone() };

    let import_card = container(
        column![
            button(text("导入或粘贴图片").size(12))
                .on_press(Message::Design(DesignMessage::ImportFillImage(id.clone(), index,)))
                .style(button::secondary)
                .width(Length::Fill),
            Space::new().height(Length::Fixed(2.0)),
            text(url_label).size(11)
        ]
        .width(Length::Fill)
        .spacing(6),
    )
    .padding(10)
    .width(Length::Fill)
    .style(|theme: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgb8(0xF2, 0xF2, 0xF2))),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: theme.extended_palette().background.strong.color,
        },
        ..Default::default()
    });

    column![
        container(prop_section(
            "模式",
            pick_list(mode_options, selected_mode, {
                let id = id.clone();
                let fills = fills.clone();
                move |o| update_image_mode(id.clone(), fills.clone(), index, o.value.to_string())
            })
            .width(Length::Fill),
        ))
        .width(Length::Fill),
        prop_section("导入", import_card),
        prop_section(
            "图片路径或 URL",
            text_input("file:///... 或 https://...", &image.url)
                .on_input({
                    let id = id.clone();
                    let fills = fills.clone();
                    move |s| update_image_url(id.clone(), fills.clone(), index, s)
                })
                .style(prop_text_input_style)
        )
    ]
    .spacing(10)
    .into()
}

fn update_image_mode(id: String, fills: Vec<FillItem>, index: usize, mode: String) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Image(img))) = new_fills.get_mut(index) {
        img.mode = mode;
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

fn update_image_url(id: String, fills: Vec<FillItem>, index: usize, url: String) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Image(img))) = new_fills.get_mut(index) {
        img.url = url;
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod image_tests;
