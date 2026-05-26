//! 时间轴布局选择器模块
//!
//! 本模块提供时间轴布局格式的可视化选择界面，允许用户在三种布局格式之间切换：
//! - 上下布局：节点按时间顺序上下交替排列
//! - 全部向上：所有节点向上延伸排列
//! - 全部向下：所有节点向下延伸排列
//!
//! 该模块的主要职责是渲染一个包含三个可点击卡片的界面，
//! 每个卡片显示对应布局格式的预览图和标签。

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapTab, TimelineLayoutFormat};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::super::super::super::previews::TimelineLayoutFormatPreview;

/// 创建时间轴布局选择器界面
///
/// 该函数生成一个包含三种时间轴布局格式选项的 UI 组件，
/// 用户可以通过点击卡片来切换不同的布局格式。
///
/// # 参数
///
/// - `tab`: 当前思维导图标签页的状态引用，用于获取当前选中的布局格式
/// - `desc_w`: 描述区域的宽度（像素），用于计算卡片的尺寸
///
/// # 返回值
///
/// 返回 `Some(Element)` 包含完整的布局选择器界面，
/// 或在构建失败时返回 `None`（当前实现始终返回 `Some`）
///
/// # 示例
///
/// ```ignore
/// let element = timeline_layout_picker(&mindmap_tab, 400.0);
/// // element 可直接用于 iced UI 树中
/// ```
///
/// # 布局结构
///
/// ```text
/// ┌─────────────────────────────────────┐
/// │          布局格式                    │
/// ├─────────┬─────────┬─────────────────┤
/// │ [预览1] │ [预览2] │ [预览3]         │
/// │  上下   │ 全部向上 │ 全部向下        │
/// └─────────┴─────────┴─────────────────┘
/// ```
pub(in super::super) fn timeline_layout_picker(
    tab: &MindMapTab,
    desc_w: f32,
) -> Option<Element<'static, Message>> {
    // 定义三种可用的时间轴布局格式
    let formats =
        [TimelineLayoutFormat::UpDown, TimelineLayoutFormat::AllUp, TimelineLayoutFormat::AllDown];

    // 卡片之间的间距（像素）
    let card_gap = 10.0;
    // 计算每个卡片的宽度：总宽度减去两个间距后三等分，最小值 120 像素
    let card_w = ((desc_w - card_gap * 2.0) / 3.0).max(120.0);
    // 卡片的固定高度（像素）
    let card_h = 66.0;

    // 卡片生成闭包：为每个布局格式创建可点击的卡片组件
    let card = |f: TimelineLayoutFormat| {
        // 判断当前卡片是否为选中状态
        let active = tab.timeline_layout_format == f;

        // 创建布局格式预览图（使用 Canvas 渲染）
        let preview: Element<'static, Message> =
            iced::widget::canvas(TimelineLayoutFormatPreview {
                format: f,
                color: Color::from_rgba8(0, 0, 0, 0.68), // 预览图使用 68% 不透明度的黑色
            })
            .width(Length::Fill)
            .height(Length::Fixed(34.0))
            .into();

        // 构建卡片按钮
        button(
            container(
                column![
                    preview, // 布局预览图
                    container(text(f.label()).size(11)) // 布局标签文本（11px 字号）
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                ]
                .spacing(6), // 预览图与标签间距 6px
            )
            .padding([8, 10]) // 上下 8px，左右 10px 内边距
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(card_w))
        .height(Length::Fixed(card_h))
        .padding(0)
        // 动态样式：根据选中状态和悬停状态调整外观
        .style(move |theme: &Theme, status| {
            let p = theme.extended_palette();
            let hovered = status == iced::widget::button::Status::Hovered;

            // 背景色逻辑：
            // - 选中：主色调 12% 不透明度
            // - 悬停：弱背景色
            // - 默认：基础背景色
            let bg = if active {
                p.primary.base.color.scale_alpha(0.12)
            } else if hovered {
                p.background.weak.color
            } else {
                p.background.base.color
            };

            iced::widget::button::Style {
                background: Some(bg.into()),
                border: Border {
                    width: if active { 2.0 } else { 1.0 }, // 选中时边框更粗
                    color: if active { p.primary.base.color } else { p.background.weak.color }, // 选中时使用主色调边框
                    radius: 12.0.into(), // 圆角 12px
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        // 点击时发送切换布局格式的消息
        .on_press(Message::MindMapTool(MindMapMessage::SetTimelineLayoutFormat(f)))
    };

    // 构建并返回完整的选择器容器
    Some(
        container(
            column![
                // 标题：居中显示"布局格式"
                container(text("布局格式").size(12))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                // 三个布局格式卡片，水平排列
                row![card(formats[0]), card(formats[1]), card(formats[2])]
                    .spacing(card_gap)
                    .align_y(Alignment::Center),
            ]
            .spacing(8), // 标题与卡片行间距 8px
        )
        .width(Length::Fixed(desc_w))
        .into(),
    )
}
