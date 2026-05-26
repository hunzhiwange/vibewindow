//! 外观属性面板模块
//!
//! 本模块提供设计元素外观属性的渲染和管理功能，包括：
//! - 透明度设置
//! - 视觉效果（投影、内阴影、图层模糊、背景模糊）的添加、编辑和删除
//!
//! # 主要组件
//!
//! - [`render_appearance`] - 渲染外观属性面板（透明度）
//! - [`render_effects`] - 渲染效果列表面板
//! - [`render_popover`] - 渲染效果详情弹出窗口
//!
//! # 支持的效果类型
//!
//! - **投影（Drop Shadow）**: 在元素外部创建阴影效果
//! - **内阴影（Inner Shadow）**: 在元素内部创建阴影效果
//! - **图层模糊（Layer Blur）**: 对整个图层进行模糊处理
//! - **背景模糊（Background Blur）**: 对元素背景进行模糊处理

use iced::widget::{Space, button, column, container, pick_list, row, text, text_input};
use iced::{Color, Element, Length, Point, Theme};
use serde_json::Value;

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{ColorPickerTarget, DesignElement, Effect, Offset};

use super::utils::{help_icon, prop_section, prop_section_with_help, prop_text_input_style};

/// 活动效果选择器状态
///
/// 用于跟踪当前正在编辑的效果选择器状态，包括其所属元素、效果索引和屏幕位置。
#[derive(Debug, Clone)]
pub struct ActiveEffectPicker {
    // 目标设计元素的唯一标识符
    pub element_id: String,
    // 效果在元素效果列表中的索引位置
    pub effect_index: usize,
    // 效果选择器弹出窗口在屏幕上的位置（用于定位弹出窗口）
    pub position: Point,
}

/// 渲染外观属性面板
///
/// 创建并返回用于设置元素外观属性的 UI 组件，当前仅支持透明度设置。
///
/// # 参数
///
/// - `element`: 设计元素的引用，包含元素的 ID 和透明度等属性
///
/// # 返回值
///
/// 返回包含外观属性控件的 Iced UI 元素
///
/// # 示例
///
/// ```ignore
/// let element = DesignElement { id: "button-1".to_string(), opacity: Some(0.8), ... };
/// let ui = render_appearance(&element);
/// ```
pub fn render_appearance<'a>(element: &'a DesignElement) -> Element<'a, Message> {
    let id = element.id.clone();
    let opacity = format_opacity((element.opacity.unwrap_or(1.0) * 100.0) as f64);

    column![
        column![
            text("外观")
                .size(12)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            prop_section(
                "颜色透明度",
                text_input("100", &opacity)
                    .on_input({
                        let id = id.clone();
                        move |s| {
                            let val = s.parse::<f64>().unwrap_or(100.0).clamp(0.0, 100.0);
                            Message::Design(DesignMessage::PropertyUpdate(
                                id.clone(),
                                "opacity".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(val / 100.0).unwrap(),
                                ),
                            ))
                        }
                    })
                    .style(prop_text_input_style),
            ),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .into()
}

/// 渲染效果列表面板
///
/// 创建并返回用于管理和添加视觉效果的 UI 组件。显示当前元素的所有效果，
/// 并提供添加新效果、删除效果、切换效果可见性等功能。
///
/// # 参数
///
/// - `element`: 设计元素的引用，包含元素 ID 和效果列表
/// - `selected_index`: 当前选中的效果索引（如果有）
///
/// # 返回值
///
/// 返回包含效果列表和添加效果按钮的 Iced UI 元素
///
/// # 功能
///
/// - 显示所有已添加的效果的列表
/// - 提供添加投影、内阴影、图层模糊、背景模糊的下拉菜单
/// - 每个效果项提供选择、切换可见性、删除等操作按钮
pub fn render_effects<'a>(
    element: &'a DesignElement,
    selected_index: Option<usize>,
) -> Element<'a, Message> {
    let id = element.id.clone();
    let effects = parse_effects(&element.effect);
    let effect_list =
        container(render_effect_list(effects.to_vec(), &id, selected_index)).width(Length::Fill);

    let add_effect_options = vec![
        "投影".to_string(),
        "内阴影".to_string(),
        "图层模糊".to_string(),
        "背景模糊".to_string(),
    ];

    column![
        column![
            row![
                text("效果").size(12).font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
                Space::new().width(Length::Fixed(6.0)),
                help_icon("添加或管理元素的视觉效果，如阴影与模糊"),
                Space::new().width(Length::Fill),
                pick_list(add_effect_options, None::<String>, move |selected| {
                    add_effect_of_kind(id.clone(), &effects, &selected)
                })
                .placeholder("+")
                .width(Length::Shrink)
                .text_size(12)
                .padding([5, 8])
            ]
            .align_y(iced::Alignment::Center),
            effect_list,
        ]
        .spacing(10)
        .width(Length::Fill)
    ]
    .spacing(10)
    .width(Length::Fill)
    .into()
}

/// 渲染效果详情弹出窗口
///
/// 根据指定的效果索引渲染该效果的详细编辑面板，用于调整效果的具体参数。
///
/// # 参数
///
/// - `element`: 设计元素的引用，包含元素 ID 和效果列表
/// - `effect_index`: 要编辑的效果在效果列表中的索引
///
/// # 返回值
///
/// 返回包含效果详细参数编辑控件的 Iced UI 元素。如果索引无效，返回空列。
pub fn render_popover<'a>(element: &'a DesignElement, effect_index: usize) -> Element<'a, Message> {
    let id = element.id.clone();
    let effects = parse_effects(&element.effect);
    if let Some(effect) = effects.get(effect_index) {
        render_effect_details(effect, effect_index, &effects, &id)
    } else {
        column![].into()
    }
}

/// 解析效果数据
///
/// 将 JSON 值解析为效果列表。支持单个效果对象或效果数组的 JSON 格式。
///
/// # 参数
///
/// - `value`: 可选的 JSON 值，可能是单个效果对象或效果数组
///
/// # 返回值
///
/// 返回解析后的效果向量。如果解析失败或值为 None，返回空向量。
fn parse_effects(value: &Option<Value>) -> Vec<Effect> {
    let value = match value {
        Some(v) => v,
        None => return vec![],
    };
    if let Ok(effects) = serde_json::from_value::<Vec<Effect>>(value.clone()) {
        effects
    } else if let Ok(effect) = serde_json::from_value::<Effect>(value.clone()) {
        vec![effect]
    } else {
        vec![]
    }
}

/// 渲染效果列表
///
/// 创建所有效果的列表视图，每个效果项包含图标、名称、选择按钮、可见性切换和删除按钮。
///
/// # 参数
///
/// - `effects`: 效果向量，包含所有要显示的效果
/// - `id`: 设计元素的唯一标识符
/// - `selected_index`: 当前选中的效果索引
///
/// # 返回值
///
/// 返回包含所有效果项的列布局 UI 元素
fn render_effect_list(
    effects: Vec<Effect>,
    id: &str,
    selected_index: Option<usize>,
) -> Element<'static, Message> {
    let mut col = column![].spacing(5).width(Length::Fill);

    // 创建操作按钮的辅助闭包
    //
    // # 参数
    // - `icon`: 按钮显示的图标
    // - `on_press`: 按钮点击时发送的消息
    //
    // # 返回值
    // 返回一个带有悬停/按下样式的图标按钮
    let action_btn = |icon: Icon, on_press: Message| {
        let content = container(iced::widget::svg(assets::get_icon(icon)).width(12).height(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        button(content)
            .on_press(on_press)
            .style(|theme: &Theme, status| {
                let ext = theme.extended_palette();
                let background = match status {
                    button::Status::Hovered => Some(ext.background.weak.color),
                    button::Status::Pressed => Some(ext.background.strong.color),
                    _ => None,
                };
                button::Style {
                    background: background.map(Into::into),
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    ..button::Style::default()
                }
            })
            .padding(0)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
    };

    // 逆序遍历效果列表，使最上层的效果在 UI 中优先显示
    for (i, effect) in effects.iter().enumerate().rev() {
        let is_selected = selected_index == Some(i);

        // 根据效果类型确定显示的图标和标签
        let (icon, label) = match effect.kind.as_str() {
            "shadow" => {
                if effect.shadow_type.as_deref() == Some("inner") {
                    (Icon::Square, "内阴影".to_string())
                } else {
                    (Icon::Square, "投影".to_string())
                }
            }
            "layer_blur" => (Icon::Sliders, "图层模糊".to_string()),
            "background_blur" => (Icon::Image, "背景模糊".to_string()),
            _ => (Icon::Circle, effect.kind.clone()),
        };

        let select_area = row![
            container(iced::widget::svg(assets::get_icon(icon)).width(16).height(16))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(24.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            container(text(label).size(12))
                .height(Length::Fixed(24.0))
                .align_y(iced::alignment::Vertical::Center),
            Space::new().width(Length::Fill),
        ]
        .spacing(10)
        .height(Length::Fixed(24.0))
        .width(Length::Fill)
        .align_y(iced::Alignment::Center);

        let select_button = button(container(select_area).width(Length::Fill))
            .on_press(Message::Design(DesignMessage::OpenEffectPicker(id.to_string(), i, None)))
            .style(button::text)
            .padding(0)
            .width(Length::Fill);

        let row_content = row![
            select_button,
            action_btn(
                if effect.visible.unwrap_or(true) { Icon::Eye } else { Icon::EyeSlash },
                toggle_effect(id.to_string(), &effects, i),
            ),
            action_btn(Icon::Trash, remove_effect(id.to_string(), &effects, i))
        ]
        .spacing(10)
        .height(Length::Fixed(24.0))
        .width(Length::Fill)
        .align_y(iced::Alignment::Center);

        let row_container = container(row_content)
            .style(move |theme: &Theme| {
                let p = theme.palette();
                container::Style {
                    background: None,
                    border: iced::Border {
                        width: if is_selected { 1.0 } else { 0.0 },
                        color: if is_selected { p.primary } else { Color::TRANSPARENT },
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding(6)
            .height(Length::Fixed(36.0))
            .width(Length::Fill);

        col = col.push(row_container);
    }

    col.into()
}

/// 渲染效果详情编辑面板
///
/// 根据效果类型渲染相应的参数编辑控件，包括：
/// - 阴影效果：类型选择、颜色、偏移（X/Y）、模糊半径、扩散半径
/// - 模糊效果：模糊半径
///
/// # 参数
///
/// - `effect`: 要编辑的效果引用
/// - `index`: 效果在列表中的索引
/// - `all_effects`: 所有效果的切片引用，用于更新整个效果列表
/// - `id`: 设计元素的唯一标识符
///
/// # 返回值
///
/// 返回包含效果参数编辑控件的 UI 元素
fn render_effect_details(
    effect: &Effect,
    index: usize,
    all_effects: &[Effect],
    id: &str,
) -> Element<'static, Message> {
    let mut col = column![].spacing(10);

    match effect.kind.as_str() {
        "shadow" => {
            let is_inner = effect.shadow_type.as_deref() == Some("inner");
            let type_options = vec!["投影", "内阴影"];
            let selected_type = if is_inner { "内阴影" } else { "投影" };

            col = col.push(prop_section_with_help(
                "类型",
                "选择阴影类型：外投影或内阴影",
                pick_list(type_options, Some(selected_type), {
                    let id = id.to_string();
                    let all_effects = all_effects.to_vec();
                    move |s| update_shadow_type(id.clone(), &all_effects, index, s == "内阴影")
                })
                .width(Length::Fill)
                .text_size(12)
                .padding([5, 8]),
            ));

            let offset = effect.offset.clone().unwrap_or(Offset { x: 0.0, y: 0.0 });
            let blur = effect.blur.unwrap_or(0.0);
            let spread = effect.spread.unwrap_or(0.0);
            let color = effect.color.clone().unwrap_or("#000000".to_string());

            // 颜色选择器区域：包含颜色预览块和十六进制输入框
            let (r, g, b, a) = parse_hex_to_rgba(&color);
            let current_color = Color::from_rgba(r, g, b, a);

            // 颜色预览按钮：点击打开颜色选择器
            let preview =
                button(container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                    move |_: &Theme| container::Style {
                        background: Some(current_color.into()),
                        border: iced::Border {
                            color: Color::from_rgb(0.8, 0.8, 0.8),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    },
                ))
                .width(32)
                .height(32)
                .padding(0)
                .on_press(Message::Design(DesignMessage::OpenColorPicker(
                    current_color,
                    ColorPickerTarget::Effect { element_id: id.to_string(), effect_index: index },
                    None,
                )));

            // 颜色十六进制输入框：允许直接输入颜色值
            let color_input = text_input("#000000", &color)
                .on_input(update_effect_color(id.to_string(), all_effects, index))
                .style(prop_text_input_style)
                .width(Length::Fill);

            col = col.push(prop_section_with_help(
                "颜色",
                "阴影颜色（支持 #RRGGBBAA）",
                row![preview, color_input].spacing(10).align_y(iced::Alignment::Center),
            ));

            // 偏移量（X/Y）设置区域
            col = col.push(
                row![
                    prop_section_with_help(
                        "X",
                        "阴影在水平方向的偏移量（px），可为负",
                        text_input("0", &offset.x.to_string())
                            .on_input(update_effect_offset_x(id.to_string(), all_effects, index))
                            .style(prop_text_input_style)
                    ),
                    prop_section_with_help(
                        "Y",
                        "阴影在垂直方向的偏移量（px），可为负",
                        text_input("0", &offset.y.to_string())
                            .on_input(update_effect_offset_y(id.to_string(), all_effects, index))
                            .style(prop_text_input_style)
                    ),
                ]
                .spacing(10),
            );

            // 模糊和扩散半径设置区域
            col = col.push(
                row![
                    prop_section_with_help(
                        "模糊",
                        "模糊半径（px），值越大越柔和",
                        text_input("0", &blur.to_string())
                            .on_input(update_effect_blur(id.to_string(), all_effects, index))
                            .style(prop_text_input_style)
                    ),
                    prop_section_with_help(
                        "扩散",
                        "阴影扩散半径（px），扩大阴影范围",
                        text_input("0", &spread.to_string())
                            .on_input(update_effect_spread(id.to_string(), all_effects, index))
                            .style(prop_text_input_style)
                    ),
                ]
                .spacing(10),
            );
        }
        "layer_blur" | "background_blur" => {
            let radius = effect.radius.unwrap_or(4.0);
            col = col.push(prop_section_with_help(
                "模糊",
                "模糊半径（px），作用于图层或背景",
                text_input("4", &radius.to_string())
                    .on_input(update_effect_radius(id.to_string(), all_effects, index))
                    .style(prop_text_input_style),
            ));
        }
        _ => {}
    }

    col.into()
}

/// 解析十六进制颜色字符串为 RGBA 值
///
/// 将十六进制颜色字符串（#RRGGBB 或 #RRGGBBAA 格式）转换为标准化的 RGBA 浮点值。
///
/// # 参数
///
/// - `hex`: 十六进制颜色字符串，支持格式：
///   - `#RRGGBB` (6 位，不透明)
///   - `#RRGGBBAA` (8 位，带透明度)
///
/// # 返回值
///
/// 返回包含红、绿、蓝、透明度分量的元组，每个分量范围 [0.0, 1.0]。
/// 如果解析失败，返回黑色 `(0.0, 0.0, 0.0, 1.0)`。
///
/// # 示例
///
/// ```ignore
/// let (r, g, b, a) = parse_hex_to_rgba("#FF0000");      // 红色，不透明
/// let (r, g, b, a) = parse_hex_to_rgba("#FF000080");    // 红色，半透明
/// ```
fn parse_hex_to_rgba(hex: &str) -> (f32, f32, f32, f32) {
    if hex.len() >= 7 && hex.starts_with('#')
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[1..3], 16),
            u8::from_str_radix(&hex[3..5], 16),
            u8::from_str_radix(&hex[5..7], 16),
        ) {
            let a = if hex.len() == 9 {
                u8::from_str_radix(&hex[7..9], 16).unwrap_or(255)
            } else {
                255
            };
            return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0);
        }
    (0.0, 0.0, 0.0, 1.0)
}

/// 添加指定类型的效果
///
/// 根据效果类型字符串创建新的效果并添加到元素的效果列表中。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `kind_str`: 效果类型字符串，支持中英文：
///   - "投影" / "Drop shadow"
///   - "内阴影" / "Inner shadow"
///   - "图层模糊" / "Layer blur"
///   - "背景模糊" / "Background blur"
///
/// # 返回值
///
/// 返回用于更新属性的消息，如果效果类型无效则返回 `Message::None`
fn add_effect_of_kind(id: String, effects: &[Effect], kind_str: &str) -> Message {
    let mut new_effects = effects.to_vec();
    let new_effect = match kind_str {
        "Drop shadow" | "投影" => Effect {
            kind: "shadow".to_string(),
            shadow_type: Some("outer".to_string()),
            color: Some("#00000040".to_string()),
            offset: Some(Offset { x: 0.0, y: 4.0 }),
            blur: Some(4.0),
            spread: Some(0.0),
            radius: None,
            visible: Some(true),
            enabled: Some(true),
        },
        "Inner shadow" | "内阴影" => Effect {
            kind: "shadow".to_string(),
            shadow_type: Some("inner".to_string()),
            color: Some("#00000040".to_string()),
            offset: Some(Offset { x: 0.0, y: 4.0 }),
            blur: Some(4.0),
            spread: Some(0.0),
            radius: None,
            visible: Some(true),
            enabled: Some(true),
        },
        "Layer blur" | "图层模糊" => Effect {
            kind: "layer_blur".to_string(),
            shadow_type: None,
            color: None,
            offset: None,
            blur: None,
            spread: None,
            radius: Some(4.0),
            visible: Some(true),
            enabled: Some(true),
        },
        "Background blur" | "背景模糊" => Effect {
            kind: "background_blur".to_string(),
            shadow_type: None,
            color: None,
            offset: None,
            blur: None,
            spread: None,
            radius: Some(4.0),
            visible: Some(true),
            enabled: Some(true),
        },
        _ => return Message::None,
    };

    new_effects.push(new_effect);
    update_effects_message(id, new_effects)
}

/// 添加默认投影效果
///
/// 向元素添加一个默认的投影效果（Drop shadow）。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
///
/// # 返回值
///
/// 返回用于更新属性的消息
#[allow(dead_code)]
fn add_effect(id: String, effects: &[Effect]) -> Message {
    add_effect_of_kind(id, effects, "Drop shadow")
}

/// 移除指定索引的效果
///
/// 从效果列表中删除指定位置的效果。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要删除的效果索引
///
/// # 返回值
///
/// 返回用于更新属性的消息
fn remove_effect(id: String, effects: &[Effect], index: usize) -> Message {
    let mut new_effects = effects.to_vec();
    if index < new_effects.len() {
        new_effects.remove(index);
    }
    update_effects_message(id, new_effects)
}

/// 切换效果的可见性
///
/// 切换指定效果的可见状态（显示/隐藏）。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要切换的效果索引
///
/// # 返回值
///
/// 返回用于更新属性的消息
fn toggle_effect(id: String, effects: &[Effect], index: usize) -> Message {
    let mut new_effects = effects.to_vec();
    if let Some(effect) = new_effects.get_mut(index) {
        let vis = effect.visible.unwrap_or(true);
        effect.visible = Some(!vis);
    }
    update_effects_message(id, new_effects)
}

/// 更新阴影类型
///
/// 修改阴影效果为投影（outer）或内阴影（inner）。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
/// - `is_inner`: true 表示内阴影，false 表示投影
///
/// # 返回值
///
/// 返回用于更新属性的消息
fn update_shadow_type(id: String, effects: &[Effect], index: usize, is_inner: bool) -> Message {
    let mut new_effects = effects.to_vec();
    if let Some(effect) = new_effects.get_mut(index) {
        effect.shadow_type = Some(if is_inner { "inner".to_string() } else { "outer".to_string() });
    }
    update_effects_message(id, new_effects)
}

/// 创建更新效果 X 偏移量的闭包
///
/// 生成一个用于处理 X 偏移量输入变更的闭包函数。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
///
/// # 返回值
///
/// 返回一个闭包，接收输入字符串并返回更新消息
fn update_effect_offset_x(
    id: String,
    effects: &[Effect],
    index: usize,
) -> Box<dyn Fn(String) -> Message> {
    let effects = effects.to_vec();
    Box::new(move |s| {
        let val = s.parse::<f32>().unwrap_or(0.0);
        let mut new_effects = effects.clone();
        if let Some(effect) = new_effects.get_mut(index) {
            let mut offset = effect.offset.clone().unwrap_or(Offset { x: 0.0, y: 0.0 });
            offset.x = val;
            effect.offset = Some(offset);
        }
        update_effects_message(id.clone(), new_effects)
    })
}

/// 创建更新效果 Y 偏移量的闭包
///
/// 生成一个用于处理 Y 偏移量输入变更的闭包函数。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
///
/// # 返回值
///
/// 返回一个闭包，接收输入字符串并返回更新消息
fn update_effect_offset_y(
    id: String,
    effects: &[Effect],
    index: usize,
) -> Box<dyn Fn(String) -> Message> {
    let effects = effects.to_vec();
    Box::new(move |s| {
        let val = s.parse::<f32>().unwrap_or(0.0);
        let mut new_effects = effects.clone();
        if let Some(effect) = new_effects.get_mut(index) {
            let mut offset = effect.offset.clone().unwrap_or(Offset { x: 0.0, y: 0.0 });
            offset.y = val;
            effect.offset = Some(offset);
        }
        update_effects_message(id.clone(), new_effects)
    })
}

/// 创建更新效果模糊半径的闭包
///
/// 生成一个用于处理模糊半径输入变更的闭包函数。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
///
/// # 返回值
///
/// 返回一个闭包，接收输入字符串并返回更新消息
fn update_effect_blur(
    id: String,
    effects: &[Effect],
    index: usize,
) -> Box<dyn Fn(String) -> Message> {
    let effects = effects.to_vec();
    Box::new(move |s| {
        let val = s.parse::<f32>().unwrap_or(0.0);
        let mut new_effects = effects.clone();
        if let Some(effect) = new_effects.get_mut(index) {
            effect.blur = Some(val);
        }
        update_effects_message(id.clone(), new_effects)
    })
}

/// 创建更新效果扩散半径的闭包
///
/// 生成一个用于处理扩散半径输入变更的闭包函数。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
///
/// # 返回值
///
/// 返回一个闭包，接收输入字符串并返回更新消息
fn update_effect_spread(
    id: String,
    effects: &[Effect],
    index: usize,
) -> Box<dyn Fn(String) -> Message> {
    let effects = effects.to_vec();
    Box::new(move |s| {
        let val = s.parse::<f32>().unwrap_or(0.0);
        let mut new_effects = effects.clone();
        if let Some(effect) = new_effects.get_mut(index) {
            effect.spread = Some(val);
        }
        update_effects_message(id.clone(), new_effects)
    })
}

/// 创建更新效果半径的闭包
///
/// 生成一个用于处理模糊效果半径输入变更的闭包函数。
/// 适用于图层模糊和背景模糊效果。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
///
/// # 返回值
///
/// 返回一个闭包，接收输入字符串并返回更新消息
fn update_effect_radius(
    id: String,
    effects: &[Effect],
    index: usize,
) -> Box<dyn Fn(String) -> Message> {
    let effects = effects.to_vec();
    Box::new(move |s| {
        let val = s.parse::<f32>().unwrap_or(0.0);
        let mut new_effects = effects.clone();
        if let Some(effect) = new_effects.get_mut(index) {
            effect.radius = Some(val);
        }
        update_effects_message(id.clone(), new_effects)
    })
}

/// 创建更新效果颜色的闭包
///
/// 生成一个用于处理效果颜色输入变更的闭包函数。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 现有效果列表的切片
/// - `index`: 要更新的效果索引
///
/// # 返回值
///
/// 返回一个闭包，接收颜色字符串并返回更新消息
fn update_effect_color(
    id: String,
    effects: &[Effect],
    index: usize,
) -> Box<dyn Fn(String) -> Message> {
    let effects = effects.to_vec();
    Box::new(move |s| {
        let mut new_effects = effects.clone();
        if let Some(effect) = new_effects.get_mut(index) {
            effect.color = Some(s);
        }
        update_effects_message(id.clone(), new_effects)
    })
}

/// 创建更新效果列表的消息
///
/// 将更新后的效果列表序列化为 JSON 并创建属性更新消息。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `effects`: 更新后的效果向量
///
/// # 返回值
///
/// 返回用于更新元素效果属性的 `DesignMessage::PropertyUpdate` 消息
fn update_effects_message(id: String, effects: Vec<Effect>) -> Message {
    Message::Design(DesignMessage::PropertyUpdate(
        id,
        "effect".to_string(),
        serde_json::to_value(effects).unwrap_or(serde_json::Value::Null),
    ))
}

/// 格式化透明度值
///
/// 将透明度数值格式化为字符串，去除不必要的尾随零和小数点。
///
/// # 参数
///
/// - `value`: 透明度值（0.0 到 100.0）
///
/// # 返回值
///
/// 返回格式化后的字符串，例如：
/// - 100.0 -> "100"
/// - 50.50 -> "50.5"
/// - 0.0 -> "0"
fn format_opacity(value: f64) -> String {
    let clamped = value.clamp(0.0, 100.0);
    let mut s = format!("{:.2}", clamped);
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

#[cfg(test)]
#[path = "appearance_tests.rs"]
mod appearance_tests;
