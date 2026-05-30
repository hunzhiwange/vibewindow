//! 关于模态框组件
//!
//! 本模块提供应用"关于"对话框的视图实现，展示 Vibe Window 应用的基本信息。
//!
//! # 功能概述
//!
//! - 显示应用名称和描述
//! - 展示核心功能列表
//! - 显示开发者信息
//! - 提供关闭按钮
//! - 半透明遮罩层背景
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::app::components::about_modal;
//!
//! // 在视图层渲染关于模态框
//! let modal = about_modal::view();
//! ```

use crate::app::components::system_settings_common::{
    primary_action_btn_style, settings_close_button, settings_modal_card, settings_modal_overlay,
    settings_muted_text_style, settings_section_card, settings_value_badge,
};
use crate::app::{Message, message};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

/// 创建关于模态框视图
///
/// 构建并返回一个完整的关于对话框组件，包含以下元素：
/// - 卡片容器：承载所有内容，带有圆角边框和阴影效果
/// - 应用信息：名称、描述和功能列表
/// - 开发者信息：显示开发者名称
/// - 关闭按钮：触发模态框关闭操作
/// - 遮罩层：半透明黑色背景，点击可关闭模态框
///
/// # 返回值
///
/// 返回一个 `Element<'a, Message>` 类型的 UI 元素，可被直接集成到
/// 应用的视图层次结构中。
///
/// # 交互行为
///
/// - 点击"关闭"按钮：发送 `ToggleAboutModal` 消息，关闭模态框
/// - 点击遮罩层：同样发送 `ToggleAboutModal` 消息，关闭模态框
///
/// # 示例
///
/// ```ignore
/// let about_view = view();
/// // 将 about_view 添加到应用的主视图中
/// ```
pub fn view<'a>() -> Element<'a, Message> {
    let close_message = Message::View(message::ViewMessage::ToggleAboutModal);
    let card = settings_modal_card(
        column![
            row![
                column![
                    text("关于 Vibe Window").size(24),
                    text("设计、代码、预览与智能体协作整合在同一桌面工作台。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(4)
                .width(Length::Fill),
                settings_close_button(close_message.clone()),
            ]
            .align_y(Alignment::Start),
            row![
                settings_value_badge("设计与开发一体"),
                settings_value_badge("本地桌面端"),
                settings_value_badge("AI 辅助工作流"),
            ]
            .spacing(8),
            settings_section_card(
                "产品定位",
                "围绕项目管理、代码编辑、UI 设计、预览和自动化任务组织一个统一工作界面。",
            ),
            settings_section_card("作者信息", "开发者：刘祥敏。",),
            container(
                button(text("关闭").size(13))
                    .on_press(close_message.clone())
                    .padding([10, 18])
                    .style(primary_action_btn_style),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(16),
    )
    .width(Length::Fixed(560.0));

    settings_modal_overlay(None, close_message, card)
}
#[cfg(test)]
#[path = "about_modal_tests.rs"]
mod about_modal_tests;
