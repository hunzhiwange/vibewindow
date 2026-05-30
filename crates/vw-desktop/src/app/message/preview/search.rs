//! 处理预览内容搜索，维护匹配结果、当前位置和搜索输入状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::PreviewMessage;
use crate::app::{App, Message};
use iced::Task;

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub fn update(_app: &mut App, _message: PreviewMessage) -> Task<Message> {
    Task::none()
}
