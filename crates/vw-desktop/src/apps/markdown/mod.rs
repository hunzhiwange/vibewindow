//! Markdown 应用入口模块，负责暴露应用状态、消息和视图装配能力。

use crate::app::{App, Message};
use iced::{Element, Task};

/// 构建或更新 view 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn view(app: &App) -> Element<'_, Message> {
    crate::app::views::markdown_tool::view(app)
}

/// 构建或更新 update 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn update(app: &mut App, message: crate::app::message::MarkdownToolMessage) -> Task<Message> {
    crate::app::message::markdown_tool::update(app, message)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
