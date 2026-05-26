//! 渐变填充属性编辑器模块
//!
//! 本模块提供了设计属性面板中渐变填充的完整编辑界面，支持以下功能：
//! - 四种渐变类型的可视化编辑（线性、径向、角向、网格）
//! - 渐变停靠点（color stops）的可视化编辑栏
//! - 渐变中心、大小、旋转角度等参数的精细控制
//! - 停靠点颜色和位置的实时预览与编辑
//!
//! ## 模块结构
//! - `actions`: 处理渐变相关的所有用户操作和状态更新
//! - `stops_bar`: 渐变停靠点的可视化进度条组件
//! - `utils`: 渐变处理相关的工具函数

use iced::widget::container;
use iced::widget::{Space, button, canvas, column, pick_list, row, text, text_input};
use iced::{Background, Color, Element, Length, Theme};
use std::fmt;

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::properties::fill::types::{FillItem, GradientFill};

use super::solid::parse_hex_to_rgba;
use crate::app::views::design::properties::utils::{
    PROP_INPUT_RADIUS, prop_section, prop_text_input_style,
};

mod actions;
mod stops_bar;
mod utils;

use self::stops_bar::GradientStopsBar;

/// 渐变类型选项
///
/// 用于渐变类型下拉选择框的选项结构体，包含显示标签和实际值。
/// 支持四种渐变类型：
/// - 线性渐变 (linear)
/// - 径向渐变 (radial)
/// - 角向渐变 (angular)
/// - 网格渐变 (mesh_gradient)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GradientTypeOption {
    /// 下拉框中显示的中文标签
    label: &'static str,
    /// 渐变类型的实际值，用于后端处理
    value: &'static str,
}

impl fmt::Display for GradientTypeOption {
    /// 格式化输出渐变类型的显示标签
    ///
    /// # 参数
    /// - `f`: 格式化器引用
    ///
    /// # 返回值
    /// 返回 `fmt::Result`，包含中文标签的格式化结果
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

/// 渲染渐变填充属性编辑面板
///
/// 创建一个完整的渐变填充编辑界面，包含类型选择器、停靠点编辑器、
/// 以及各种参数（中心点、大小、旋转）的输入控件。
///
/// # 参数
///
/// * `gradient` - 当前渐变填充的数据，包含渐变类型、颜色停靠点等
/// * `index` - 当前填充项在填充列表中的索引位置
/// * `fills` - 完整的填充列表，用于操作时重建整个填充状态
/// * `id` - 当前正在编辑的设计元素唯一标识符
///
/// # 返回值
///
/// 返回一个 `Element<Message>`，包含完整渲染的渐变编辑界面
///
/// # UI 组件
///
/// 界面由以下部分组成（从上到下）：
/// 1. **类型选择**：下拉选择渐变类型
/// 2. **停靠点头部**：标题和添加按钮
/// 3. **停靠点可视化栏**：Canvas 绘制的颜色渐变预览条
/// 4. **停靠点列表**：每个停靠点的位置、颜色和删除按钮
/// 5. **中心点设置**：X/Y 坐标百分比输入（径向/角向/线性可用）
/// 6. **大小设置**：水平和垂直尺寸百分比（径向/角向可用）
/// 7. **旋转设置**：旋转角度输入（线性/径向/角向可用）
///
/// # 示例
///
/// ```ignore
/// let element = render(
///     gradient_fill,
///     0,
///     fill_items,
///     "element-123".to_string(),
/// );
/// ```
pub fn render(
    gradient: GradientFill,
    index: usize,
    fills: Vec<FillItem>,
    id: String,
) -> Element<'static, Message> {
    // 构建渐变类型选项列表，包含中文标签和英文值
    let type_options = vec![
        GradientTypeOption { label: "线性", value: "linear" },
        GradientTypeOption { label: "径向", value: "radial" },
        GradientTypeOption { label: "角向", value: "angular" },
        GradientTypeOption { label: "网格", value: "mesh_gradient" },
    ];

    // 从选项列表中查找当前选中的渐变类型
    let selected_type = type_options.iter().copied().find(|o| o.value == gradient.gradient_type);

    // 中心点设置区域
    // 仅对线性、径向、角向渐变显示，网格渐变不支持
    let center_section: Element<Message> = if ["radial", "angular", "linear"]
        .contains(&gradient.gradient_type.as_str())
    {
        // 获取中心点坐标，默认为 (50%, 50%)
        let x = gradient.center.as_ref().map(|c| c.x).unwrap_or(50.0);
        let y = gradient.center.as_ref().map(|c| c.y).unwrap_or(50.0);

        prop_section(
            "中心 %",
            row![
                // X 坐标输入框，用于设置渐变中心的水平位置
                text_input("X 50%", &x.to_string())
                    .on_input({
                        let id = id.clone();
                        let fills = fills.clone();
                        move |s| {
                            actions::update_gradient_center_x(id.clone(), fills.clone(), index, s)
                        }
                    })
                    .style(prop_text_input_style)
                    .width(Length::Fill),
                // Y 坐标输入框，用于设置渐变中心的垂直位置
                text_input("Y 50%", &y.to_string())
                    .on_input({
                        let id = id.clone();
                        let fills = fills.clone();
                        move |s| {
                            actions::update_gradient_center_y(id.clone(), fills.clone(), index, s)
                        }
                    })
                    .style(prop_text_input_style)
                    .width(Length::Fill)
            ]
            .spacing(8),
        )
    } else {
        column![].into()
    };

    // 大小设置区域
    // 根据渐变类型显示不同的输入控件
    let size_section: Element<Message> =
        if ["radial", "linear", "angular"].contains(&gradient.gradient_type.as_str()) {
            // 获取大小参数，默认为 100%
            let height = gradient.size.as_ref().and_then(|s| s.height).unwrap_or(100.0);
            let size_h = gradient.size_h.unwrap_or(100.0);

            // 根据渐变类型决定显示的输入项
            // 线性渐变只有"垂直"方向（实际是渐变方向长度）
            // 径向和角向渐变有"水平"和"垂直"两个方向
            let size_inputs: Element<Message> = if gradient.gradient_type == "linear" {
                row![
                    text("垂直").size(12),
                    text_input("100", &size_h.to_string())
                        .on_input({
                            let id = id.clone();
                            let fills = fills.clone();
                            move |s| {
                                actions::update_gradient_size_h(id.clone(), fills.clone(), index, s)
                            }
                        })
                        .style(prop_text_input_style)
                        .width(Length::Fill)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                row![
                    text("水平").size(12),
                    text_input("100", &size_h.to_string())
                        .on_input({
                            let id = id.clone();
                            let fills = fills.clone();
                            move |s| {
                                actions::update_gradient_size_h(id.clone(), fills.clone(), index, s)
                            }
                        })
                        .style(prop_text_input_style),
                    text("垂直").size(12),
                    text_input("100", &height.to_string())
                        .on_input({
                            let id = id.clone();
                            let fills = fills.clone();
                            move |s| {
                                actions::update_gradient_size_v(id.clone(), fills.clone(), index, s)
                            }
                        })
                        .style(prop_text_input_style)
                ]
                .spacing(5)
                .align_y(iced::Alignment::Center)
                .into()
            };

            prop_section("大小 %", size_inputs)
        } else {
            column![].into()
        };

    // 旋转角度设置区域
    // 仅对线性、径向、角向渐变显示
    let rotation_section = if ["linear", "radial", "angular"]
        .contains(&gradient.gradient_type.as_str())
    {
        prop_section(
            "旋转",
            text_input("0", &gradient.rotation.to_string())
                .on_input({
                    let id = id.clone();
                    let fills = fills.clone();
                    move |s| actions::update_gradient_rotation(id.clone(), fills.clone(), index, s)
                })
                .style(prop_text_input_style),
        )
    } else {
        column![].into()
    };

    // 渐变停靠点可视化预览条
    // 使用 Canvas 绘制，高度 36px
    let stops_bar = canvas(GradientStopsBar {
        stops: gradient.colors.clone(),
        on_change: Box::new({
            let id = id.clone();
            let fills = fills.clone();
            move |new_stops| {
                actions::update_gradient_stops(id.clone(), fills.clone(), index, new_stops)
            }
        }),
    })
    .width(Length::Fill)
    .height(36);

    // 停靠点列表内容
    // 逆序显示停靠点（最新的在上面），每个停靠点包含位置、颜色色块、颜色输入和删除按钮
    let stops_list_content =
        column(gradient.colors.iter().enumerate().rev().map(|(stop_idx, stop)| {
            let stop_color = stop.color.clone();
            // 将位置从 0.0-1.0 转换为 0-100 百分比显示
            let stop_percent = (stop.position * 100.0).clamp(0.0, 100.0);
            let stop_pos = utils::format_percent(stop_percent);
            let id_clone = id.clone();
            let fills_clone = fills.clone();

            // 颜色色块按钮
            // 显示当前停靠点的颜色，点击可打开颜色选择器
            let swatch: Element<Message> =
                button(container(Space::new()).width(Length::Fill).height(Length::Fill))
                    .width(Length::Fixed(28.0))
                    .height(Length::Fixed(28.0))
                    .padding(0)
                    .style(move |theme: &Theme, status: button::Status| {
                        // 解析十六进制颜色为 RGBA
                        let rgba = parse_hex_to_rgba(&stop_color);
                        let c = Color::from_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
                        let ext = theme.extended_palette();
                        // 悬停时边框颜色变淡，提供视觉反馈
                        let border_color = if status == button::Status::Hovered {
                            ext.background.strong.color.scale_alpha(0.85)
                        } else {
                            ext.background.strong.color
                        };
                        button::Style {
                            background: Some(Background::Color(c)),
                            text_color: theme.palette().text,
                            border: iced::Border {
                                width: 1.0,
                                color: border_color,
                                radius: PROP_INPUT_RADIUS.into(),
                            },
                            ..button::Style::default()
                        }
                    })
                    .on_press({
                        let id = id.clone();
                        let color_str = stop.color.clone();
                        let rgba = parse_hex_to_rgba(&color_str);
                        let c = Color::from_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
                        // 点击时打开颜色选择器，目标是当前停靠点
                        Message::Design(DesignMessage::OpenColorPicker(
                            c,
                            crate::app::views::design::models::ColorPickerTarget::GradientStop {
                                element_id: id,
                                fill_index: index,
                                stop_index: stop_idx,
                            },
                            None,
                        ))
                    })
                    .into();

            // 单个停靠点的行布局
            row![
                // 位置输入框（百分比）
                text_input("位置", &stop_pos)
                    .width(Length::Fixed(60.0))
                    .padding(6)
                    .size(12)
                    .on_input({
                        let id = id.clone();
                        let fills = fills.clone();
                        move |s| {
                            actions::update_gradient_stop_position(
                                id.clone(),
                                fills.clone(),
                                index,
                                stop_idx,
                                s,
                            )
                        }
                    })
                    .style(prop_text_input_style),
                // 颜色色块按钮
                swatch,
                // 颜色值输入框（十六进制格式）
                text_input("#rrggbbaa", &stop.color)
                    .width(Length::Fill)
                    .padding(6)
                    .size(12)
                    .style(prop_text_input_style)
                    .on_input({
                        let id = id.clone();
                        let fills = fills.clone();
                        move |s| {
                            actions::update_gradient_stop_color(
                                id.clone(),
                                fills.clone(),
                                index,
                                stop_idx,
                                utils::normalize_hex_color_input(&s),
                            )
                        }
                    }),
                // 删除停靠点按钮
                button(
                    container(text("-").size(12).line_height(1.0))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0))
                .padding(0)
                .on_press(actions::remove_gradient_stop(id_clone, fills_clone, index, stop_idx))
                .style(|theme: &Theme, status: button::Status| {
                    let ext = theme.extended_palette();
                    // 根据按钮状态设置不同的背景色
                    let bg = match status {
                        button::Status::Pressed => Some(ext.background.strong.color.into()),
                        button::Status::Hovered => {
                            Some(ext.background.base.color.scale_alpha(0.70).into())
                        }
                        _ => Some(ext.background.base.color.into()),
                    };
                    button::Style {
                        background: bg,
                        text_color: theme.palette().text,
                        border: iced::Border {
                            width: 1.0,
                            color: ext.background.strong.color,
                            radius: PROP_INPUT_RADIUS.into(),
                        },
                        ..button::Style::default()
                    }
                })
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill)
            .into()
        }))
        .spacing(5)
        .width(Length::Fill);

    // 停靠点列表容器
    // 带有背景色和圆角边框的样式化容器
    let stops_list: Element<Message> = container(stops_list_content)
        .style(|theme: &Theme| {
            let extended = theme.extended_palette();
            container::Style {
                background: Some(extended.background.weak.color.into()),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: extended.background.strong.color,
                },
                ..Default::default()
            }
        })
        .padding(8)
        .width(Length::Fill)
        .into();

    // 渐变类型选择区域
    let type_section = prop_section(
        "类型",
        pick_list(type_options.clone(), selected_type, {
            let id = id.clone();
            let fills = fills.clone();
            move |o| {
                actions::update_gradient_type(id.clone(), fills.clone(), index, o.value.to_string())
            }
        })
        .width(Length::Fill),
    );

    // 停靠点区域头部
    // 包含标题和添加新停靠点的按钮
    let stops_header = row![
        text("停靠点").size(11).style(text::secondary),
        Space::new().width(Length::Fill),
        button(text("+").size(14))
            .on_press(actions::add_gradient_stop(id.clone(), fills.clone(), index))
            .style(button::text)
            .padding(2)
    ]
    .align_y(iced::Alignment::Center);

    // 组装完整的渐变编辑面板
    // 按从上到下的顺序排列所有组件
    column![
        type_section,
        stops_header,
        stops_bar,
        stops_list,
        center_section,
        size_section,
        rotation_section
    ]
    .spacing(10)
    .into()
}

#[cfg(test)]
mod tests;
