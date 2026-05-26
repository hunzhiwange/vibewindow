//! 思维导图布局格式选择器视图模块
//!
//! 本模块提供了思维导图布局格式的可视化选择界面组件。
//! 用户可以通过该界面在三种布局格式之间进行切换：
//! - 右对齐布局
//! - 左对齐布局
//! - 双向布局
//!
//! 主要功能：
//! - 渲染布局格式预览卡片
//! - 提供交互式布局选择按钮
//! - 根据当前选中的布局格式更新视图状态

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapLayoutFormat, MindMapTab};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::super::super::super::previews::LayoutFormatPreview;

/// 创建思维导图布局格式选择器视图
///
/// 该函数渲染一个包含三个布局格式选项卡片的界面组件，
/// 每个卡片显示布局预览和对应的格式名称。用户点击卡片可切换布局格式。
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态引用，包含当前选中的布局格式等信息
/// * `desc_w` - 描述区域的宽度（像素），用于计算每个卡片的宽度
///
/// # 返回值
///
/// 返回 `Option<Element<'static, Message>>`：
/// - `Some(Element)` - 包含布局选择器界面的UI元素
/// - `None` - 如果不满足渲染条件（当前实现总是返回 Some）
///
/// # 布局格式
///
/// 提供三种布局格式选项：
/// 1. `RightAligned` - 右对齐布局，所有节点向右侧展开
/// 2. `LeftAligned` - 左对齐布局，所有节点向左侧展开
/// 3. `Bidirectional` - 双向布局，节点向两侧对称展开
///
/// # 示例
///
/// ```ignore
/// let tab = MindMapTab::default();
/// let element = mindmap_layout_picker(&tab, 400.0);
/// // element 将包含一个宽度为 400 像素的布局选择器界面
/// ```
pub(in super::super) fn mindmap_layout_picker(
    tab: &MindMapTab,
    desc_w: f32,
) -> Option<Element<'static, Message>> {
    // 定义可用的布局格式数组
    let formats = [
        MindMapLayoutFormat::RightAligned,
        MindMapLayoutFormat::LeftAligned,
        MindMapLayoutFormat::Bidirectional,
    ];

    // 计算卡片布局参数
    let card_gap = 10.0; // 卡片之间的间距
    // 计算每个卡片的宽度：(总宽度 - 2个间距) / 3，最小为 120 像素
    let card_w = ((desc_w - card_gap * 2.0) / 3.0).max(120.0);
    let card_h = 66.0; // 卡片固定高度

    // 卡片构建闭包：为指定的布局格式创建交互式卡片组件
    let card = |f: MindMapLayoutFormat| {
        // 判断当前卡片对应的布局格式是否为激活状态
        let active = tab.layout_format == f;

        // 创建布局格式预览画布
        // 使用 Canvas 组件渲染布局格式的可视化预览
        let preview: Element<'static, Message> = iced::widget::canvas(LayoutFormatPreview {
            format: f,
            color: Color::from_rgba8(0, 0, 0, 0.68), // 预览颜色：半透明黑色
        })
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .into();

        // 构建卡片按钮
        // 卡片包含：布局预览 + 格式名称标签
        button(
            container(
                column![
                    preview, // 布局格式可视化预览
                    container(text(f.label()).size(11)) // 布局格式名称标签（11px 字体）
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                ]
                .spacing(6), // 预览和标签之间的垂直间距
            )
            .padding([8, 10]) // 卡片内边距：[垂直, 水平]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(card_w))
        .height(Length::Fixed(card_h))
        .padding(0)
        // 设置按钮样式：根据激活状态和悬停状态动态调整
        .style(move |theme: &Theme, status| {
            let p = theme.extended_palette();
            let hovered = status == iced::widget::button::Status::Hovered;

            // 根据状态确定背景颜色
            let bg = if active {
                // 激活状态：主色调 12% 透明度
                p.primary.base.color.scale_alpha(0.12)
            } else if hovered {
                // 悬停状态：弱背景色
                p.background.weak.color
            } else {
                // 默认状态：基础背景色
                p.background.base.color
            };

            iced::widget::button::Style {
                background: Some(bg.into()),
                border: Border {
                    width: if active { 2.0 } else { 1.0 }, // 激活时边框更粗
                    color: if active {
                        p.primary.base.color // 激活时使用主色调边框
                    } else {
                        p.background.weak.color // 默认使用弱背景色边框
                    },
                    radius: 12.0.into(), // 圆角半径
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        // 点击事件：发送设置布局格式的消息
        .on_press(Message::MindMapTool(MindMapMessage::SetLayoutFormat(f)))
    };

    // 构建并返回完整的布局选择器界面
    Some(
        container(
            column![
                // 标题：居中显示的"布局格式"文本
                container(text("布局格式").size(12))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                // 三个布局格式卡片，水平排列
                row![card(formats[0]), card(formats[1]), card(formats[2])]
                    .spacing(card_gap)
                    .align_y(Alignment::Center),
            ]
            .spacing(8), // 标题和卡片行之间的垂直间距
        )
        .width(Length::Fixed(desc_w))
        .into(),
    )
}
