//! 承载 SQL 工具的格式化、持久化与临时状态逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use crate::app::Message;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::config::save_sql_tool_content;
#[cfg(target_arch = "wasm32")]
use crate::app::config::save_sql_tool_content_async;
use iced::Task;

#[cfg(target_arch = "wasm32")]
/// 执行 save_sql_tool_content_task 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn save_sql_tool_content_task(content: String) -> Task<Message> {
    Task::perform(async move { save_sql_tool_content_async(&content).await }, |result| {
        if let Err(error) = result {
            tracing::warn!(target: "vw_desktop", error = %error, "failed to save sql tool content");
        }
        Message::None
    })
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 save_sql_tool_content_task 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn save_sql_tool_content_task(content: String) -> Task<Message> {
    save_sql_tool_content(&content);
    Task::none()
}
#[cfg(test)]
#[path = "persistence_tests.rs"]
mod persistence_tests;
