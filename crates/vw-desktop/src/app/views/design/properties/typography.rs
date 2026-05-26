//! 设计器属性面板的局部渲染模块，负责把元素布局或文字状态转换为可编辑控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use super::number_input::NumberInput;
use super::utils::{prop_section, prop_text_input_style};
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::parse_font_size;
use crate::app::views::design::models::{DesignElement, VariableDef};
use iced::widget::{button, column, container, pick_list, row, svg, text, text_input};
use iced::{Color, Element, Length, Point, Theme};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// ActiveFontPicker 保存本视图片段需要跨控件传递的轻量状态。
#[derive(Debug, Clone)]
pub struct ActiveFontPicker {
    pub element_id: String,
    pub position: Point,
}

/// 渲染对应界面。
///
/// # 参数
/// - `element`: 当前视图构建所需的状态、配置或消息。
/// - `variables`: 当前视图构建所需的状态、配置或消息。
/// - `theme_mode`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render<'a>(
    element: &'a DesignElement,
    variables: &'a HashMap<String, VariableDef>,
    theme_mode: Option<&'a str>,
) -> Element<'a, Message> {
    let id = element.id.clone();
    let font_size = parse_font_size(&element.font_size, variables, theme_mode).to_string();
    let initial_font_size: f32 = font_size.parse::<f32>().unwrap_or(16.0);
    let font_family = element.font_family.clone().unwrap_or_default();
    let font_weight_label = font_weight_label(&element.font_weight);
    let line_height = raw_value_to_string(&element.line_height);
    let letter_spacing = raw_value_to_string(&element.letter_spacing);
    let text_align = element.text_align.clone().unwrap_or_else(|| "left".to_string());
    let text_align_vertical =
        element.text_align_vertical.clone().unwrap_or_else(|| "top".to_string());
    let font_style = element.font_style.clone().unwrap_or_else(|| "normal".to_string());
    let text_decoration = element.text_decoration.clone().unwrap_or_else(|| "none".to_string());

    let mut font_weight_options = available_weights_for_font(&font_family);
    if font_weight_options.is_empty() {
        font_weight_options = vec![
            "Light".to_string(),
            "Regular".to_string(),
            "Medium".to_string(),
            "Semi Bold".to_string(),
            "Bold".to_string(),
            "Extra Bold".to_string(),
        ];
    }
    let selected_weight_label = if font_weight_options.contains(&font_weight_label) {
        Some(font_weight_label.clone())
    } else if !font_weight_options.is_empty() {
        Some(font_weight_options[0].clone())
    } else {
        None
    };

    column![
        text("版式设计")
            .size(12)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        prop_section(
            "字体",
            column![
                row![
                    text_input("输入字体名称...", &font_family)
                        .on_input({
                            let id = id.clone();
                            move |s| {
                                Message::Design(DesignMessage::PropertyUpdate(
                                    id.clone(),
                                    "fontFamily".to_string(),
                                    serde_json::Value::String(s),
                                ))
                            }
                        })
                        .style(prop_text_input_style)
                        .padding(4)
                        .size(12)
                        .width(Length::Fill),
                    button(
                        container(svg(assets::get_icon(Icon::ChevronDown)).width(14).height(14))
                            .width(Length::Fixed(28.0))
                            .height(Length::Fixed(28.0))
                            .center_x(Length::Fill)
                            .center_y(Length::Fill),
                    )
                    .on_press(Message::Design(DesignMessage::OpenFontPicker(id.clone(), None)))
                    .padding(0)
                    .style(|theme: &Theme, status| {
                        let ext = theme.extended_palette();
                        let background = if status == button::Status::Hovered {
                            ext.background.weak.color
                        } else {
                            ext.background.base.color
                        };
                        button::Style {
                            background: Some(background.into()),
                            border: iced::Border {
                                color: ext.background.strong.color,
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            ..button::Style::default()
                        }
                    })
                ]
                .spacing(6)
            ]
            .spacing(4)
        ),
        prop_section("样式", {
            let id_bold = id.clone();
            let id_italic = id.clone();
            let id_underline = id.clone();
            let id_strike = id.clone();

            let is_bold = ["Semi Bold", "Bold", "Extra Bold"].contains(&font_weight_label.as_str());
            let is_italic = font_style == "italic";
            let is_underline = text_decoration.contains("underline");
            let is_strike = text_decoration.contains("line-through");

            let current_dec = text_decoration.clone();

            let btn = |icon: Icon, selected: bool, on_press: Message| {
                let icon = svg(assets::get_icon(icon)).width(14).height(14).style(
                    move |theme: &Theme, _| iced::widget::svg::Style {
                        color: Some(if selected { Color::WHITE } else { theme.palette().text }),
                    },
                );

                let content = container(icon)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill);

                button(content)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(32.0))
                    .padding([0, 10])
                    .style(move |theme: &Theme, status| {
                        let p = theme.palette();
                        let ext = theme.extended_palette();
                        let background = if selected {
                            Some(p.primary.into())
                        } else if status == button::Status::Hovered {
                            Some(ext.background.strong.color.into())
                        } else {
                            Some(ext.background.weak.color.into())
                        };

                        button::Style {
                            background,
                            text_color: if selected { Color::WHITE } else { p.text },
                            border: iced::Border {
                                radius: 8.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            ..button::Style::default()
                        }
                    })
                    .on_press(on_press)
            };

            row![
                btn(Icon::TypeBold, is_bold, {
                    let new_weight = if is_bold { "400" } else { "700" };
                    Message::Design(DesignMessage::PropertyUpdate(
                        id_bold,
                        "fontWeight".to_string(),
                        serde_json::Value::String(new_weight.to_string()),
                    ))
                }),
                btn(Icon::TypeItalic, is_italic, {
                    let new_style = if is_italic { "normal" } else { "italic" };
                    Message::Design(DesignMessage::PropertyUpdate(
                        id_italic,
                        "fontStyle".to_string(),
                        serde_json::Value::String(new_style.to_string()),
                    ))
                }),
                btn(Icon::TypeUnderline, is_underline, {
                    let mut parts: Vec<&str> = current_dec.split_whitespace().collect();
                    if is_underline {
                        parts.retain(|&x| x != "underline");
                    } else {
                        parts.push("underline");
                    }
                    parts.retain(|&x| x != "none");
                    let new_dec =
                        if parts.is_empty() { "none".to_string() } else { parts.join(" ") };
                    Message::Design(DesignMessage::PropertyUpdate(
                        id_underline,
                        "textDecoration".to_string(),
                        serde_json::Value::String(new_dec),
                    ))
                }),
                btn(Icon::TypeStrikethrough, is_strike, {
                    let mut parts: Vec<&str> = current_dec.split_whitespace().collect();
                    if is_strike {
                        parts.retain(|&x| x != "line-through");
                    } else {
                        parts.push("line-through");
                    }
                    parts.retain(|&x| x != "none");
                    let new_dec =
                        if parts.is_empty() { "none".to_string() } else { parts.join(" ") };
                    Message::Design(DesignMessage::PropertyUpdate(
                        id_strike,
                        "textDecoration".to_string(),
                        serde_json::Value::String(new_dec),
                    ))
                }),
            ]
            .spacing(8)
            .width(Length::Fill)
        }),
        row![
            container(prop_section(
                "字体粗细",
                pick_list(font_weight_options.clone(), selected_weight_label, {
                    let id = id.clone();
                    move |s| {
                        let value = font_weight_value_from_label(&s);
                        Message::Design(DesignMessage::PropertyUpdate(
                            id.clone(),
                            "fontWeight".to_string(),
                            serde_json::Value::String(value),
                        ))
                    }
                })
                .text_size(12)
                .padding(6)
                .width(Length::Fill)
            ))
            .width(Length::FillPortion(1)),
            container(prop_section(
                "字体大小",
                NumberInput::new(initial_font_size, 1.0, 512.0, 1.0, 0, 0.1, {
                    let id = id.clone();
                    move |v| {
                        Message::Design(DesignMessage::PropertyUpdate(
                            id.clone(),
                            "fontSize".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(v as f64).unwrap(),
                            ),
                        ))
                    }
                })
            ))
            .width(Length::FillPortion(1))
        ]
        .spacing(10)
        .width(Length::Fill),
        row![
            container(prop_section(
                "行高",
                text_input("1.2", &line_height)
                    .on_input({
                        let id = id.clone();
                        move |s| {
                            Message::Design(DesignMessage::PropertyUpdate(
                                id.clone(),
                                "lineHeight".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(s.parse().unwrap_or(1.2)).unwrap(),
                                ),
                            ))
                        }
                    })
                    .style(prop_text_input_style)
                    .padding(6)
                    .size(12)
                    .width(Length::Fill)
            ))
            .width(Length::FillPortion(1)),
            container(prop_section(
                "字母间距",
                text_input("0", &letter_spacing)
                    .on_input({
                        let id = id.clone();
                        move |s| {
                            Message::Design(DesignMessage::PropertyUpdate(
                                id.clone(),
                                "letterSpacing".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(s.parse().unwrap_or(0.0)).unwrap(),
                                ),
                            ))
                        }
                    })
                    .style(prop_text_input_style)
                    .padding(6)
                    .size(12)
                    .width(Length::Fill)
            ))
            .width(Length::FillPortion(1))
        ]
        .spacing(10)
        .width(Length::Fill),
        {
            let id_h = id.clone();
            let id_v = id.clone();
            let segment_btn = |icon: Icon, selected: bool, on_press: Message| {
                let icon = svg(assets::get_icon(icon)).width(16).height(16).style(
                    move |theme: &Theme, _| iced::widget::svg::Style {
                        color: Some(if selected { Color::WHITE } else { theme.palette().text }),
                    },
                );
                button(icon)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(28.0))
                    .padding([1, 0])
                    .style(move |theme: &Theme, status| {
                        let p = theme.palette();
                        let ext = theme.extended_palette();
                        let background = if selected {
                            Some(p.primary.into())
                        } else if status == button::Status::Hovered {
                            Some(ext.background.strong.color.into())
                        } else {
                            None
                        };

                        button::Style {
                            background,
                            text_color: if selected { Color::WHITE } else { p.text },
                            border: iced::Border {
                                radius: 8.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            ..button::Style::default()
                        }
                    })
                    .on_press(on_press)
            };
            let group_container = |content: Element<'a, Message>| {
                container(content).width(Length::Fill).padding(1).style(|theme: &Theme| {
                    let ext = theme.extended_palette();
                    container::Style {
                        background: Some(ext.background.weak.color.into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: ext.background.strong.color,
                        },
                        ..Default::default()
                    }
                })
            };

            let horizontal_group = column![
                text("水平").size(11).style(text::secondary),
                group_container(
                    row![
                        segment_btn(
                            Icon::TextLeft,
                            text_align == "left",
                            Message::Design(DesignMessage::PropertyUpdate(
                                id_h.clone(),
                                "textAlign".to_string(),
                                serde_json::Value::String("left".to_string()),
                            ))
                        ),
                        segment_btn(
                            Icon::TextCenter,
                            text_align == "center",
                            Message::Design(DesignMessage::PropertyUpdate(
                                id_h.clone(),
                                "textAlign".to_string(),
                                serde_json::Value::String("center".to_string()),
                            ))
                        ),
                        segment_btn(
                            Icon::TextRight,
                            text_align == "right",
                            Message::Design(DesignMessage::PropertyUpdate(
                                id_h.clone(),
                                "textAlign".to_string(),
                                serde_json::Value::String("right".to_string()),
                            ))
                        )
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .into()
                )
            ]
            .spacing(6)
            .width(Length::FillPortion(1));

            let vertical_group = column![
                text("垂直").size(11).style(text::secondary),
                group_container(
                    row![
                        segment_btn(
                            Icon::AlignTop,
                            text_align_vertical == "top",
                            Message::Design(DesignMessage::PropertyUpdate(
                                id_v.clone(),
                                "textAlignVertical".to_string(),
                                serde_json::Value::String("top".to_string()),
                            ))
                        ),
                        segment_btn(
                            Icon::AlignMiddle,
                            text_align_vertical == "center",
                            Message::Design(DesignMessage::PropertyUpdate(
                                id_v.clone(),
                                "textAlignVertical".to_string(),
                                serde_json::Value::String("center".to_string()),
                            ))
                        ),
                        segment_btn(
                            Icon::AlignBottom,
                            text_align_vertical == "bottom",
                            Message::Design(DesignMessage::PropertyUpdate(
                                id_v.clone(),
                                "textAlignVertical".to_string(),
                                serde_json::Value::String("bottom".to_string()),
                            ))
                        )
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .into()
                )
            ]
            .spacing(6)
            .width(Length::FillPortion(1));

            row![horizontal_group, vertical_group]
                .spacing(10)
                .align_y(iced::Alignment::Center)
                .width(Length::Fill)
        },
    ]
    .spacing(10)
    .into()
}

fn raw_value_to_string(v: &Option<serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        Some(serde_json::Value::Array(arr)) => serde_json::to_string(arr).unwrap_or_default(),
        Some(serde_json::Value::Object(map)) => serde_json::to_string(map).unwrap_or_default(),
        _ => String::new(),
    }
}

/// 处理字体相关状态。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
#[cfg(target_arch = "wasm32")]
pub(crate) fn available_system_fonts() -> Vec<String> {
    let mut fonts = vec![
        "Arial".to_string(),
        "Helvetica".to_string(),
        "Times New Roman".to_string(),
        "Courier New".to_string(),
        "Georgia".to_string(),
        "Tahoma".to_string(),
        "Trebuchet MS".to_string(),
        "Verdana".to_string(),
        "Roboto".to_string(),
        "Inter".to_string(),
        "Noto Sans".to_string(),
        "Noto Serif".to_string(),
        "Noto Sans CJK SC".to_string(),
        "Microsoft YaHei".to_string(),
        "PingFang SC".to_string(),
        "SF Pro Text".to_string(),
        "Source Han Sans SC".to_string(),
    ];
    fonts.sort();
    fonts.dedup();
    fonts
}

/// 处理字体相关状态。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn available_system_fonts() -> Vec<String> {
    let mut dirs: Vec<&'static str> = Vec::new();
    #[cfg(target_os = "macos")]
    {
        dirs.push("/System/Library/Fonts");
        dirs.push("/Library/Fonts");
        if let Some(home) = std::env::var_os("HOME")
            && let Some(h) = home.to_str() {
                dirs.push(Box::leak(format!("{}/Library/Fonts", h).into_boxed_str()));
            }
    }
    #[cfg(target_os = "linux")]
    {
        dirs.push("/usr/share/fonts");
        dirs.push("/usr/local/share/fonts");
        if let Some(home) = std::env::var_os("HOME") {
            if let Some(h) = home.to_str() {
                dirs.push(Box::leak(format!("{}/.fonts", h).into_boxed_str()));
                dirs.push(Box::leak(format!("{}/.local/share/fonts", h).into_boxed_str()));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        dirs.push("C:\\Windows\\Fonts");
    }

    let mut names: Vec<String> = Vec::new();
    for d in dirs {
        let p = Path::new(d);
        if !p.exists() {
            continue;
        }
        collect_font_names(p, &mut names, 2);
    }
    for n in &mut names {
        *n = normalize_font_name(n);
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_font_names(dir: &Path, out: &mut Vec<String>, depth: usize) {
    if depth == 0 {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_font_names(&path, out, depth.saturating_sub(1));
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_ascii_lowercase();
                if ["ttf", "otf", "ttc", "otc", "woff", "woff2"].contains(&ext.as_str())
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        out.push(stem.to_string());
                    }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_font_name(s: &str) -> String {
    let mut n = s.replace(['_', '-'], " ");
    if let Some(pos) = n.rfind('.') {
        n.truncate(pos);
    }
    n.trim().to_string()
}

/// 处理字体相关状态。
///
/// # 参数
/// - `family`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(crate) fn available_weights_for_font(family: &str) -> Vec<String> {
    let name = family.to_ascii_lowercase();
    if name.is_empty() {
        return Vec::new();
    }
    if name.contains("roboto") || name.contains("inter") {
        return vec![
            "Light".to_string(),
            "Regular".to_string(),
            "Medium".to_string(),
            "Semi Bold".to_string(),
            "Bold".to_string(),
            "Extra Bold".to_string(),
        ];
    }
    if name.contains("pingfang") || name.contains("ping fang") || name.contains("sf pro") {
        return vec![
            "Light".to_string(),
            "Regular".to_string(),
            "Medium".to_string(),
            "Semi Bold".to_string(),
            "Bold".to_string(),
        ];
    }
    if name.contains("source han") || name.contains("noto sans cjk") || name.contains("noto sans") {
        return vec!["Regular".to_string(), "Bold".to_string()];
    }
    if name.contains("microsoft yahei") || name.contains("yahei") {
        return vec!["Regular".to_string(), "Bold".to_string()];
    }
    if name.contains("arial")
        || name.contains("helvetica")
        || name.contains("times")
        || name.contains("courier")
        || name.contains("georgia")
        || name.contains("verdana")
        || name.contains("tahoma")
    {
        return vec!["Regular".to_string(), "Bold".to_string()];
    }
    Vec::new()
}

/// 处理字体相关状态。
///
/// # 参数
/// - `v`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(crate) fn font_weight_label(v: &Option<serde_json::Value>) -> String {
    match v.as_ref().and_then(|val| val.as_str()) {
        Some("300") => "Light".to_string(),
        Some("400") => "Regular".to_string(),
        Some("500") => "Medium".to_string(),
        Some("600") => "Semi Bold".to_string(),
        Some("700") => "Bold".to_string(),
        Some("800") => "Extra Bold".to_string(),
        Some(other) => other.to_string(),
        None => "Regular".to_string(),
    }
}

/// 处理字体相关状态。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(crate) fn font_weight_value_from_label(label: &str) -> String {
    match label {
        "Light" => "300".to_string(),
        "Regular" => "400".to_string(),
        "Medium" => "500".to_string(),
        "Semi Bold" => "600".to_string(),
        "Bold" => "700".to_string(),
        "Extra Bold" => "800".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
#[path = "typography_tests.rs"]
mod typography_tests;
