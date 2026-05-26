//! 组织架构图布局选择器模块
//!
//! 本模块提供了组织架构图布局格式的选择器 UI 组件。
//! 该组件允许用户在多种布局格式之间进行选择，包括：
//! - 自上而下布局（Top-Down）
//! - 从左到右布局（Left-Right）
//!
//! 主要功能：
//! - 渲染布局格式选择器面板
//! - 提供布局格式的可视化预览
//! - 支持用户交互式切换布局格式
//! - 自动高亮当前激活的布局格式

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapTab, OrgChartLayoutFormat};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::super::super::super::previews::OrgChartLayoutFormatPreview;

/// 创建组织架构图布局选择器 UI 组件
///
/// 该函数生成一个包含布局格式选择卡片的面板，每个卡片展示对应布局的可视化预览。
/// 用户可以通过点击卡片来切换不同的布局格式。
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态引用，用于获取当前激活的布局格式
/// * `desc_w` - 描述区域的宽度（像素），用于计算卡片布局
///
/// # 返回值
///
/// 返回 `Option<Element<'static, Message>>`：
/// - `Some(Element)` - 包含布局选择器面板的 UI 元素
/// - `None` - 在当前上下文中不显示选择器（预留扩展）
///
/// # 示例
///
/// ```ignore
/// let tab = MindMapTab::default();
/// let element = org_chart_layout_picker(&tab, 300.0);
/// // element 包含一个带有两个布局选择卡片的 UI 组件
/// ```
///
/// # UI 结构
///
/// ```text
/// ┌─────────────────────────────┐
/// │        布局格式              │
/// ├───────────────┬─────────────┤
/// │  ┌──────────┐ │ ┌─────────┐ │
/// │  │ 预览图像  │ │ │预览图像 │ │
/// │  │  从上到下 │ │ │ 从左到右│ │
/// │  └──────────┘ │ └─────────┘ │
/// └───────────────┴─────────────┘
/// ```
pub(in super::super) fn org_chart_layout_picker(
    tab: &MindMapTab,
    desc_w: f32,
) -> Option<Element<'static, Message>> {
    // 定义可用的布局格式选项
    let formats = [OrgChartLayoutFormat::TopDown, OrgChartLayoutFormat::LeftRight];

    // 卡片之间的间距（像素）
    let card_gap = 10.0;
    // 计算每个卡片的宽度，确保至少为 160 像素
    let card_w = ((desc_w - card_gap * 1.0) / 2.0).max(160.0);
    // 卡片的固定高度
    let card_h = 66.0;

    // 定义创建布局格式卡片的闭包
    // 该闭包为每个布局格式生成一个可交互的按钮卡片
    let card = |f: OrgChartLayoutFormat| {
        // 判断当前格式是否为激活状态
        let active = tab.org_chart_layout_format == f;

        // 创建布局格式的预览画布元素
        // 使用 Canvas 组件渲染布局格式的可视化预览
        let preview: Element<'static, Message> =
            iced::widget::canvas(OrgChartLayoutFormatPreview {
                format: f,
                color: Color::from_rgba8(0, 0, 0, 0.68), // 深灰色，透明度 68%
            })
            .width(Length::Fill)
            .height(Length::Fixed(34.0))
            .into();

        // 构建卡片按钮的 UI 结构
        button(
            container(
                column![
                    preview, // 布局预览图像
                    container(text(f.label()).size(11))
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)  // 标签居中显示
                ]
                .spacing(6), // 预览和标签之间的间距
            )
            .padding([8, 10]) // 容器内边距 [垂直, 水平]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(card_w))
        .height(Length::Fixed(card_h))
        .padding(0)
        // 动态设置按钮样式，根据激活状态和悬停状态调整外观
        .style(move |theme: &Theme, status| {
            let p = theme.extended_palette();
            // 检查按钮是否处于悬停状态
            let hovered = status == iced::widget::button::Status::Hovered;

            // 根据状态确定背景颜色
            let bg = if active {
                // 激活状态：使用主色调，透明度 12%
                p.primary.base.color.scale_alpha(0.12)
            } else if hovered {
                // 悬停状态：使用弱背景色
                p.background.weak.color
            } else {
                // 默认状态：使用基础背景色
                p.background.base.color
            };

            // 构建按钮样式
            iced::widget::button::Style {
                background: Some(bg.into()),
                border: Border {
                    width: if active { 2.0 } else { 1.0 }, // 激活状态边框更粗
                    color: if active { p.primary.base.color } else { p.background.weak.color },
                    radius: 12.0.into(), // 圆角半径
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        // 点击按钮时发送设置布局格式的消息
        .on_press(Message::MindMapTool(MindMapMessage::SetOrgChartLayoutFormat(f)))
    };

    // 构建并返回完整的布局选择器面板
    Some(
        container(
            column![
                // 标题：布局格式
                container(text("布局格式").size(12))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                // 布局格式卡片行
                row![card(formats[0]), card(formats[1])]
                    .spacing(card_gap)
                    .align_y(Alignment::Center), // 垂直居中对齐
            ]
            .spacing(8), // 标题和卡片行之间的间距
        )
        .width(Length::Fixed(desc_w))
        .into(),
    )
}
