//! 终端标签页重命名模块
//!
//! 本模块提供终端标签页标题编辑功能的更新逻辑，包括：
//! - 启动重命名编辑模式
//! - 处理编辑过程中的文本变化
//! - 保存或取消重命名操作
//!
//! 重命名流程采用内联编辑模式，用户在原标签标题位置直接编辑，
//! 支持提交保存或取消操作回滚到原标题。

use super::TerminalMessage;
use crate::app::{App, Message};
use iced::Task;

/// 处理终端标签页重命名相关的消息更新
///
/// 该函数是终端重命名功能的核心处理器，根据接收到的消息类型
/// 执行相应的状态更新操作。
///
/// # 参数
///
/// - `app`: 应用程序状态的可变引用，用于访问和修改终端相关数据
/// - `message`: 终端消息枚举，包含重命名操作的具体指令和数据
///
/// # 返回值
///
/// 返回 `Task<Message>`，当前所有重命名操作均不需要异步任务，
/// 始终返回 `Task::none()`
///
/// # 处理的消息类型
///
/// - `RenameStart(id)`: 启动指定标签页的重命名编辑模式
/// - `RenameChanged(id, value)`: 更新编辑中的标题文本
/// - `RenameSave(id)`: 保存编辑后的标题
/// - `RenameCancel(id)`: 取消编辑，丢弃修改
///
/// # 示例
///
/// ```ignore
/// // 在用户右键点击标签页选择"重命名"时触发
/// let task = update(&mut app, TerminalMessage::RenameStart(tab_id));
/// // 用户输入过程中持续触发
/// let task = update(&mut app, TerminalMessage::RenameChanged(tab_id, "新标题".to_string()));
/// // 用户按回车确认保存
/// let task = update(&mut app, TerminalMessage::RenameSave(tab_id));
/// ```
pub fn update(app: &mut App, message: TerminalMessage) -> Task<Message> {
    match message {
        // 启动重命名编辑模式
        TerminalMessage::RenameStart(id) => {
            // 关闭上下文菜单，避免与编辑界面冲突
            app.terminal.tab_context_menu_id = None;
            app.terminal.tab_context_menu_pos = None;

            // 选中目标标签页，确保其处于活动状态
            app.terminal.select_terminal(id);

            // 初始化编辑状态，将当前标题作为编辑初始值
            if let Some(tab) = app.terminal.tabs.iter_mut().find(|t| t.id == id) {
                tab.edit_title = Some(tab.title.clone());
            }
            Task::none()
        }

        // 处理编辑过程中的文本变化
        TerminalMessage::RenameChanged(id, v) => {
            // 更新临时编辑缓冲区的内容
            if let Some(tab) = app.terminal.tabs.iter_mut().find(|t| t.id == id) {
                tab.edit_title = Some(v);
            }
            Task::none()
        }

        // 保存编辑后的标题
        TerminalMessage::RenameSave(id) => {
            // 关闭上下文菜单
            app.terminal.tab_context_menu_id = None;
            app.terminal.tab_context_menu_pos = None;

            // 提取编辑缓冲区内容并应用
            if let Some(tab) = app.terminal.tabs.iter_mut().find(|t| t.id == id)
                && let Some(new) = tab.edit_title.take()
            {
                // 去除首尾空白字符
                let name = new.as_str().trim().to_owned();
                // 仅当标题非空时才更新，防止设置空白标题
                if !name.is_empty() {
                    tab.title = name;
                }
            }
            Task::none()
        }

        // 取消编辑操作
        TerminalMessage::RenameCancel(id) => {
            // 关闭上下文菜单
            app.terminal.tab_context_menu_id = None;
            app.terminal.tab_context_menu_pos = None;

            // 清除编辑状态，丢弃所有修改，保留原标题
            if let Some(tab) = app.terminal.tabs.iter_mut().find(|t| t.id == id) {
                tab.edit_title = None;
            }
            Task::none()
        }

        // 其他消息类型在本模块不处理，返回空任务
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "rename_tests.rs"]
mod rename_tests;
