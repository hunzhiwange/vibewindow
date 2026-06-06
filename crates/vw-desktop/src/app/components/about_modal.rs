//! 关于模态框组件
//!
//! 本模块提供应用"关于"对话框的视图实现，展示 Vibe Window 应用的基本信息。
//!
//! # 功能概述
//!
//! - 显示应用名称和描述
//! - 展示核心功能列表
//! - 显示开发者信息
//! - 提供复制应用信息和关闭按钮
//! - 半透明遮罩层背景
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::app::components::about_modal;
//!
//! // 在视图层渲染关于模态框
//! let modal = about_modal::view(app);
//! ```

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_close_button, settings_modal_card,
    settings_modal_overlay, settings_muted_text_style, settings_section_card, settings_value_badge,
};
use crate::app::{App, Message, message};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};
use std::hash::{Hash, Hasher};

const ABOUT_SUBTITLE: &str = "设计、代码、预览与智能体协作整合在同一桌面工作台。";
const PRODUCT_POSITION_TITLE: &str = "产品定位";
const PRODUCT_POSITION_DESCRIPTION: &str =
    "围绕项目管理、代码编辑、UI 设计、预览和自动化任务组织一个统一工作界面。";
const AUTHOR_INFO_TITLE: &str = "作者信息";
const AUTHOR_INFO_DESCRIPTION: &str = "开发者：刘祥敏。";

fn about_copy_text() -> String {
    format!(
        "Vibe Window\n{ABOUT_SUBTITLE}\n\n{PRODUCT_POSITION_TITLE}\n{PRODUCT_POSITION_DESCRIPTION}\n\n{AUTHOR_INFO_TITLE}\n{AUTHOR_INFO_DESCRIPTION}"
    )
}

fn hash_copy_text(content: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// 创建关于模态框视图
///
/// 构建并返回一个完整的关于对话框组件，包含以下元素：
/// - 卡片容器：承载所有内容，带有圆角边框和阴影效果
/// - 应用信息：名称、描述和功能列表
/// - 开发者信息：显示开发者名称
/// - 复制按钮：复制当前应用信息，并在 2 秒内显示完成状态
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
/// let about_view = view(app);
/// // 将 about_view 添加到应用的主视图中
/// ```
pub fn view<'a>(app: &'a App) -> Element<'a, Message> {
    let close_message = Message::View(message::ViewMessage::ToggleAboutModal);
    let copy_text = about_copy_text();
    let copy_label = if app.last_copied_code_hash == Some(hash_copy_text(&copy_text)) {
        "✓"
    } else {
        "复制应用信息"
    };
    let copy_button = button(text(copy_label).size(13))
        .on_press(Message::CopyCode(copy_text))
        .padding([10, 18])
        .width(Length::Fixed(132.0))
        .style(rounded_action_btn_style);

    let card = settings_modal_card(
        column![
            row![
                column![
                    text("关于 Vibe Window").size(24),
                    text(ABOUT_SUBTITLE).size(12).style(settings_muted_text_style),
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
            settings_section_card(PRODUCT_POSITION_TITLE, PRODUCT_POSITION_DESCRIPTION),
            settings_section_card(AUTHOR_INFO_TITLE, AUTHOR_INFO_DESCRIPTION),
            container(
                row![
                    copy_button,
                    button(text("关闭").size(13))
                        .on_press(close_message.clone())
                        .padding([10, 18])
                        .style(primary_action_btn_style),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
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
