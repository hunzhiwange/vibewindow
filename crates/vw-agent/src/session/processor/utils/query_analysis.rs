//! 用户查询分析辅助逻辑，负责判断是否可以根据工具活动自动完成待办状态。

use super::super::types::ToolSessionState;

/// 执行 should_try_auto_complete_todos 操作，并返回调用方需要的结果。
pub(crate) fn should_try_auto_complete_todos(text: &str, tool_state: &ToolSessionState) -> bool {
    if tool_state.non_todo_tool_runs == 0 {
        return false;
    }
    let t = text.trim();
    if t.len() < 40 {
        return false;
    }
    let lower = t.to_lowercase();
    if lower.contains("todo") || t.contains("待办") || t.contains("任务未完成") {
        return false;
    }
    true
}
#[cfg(test)]
#[path = "query_analysis_tests.rs"]
mod query_analysis_tests;
