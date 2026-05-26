//! 图表类型选择面板模块
//!
//! 本模块提供思维导图编辑器中的图表类型选择功能。用户可以在此面板中
//! 选择不同的图表类型（如思维导图、组织架构图、鱼骨图等），并根据
//! 所选类型显示对应的布局选项。
//!
//! # 主要功能
//!
//! - 展示所有可用的图表类型按钮列表
//! - 高亮显示当前选中的图表类型
//! - 根据图表类型动态显示对应的布局选择器
//! - 提供统一的视觉样式和交互体验
//!
//! # 支持的图表类型
//!
//! - `MindMap` - 思维导图
//! - `OrgChart` - 组织架构图
//! - `Fishbone` - 鱼骨图（因果分析图）
//! - `Timeline` - 时间线图
//! - `Tree` - 树状图
//! - `Bracket` - 括号图

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapDiagramType, MindMapTab};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::layout_picker::{
    bracket_layout_picker, fishbone_layout_picker, mindmap_layout_picker, org_chart_layout_picker,
    timeline_layout_picker, tree_layout_picker,
};

/// 构建图表类型选择面板
///
/// 该函数创建一个包含图表类型选择按钮和布局选择器的面板。
/// 面板采用左右布局：左侧为类型按钮列表，右侧为对应的布局选项。
///
/// # 参数
///
/// - `tab`: 当前思维导图标签页的状态引用，包含当前选中的图表类型等信息
/// - `panel_w`: 面板的总宽度（像素）
/// - `_panel_h`: 面板的总高度（像素），当前未使用，保留供未来扩展
///
/// # 返回值
///
/// 返回一个 `Element<'static, Message>`，表示可渲染的 UI 元素树。
/// 该元素包含完整的图表类型选择功能。
///
/// # UI 结构
///
/// ```text
/// ┌─────────────────────────────────────┐
/// │ 图表类型                             │
/// ├──────────────┬──────────────────────┤
/// │ [思维导图]   │                       │
/// │ [组织架构图] │   布局选择器          │
/// │ [鱼骨图]     │   (根据类型动态显示)   │
/// │ [时间线图]   │                       │
/// │ [树状图]     │                       │
/// │ [括号图]     │                       │
/// └──────────────┴──────────────────────┘
/// ```
///
/// # 示例
///
/// ```ignore
/// let panel = diagram_type_panel(&tab, 400.0, 300.0);
/// // 将 panel 添加到更大的 UI 布局中
/// ```
pub(in super::super) fn diagram_type_panel(
    tab: &MindMapTab,
    panel_w: f32,
    _panel_h: f32,
) -> Element<'static, Message> {
    // 定义所有支持的图表类型，按显示顺序排列
    let types = [
        MindMapDiagramType::MindMap,
        MindMapDiagramType::OrgChart,
        MindMapDiagramType::Fishbone,
        MindMapDiagramType::Timeline,
        MindMapDiagramType::Tree,
        MindMapDiagramType::Bracket,
    ];

    // 创建单个类型按钮的闭包
    // 根据是否选中状态应用不同的样式
    let type_btn = |t: MindMapDiagramType| {
        // 判断当前按钮是否为选中状态
        let active = tab.diagram_type == t;
        button(
            container(text(t.label()).size(12))
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .padding([8, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            // 获取主题的扩展调色板
            let p = theme.extended_palette();
            // 检测鼠标是否悬停在按钮上
            let hovered = status == iced::widget::button::Status::Hovered;

            // 根据状态确定背景颜色
            let bg = if active {
                // 选中状态：使用主色调的 10% 透明度
                p.primary.base.color.scale_alpha(0.10)
            } else if hovered {
                // 悬停状态：使用弱背景色
                p.background.weak.color
            } else {
                // 默认状态：透明背景
                Color::TRANSPARENT
            };

            // 构建按钮样式
            iced::widget::button::Style {
                background: Some(bg.into()),
                border: Border {
                    // 选中时边框加粗为 2.0，否则为 1.0
                    width: if active { 2.0 } else { 1.0 },
                    // 选中时使用主色调边框，否则使用强背景色
                    color: if active { p.primary.base.color } else { p.background.strong.color },
                    radius: 10.0.into(),
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        // 点击按钮时发送设置图表类型的消息
        .on_press(Message::MindMapTool(MindMapMessage::SetDiagramType(t)))
    };

    // 构建类型按钮列表
    // 固定宽度为 140 像素，按钮之间间距 4 像素
    let mut type_list = column![].spacing(4).width(Length::Fixed(140.0));
    for t in types {
        type_list = type_list.push(type_btn(t));
    }

    // 将类型列表包装在容器中
    let type_list = container(type_list);

    // 计算布局选择器的宽度
    // 总宽度 - 左右内边距(12*2) - 间距(10) - 类型列表宽度(140)
    // 最小宽度为 260 像素，确保布局选择器有足够的显示空间
    let desc_w = (panel_w - 12.0 * 2.0 - 10.0 - 140.0).max(260.0);

    // 根据当前选中的图表类型，获取对应的布局选择器
    let layout_picker = match tab.diagram_type {
        MindMapDiagramType::MindMap => mindmap_layout_picker(tab, desc_w),
        MindMapDiagramType::OrgChart => org_chart_layout_picker(tab, desc_w),
        MindMapDiagramType::Fishbone => fishbone_layout_picker(tab, desc_w),
        MindMapDiagramType::Timeline => timeline_layout_picker(tab, desc_w),
        MindMapDiagramType::Tree => tree_layout_picker(tab, desc_w),
        MindMapDiagramType::Bracket => bracket_layout_picker(tab, desc_w),
    };

    // 构建主要内容区域
    // 如果有布局选择器，则采用左右布局；否则仅显示类型列表
    let content: Element<'static, Message> = if let Some(layout_picker) = layout_picker {
        row![type_list, layout_picker].spacing(10).align_y(Alignment::Start).into()
    } else {
        type_list.into()
    };

    // 构建最终的面板容器
    container(
        column![row![text("图表类型").size(13),].align_y(Alignment::Center), content,]
            .spacing(10)
            .padding(12),
    )
    .width(Length::Fixed(panel_w))
    .style(|theme: &Theme| {
        // 获取主题调色板
        let p = theme.extended_palette();
        // 应用面板容器样式
        iced::widget::container::Style {
            // 使用基础背景色
            background: Some(p.background.base.color.into()),
            // 添加细边框和圆角
            border: Border { width: 1.0, color: p.background.strong.color, radius: 12.0.into() },
            // 添加阴影效果，增强视觉层次感
            shadow: iced::Shadow {
                // 阴影颜色：黑色带 12% 透明度
                color: Color::BLACK.scale_alpha(0.12),
                // 阴影偏移：向下 6 像素
                offset: iced::Vector::new(0.0, 6.0),
                // 模糊半径：18 像素
                blur_radius: 18.0,
            },
            ..Default::default()
        }
    })
    .into()
}
