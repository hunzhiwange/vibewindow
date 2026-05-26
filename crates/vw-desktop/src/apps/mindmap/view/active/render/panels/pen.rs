//! 画笔工具面板渲染模块
//!
//! 本模块提供了思维导图画笔工具的设置面板UI组件。当用户选择画笔工具时，
//! 面板会显示颜色选择器和笔触宽度调节滑块，允许用户自定义涂鸦样式。
//!
//! # 主要功能
//!
//! - **颜色选择**：提供9种预设颜色（白色、黑色、红色、橙色、黄色、绿色、青色、蓝色、紫色）
//! - **宽度调节**：支持1px到18px的笔触宽度调节
//! - **视觉反馈**：当前选中的颜色会高亮显示，悬停状态有视觉提示

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapTab};
use iced::widget::{Space, button, container, row, slider, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::super::super::super::common::rgba_u32_to_color;

/// 创建画笔工具面板
///
/// 根据当前标签页的工具状态，渲染画笔工具的设置面板。
/// 当用户切换到画笔工具时，面板会显示颜色选择器和宽度调节器。
/// 如果当前工具不是画笔，则返回一个零尺寸的空容器。
///
/// # 参数
///
/// - `tab` - 当前思维导图标签页的状态引用，包含画笔工具的设置信息
/// - `pen_panel_w` - 面板的宽度（像素）
/// - `pen_panel_h` - 面板的高度（像素）
///
/// # 返回值
///
/// 返回一个Iced UI元素，包含颜色选择器和宽度滑块。
///
/// # UI结构
///
/// ```text
/// [颜色色块1][颜色色块2]...[颜色色块9] [间隔] [宽度滑块] [宽度数值]
/// ```
pub(in super::super) fn pen_panel(
    tab: &MindMapTab,
    pen_panel_w: f32,
    pen_panel_h: f32,
) -> Element<'_, Message> {
    // 如果当前工具不是画笔，返回零尺寸的空容器（隐藏面板）
    if tab.canvas_tool != MindMapCanvasTool::Pen {
        return container(Space::new()).width(Length::Fixed(0.0)).height(Length::Fixed(0.0)).into();
    }

    // 预定义的颜色调色板（RGBA格式，u32表示）
    // 依次为：白色、黑色、红色、橙色、黄色、绿色、青色、蓝色、紫色
    let palette = [
        0xFFFFFFFF, 0x111827FF, 0xEF4444FF, 0xF97316FF, 0xF59E0BFF, 0x22C55EFF, 0x06B6D4FF,
        0x3B82F6FF, 0xA855F7FF,
    ];

    // 创建单个颜色选择按钮的闭包
    // 该按钮显示一个色块，点击后设置画笔颜色
    let swatch_btn = |rgba: u32| -> Element<'_, Message> {
        // 检查该颜色是否为当前选中的颜色
        let active = tab.doodle_rgba == rgba;
        // 将u32格式的RGBA转换为Iced的Color类型
        let bg = rgba_u32_to_color(rgba);

        button(container(Space::new()).width(Length::Fill).height(Length::Fill))
            .padding(0)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            // 点击按钮时发送设置画笔颜色的消息
            .on_press(Message::MindMapTool(MindMapMessage::SetDoodleColor(rgba)))
            .style(move |theme: &Theme, status| {
                let p = theme.extended_palette();
                // 判断颜色是否为深色（RGB分量之和小于1.0）
                let swatch_is_dark = bg.r + bg.g + bg.b < 1.0;

                iced::widget::button::Style {
                    // 按钮背景色即为色块颜色
                    background: Some(Background::Color(bg)),
                    border: Border {
                        // 选中状态的边框宽度为2.0，否则为1.0
                        width: if active { 2.0 } else { 1.0 },
                        // 边框颜色根据状态决定
                        color: if active {
                            // 选中时使用主题的主色调
                            p.primary.base.color
                        } else {
                            match status {
                                // 悬停状态使用强背景色
                                iced::widget::button::Status::Hovered => p.background.strong.color,
                                _ => {
                                    // 其他状态使用半透明边框
                                    // 深色色块使用白色半透明边框，浅色色块使用黑色半透明边框
                                    if swatch_is_dark {
                                        Color::from_rgba8(255, 255, 255, 0.16)
                                    } else {
                                        Color::from_rgba8(0, 0, 0, 0.12)
                                    }
                                }
                            }
                        },
                        // 圆角边框（999.0表示完全圆形）
                        radius: 999.0.into(),
                    },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .into()
    };

    // 创建颜色色块行容器，将所有颜色按钮依次添加到行中
    let mut swatches = row![].spacing(6).align_y(Alignment::Center);
    for rgba in palette {
        swatches = swatches.push(swatch_btn(rgba));
    }

    // 创建笔触宽度滑块
    // 范围：1.0px 到 18.0px，当前值为标签页中保存的宽度值
    let width_slider = slider(1.0..=18.0, tab.doodle_width_px, |v| {
        Message::MindMapTool(MindMapMessage::SetDoodleWidth(v))
    })
    .width(Length::Fixed(120.0));

    // 组装面板内容：颜色色块 + 间隔 + 宽度滑块 + 宽度数值文本
    let panel = row![
        swatches,
        Space::new().width(Length::Fixed(10.0)).height(Length::Fixed(1.0)),
        width_slider,
        // 显示当前宽度值（单位：px），限制在1.0-18.0范围内
        text(format!("{:.0}px", tab.doodle_width_px.clamp(1.0, 18.0))).size(12),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    // 将面板内容包装在容器中，应用背景、边框和阴影样式
    container(panel)
        .width(Length::Fixed(pen_panel_w))
        .height(Length::Fixed(pen_panel_h))
        .padding([6, 10])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                // 使用主题的基础背景色
                background: Some(p.background.base.color.into()),
                // 添加淡色边框和圆角
                border: Border { width: 1.0, color: p.background.weak.color, radius: 10.0.into() },
                // 添加柔和的投影效果
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.12),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 22.0,
                },
                ..Default::default()
            }
        })
        .into()
}
