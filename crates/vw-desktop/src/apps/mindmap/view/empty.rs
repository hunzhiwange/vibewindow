//! 思维导图空白状态视图模块
//!
//! 本模块提供思维导图应用的初始空白状态界面渲染功能。
//! 当用户首次打开思维导图应用或未打开任何文件时，显示此界面。
//!
//! # 功能
//!
//! - 提供新建空白思维导图的入口
//! - 提供打开已有 Markdown 文件的入口
//! - 友好的引导用户开始使用思维导图功能

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

/// 渲染思维导图空白状态界面
///
/// 此函数创建并返回思维导图应用的空白状态视图元素。
/// 界面包含标题、说明文字以及两个操作按钮（新建空白、打开文件）。
///
/// # 返回值
///
/// 返回一个 `Element<'static, Message>` 类型的 UI 元素，
/// 该元素可以被 Iced 框架渲染并响应用户交互。
///
/// # UI 布局
///
/// ```text
/// ┌────────────────────────────────────┐
/// │           思维导图                   │
/// │  新建一个空白思维导图，或打开已有     │
/// │       Markdown 文件                 │
/// │   [新建空白]  [打开文件]            │
/// └────────────────────────────────────┘
/// ```
///
/// # 交互
///
/// - 点击"新建空白"按钮：触发 `MindMapMessage::New` 消息
/// - 点击"打开文件"按钮：触发 `MindMapMessage::Open` 消息
pub(super) fn render() -> Element<'static, Message> {
    let content = column![
        text("思维导图").size(20),
        text("新建一个空白思维导图，或打开已有 Markdown 文件").size(14),
        row![
            button("新建空白")
                .on_press(Message::MindMapTool(MindMapMessage::New))
                .style(button::primary)
                .padding([6, 16]),
            button("打开文件")
                .on_press(Message::MindMapTool(MindMapMessage::Open))
                .style(button::secondary)
                .padding([6, 16]),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(12)
    .align_x(iced::alignment::Horizontal::Center);

    container(content).center(Length::Fill).into()
}
