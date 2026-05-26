//! 括号布局格式选择器模块
//!
//! 本模块提供了括号布局格式的选择界面组件，用于在思维导图中选择不同的括号布局样式。
//! 主要功能包括：
//! - 提供可视化的布局格式预览卡片
//! - 支持用户交互切换不同的布局格式
//! - 根据当前选中状态动态更新UI样式
//!
//! # 布局格式类型
//! - `BraceRight`: 右向括号布局
//! - `BraceLeft`: 左向括号布局

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{BracketLayoutFormat, MindMapTab};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use super::super::super::super::previews::BracketLayoutFormatPreview;

/// 创建括号布局格式选择器UI组件
///
/// 该函数生成一个包含两种括号布局格式选项的选择器界面，每个选项都以卡片形式展示，
/// 包含布局预览图和格式标签。用户可以通过点击卡片来切换不同的布局格式。
///
/// # 参数
///
/// * `tab` - 思维导图标签页的当前状态引用，用于获取当前选中的布局格式
/// * `desc_w` - 描述区域的总宽度（像素），用于计算卡片的宽度和间距
///
/// # 返回值
///
/// 返回 `Option<Element<'static, Message>>`：
/// - `Some(Element)` - 成功创建的选择器UI元素
/// - `None` - 虽然当前实现总是返回 Some，保留 Option 类型以符合父模块的接口约定
///
/// # 布局结构
///
/// ```text
/// ┌─────────────────────────┐
/// │      布局格式            │  <- 标题栏
/// ├───────────┬─────────────┤
/// │  右向括号  │  左向括号    │  <- 两个格式卡片
/// │   [预览]  │   [预览]    │
/// │   标签    │    标签     │
/// └───────────┴─────────────┘
/// ```
///
/// # 样式规则
///
/// - **选中状态**：卡片背景使用主题色半透明，边框加粗为2px并使用主题色
/// - **悬停状态**：卡片背景使用弱背景色，边框保持1px
/// - **默认状态**：卡片背景使用基础背景色，边框保持1px
pub(in super::super) fn bracket_layout_picker(
    tab: &MindMapTab,
    desc_w: f32,
) -> Option<Element<'static, Message>> {
    // 定义可用的布局格式列表：右向括号和左向括号
    let formats = [BracketLayoutFormat::BraceRight, BracketLayoutFormat::BraceLeft];

    // 卡片之间的间距（像素）
    let card_gap = 10.0;
    // 计算每个卡片的宽度：总宽度减去间距后平均分配，最小宽度160像素
    let card_w = ((desc_w - card_gap * 1.0) / 2.0).max(160.0);
    // 卡片的固定高度（像素）
    let card_h = 66.0;

    // 创建单个布局格式卡片的闭包
    //
    // 该闭包为每种布局格式生成一个可点击的卡片，包含：
    // 1. 画布预览：使用 BracketLayoutFormatPreview 绘制布局格式的可视化预览
    // 2. 格式标签：显示布局格式的名称
    // 3. 交互样式：根据选中状态和悬停状态动态调整样式
    let card = |f: BracketLayoutFormat| {
        // 判断当前卡片是否为选中状态
        let active = tab.bracket_layout_format == f;

        // 创建布局格式的画布预览组件
        let preview: Element<'static, Message> = iced::widget::canvas(BracketLayoutFormatPreview {
            format: f,
            color: Color::from_rgba8(0, 0, 0, 0.68), // 使用半透明黑色绘制预览
        })
        .width(Length::Fill) // 宽度填充容器
        .height(Length::Fixed(34.0)) // 固定预览高度为34像素
        .into();

        // 构建卡片按钮组件
        button(
            container(
                column![
                    preview, // 布局预览画布
                    container(text(f.label()).size(11)) // 格式标签文本（11号字体）
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)  // 标签居中对齐
                ]
                .spacing(6), // 预览和标签之间的垂直间距
            )
            .padding([8, 10]) // 卡片内容内边距：上下8px，左右10px
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(card_w)) // 设置卡片固定宽度
        .height(Length::Fixed(card_h)) // 设置卡片固定高度
        .padding(0) // 按钮本身无额外内边距
        .style(move |theme: &Theme, status| {
            // 获取主题的扩展调色板，用于访问各种颜色定义
            let p = theme.extended_palette();
            // 检查按钮是否处于悬停状态
            let hovered = status == iced::widget::button::Status::Hovered;

            // 根据选中状态和悬停状态决定背景颜色
            let bg = if active {
                // 选中状态：使用主题色的12%透明度版本
                p.primary.base.color.scale_alpha(0.12)
            } else if hovered {
                // 悬停但未选中：使用弱背景色
                p.background.weak.color
            } else {
                // 默认状态：使用基础背景色
                p.background.base.color
            };

            // 构建按钮样式
            iced::widget::button::Style {
                background: Some(bg.into()), // 设置背景色
                border: Border {
                    width: if active { 2.0 } else { 1.0 }, // 选中时边框加粗
                    color: if active {
                        p.primary.base.color // 选中时使用主题色边框
                    } else {
                        p.background.weak.color // 未选中时使用弱背景色边框
                    },
                    radius: 12.0.into(), // 圆角半径12像素
                },
                text_color: theme.palette().text, // 使用主题文本颜色
                ..Default::default()
            }
        })
        // 点击卡片时触发设置布局格式的消息
        .on_press(Message::MindMapTool(MindMapMessage::SetBracketLayoutFormat(f)))
    };

    // 构建并返回完整的选择器容器组件
    Some(
        container(
            column![
                // 标题栏：显示"布局格式"
                container(text("布局格式").size(12))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                // 卡片行：包含两个布局格式卡片
                row![card(formats[0]), card(formats[1])]
                    .spacing(card_gap) // 卡片之间的间距
                    .align_y(Alignment::Center), // 卡片垂直居中对齐
            ]
            .spacing(8), // 标题栏和卡片行之间的间距
        )
        .width(Length::Fixed(desc_w)) // 容器宽度与描述区域宽度一致
        .into(),
    )
}
