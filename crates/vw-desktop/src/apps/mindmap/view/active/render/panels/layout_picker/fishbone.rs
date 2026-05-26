//! 鱼骨图布局选择器模块
//!
//! 本模块提供了鱼骨图（因果图）布局格式的可视化选择器组件。
//! 用户可以通过该组件选择鱼骨图的朝向（鱼头朝左或朝右）。
//!
//! # 功能
//!
//! - 展示不同的鱼骨图布局预览
//! - 高亮显示当前选中的布局格式
//! - 响应用户点击切换布局格式
//!
//! # 使用场景
//!
//! 该组件通常在思维导图编辑面板中使用，让用户直观地选择鱼骨图的布局方向。

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{FishboneLayoutFormat, MindMapTab};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::super::super::super::previews::FishboneLayoutFormatPreview;

/// 创建鱼骨图布局选择器
///
/// 该函数构建一个包含布局预览的 UI 组件，用户可以点击选择不同的鱼骨图布局格式。
/// 组件会显示两个布局卡片（头朝右和头朝左），并高亮显示当前激活的布局。
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态引用，包含当前选中的布局格式
/// * `desc_w` - 描述区域的宽度，用于计算卡片布局
///
/// # 返回值
///
/// 返回 `Option<Element<'static, Message>>`：
/// - `Some(Element)` - 包含布局选择器的 UI 元素
/// - `None` - 如果创建失败（当前实现总是返回 Some）
///
/// # 布局结构
///
/// ```text
/// ┌─────────────────────────────────┐
/// │         布局格式                 │  <- 标题
/// ├─────────────┬───────────────────┤
/// │  [预览图]   │    [预览图]        │  <- 布局卡片
/// │  头朝右     │    头朝左          │  <- 格式标签
/// └─────────────┴───────────────────┘
/// ```
///
/// # 示例
///
/// ```rust,ignore
/// let tab = MindMapTab::default();
/// let element = fishbone_layout_picker(&tab, 300.0);
/// // 返回一个包含两个布局选项的 UI 组件
/// ```
pub(in super::super) fn fishbone_layout_picker(
    tab: &MindMapTab,
    desc_w: f32,
) -> Option<Element<'static, Message>> {
    // 定义可选的鱼骨图布局格式：头朝右和头朝左
    let formats = [FishboneLayoutFormat::HeadRight, FishboneLayoutFormat::HeadLeft];

    // 卡片之间的间距
    let card_gap = 10.0;

    // 计算每个卡片的宽度：减去间距后平分，最小宽度为 160.0 像素
    let card_w = ((desc_w - card_gap * 1.0) / 2.0).max(160.0);

    // 卡片高度固定为 66.0 像素
    let card_h = 66.0;

    // 创建单个布局格式选择卡片
    //
    // 该闭包为指定的布局格式创建一个可点击的卡片组件，
    // 包含布局预览图和格式标签。
    //
    // # 参数
    //
    // * `f` - 鱼骨图布局格式（头朝右或头朝左）
    //
    // # 返回值
    //
    // 返回一个按钮元素，点击后会触发布局格式切换
    let card = |f: FishboneLayoutFormat| {
        // 判断当前格式是否被激活
        let active = tab.fishbone_layout_format == f;

        // 创建布局预览图（使用 Canvas 绘制）
        let preview: Element<'static, Message> =
            iced::widget::canvas(FishboneLayoutFormatPreview {
                format: f,
                color: Color::from_rgba8(0, 0, 0, 0.68), // 使用半透明黑色绘制预览
            })
            .width(Length::Fill) // 宽度填满容器
            .height(Length::Fixed(34.0)) // 固定高度 34 像素
            .into();

        // 构建按钮容器
        button(
            container(
                column![
                    preview, // 布局预览图
                    container(text(f.label()).size(11)) // 格式标签文本
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)  // 标签居中对齐
                ]
                .spacing(6), // 预览图和标签之间的间距
            )
            .padding([8, 10]) // 内边距：上下 8px，左右 10px
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(card_w))
        .height(Length::Fixed(card_h))
        .padding(0)
        // 设置按钮样式
        .style(move |theme: &Theme, status| {
            let p = theme.extended_palette();

            // 判断按钮是否处于悬停状态
            let hovered = status == iced::widget::button::Status::Hovered;

            // 根据激活状态和悬停状态决定背景颜色
            let bg = if active {
                // 激活状态：使用主色调的半透明版本（透明度 12%）
                p.primary.base.color.scale_alpha(0.12)
            } else if hovered {
                // 悬停但未激活：使用弱背景色
                p.background.weak.color
            } else {
                // 默认状态：使用基础背景色
                p.background.base.color
            };

            // 返回按钮样式配置
            iced::widget::button::Style {
                background: Some(bg.into()),
                border: Border {
                    width: if active { 2.0 } else { 1.0 }, // 激活状态边框更粗
                    color: if active {
                        p.primary.base.color // 激活状态使用主色调边框
                    } else {
                        p.background.weak.color // 默认使用弱背景色边框
                    },
                    radius: 12.0.into(), // 圆角半径 12 像素
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        // 点击按钮时发送布局格式切换消息
        .on_press(Message::MindMapTool(MindMapMessage::SetFishboneLayoutFormat(f)))
    };

    // 构建完整的布局选择器容器
    Some(
        container(
            column![
                // 标题：布局格式
                container(text("布局格式").size(12))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                // 布局卡片行：包含两个布局选项
                row![card(formats[0]), card(formats[1])]
                    .spacing(card_gap) // 卡片之间的间距
                    .align_y(Alignment::Center), // 垂直居中对齐
            ]
            .spacing(8), // 标题和卡片行之间的间距
        )
        .width(Length::Fixed(desc_w))
        .into(),
    )
}
