//! 设计编辑器消息处理模块
//!
//! 本模块负责处理设计视图中与编辑器相关的所有消息。
//! 为降低单文件复杂度，内部实现按提示词构造、文档解析、画布同步、日志与任务执行拆分到
//! `editor/` 子模块中，外部仍保持原有 `editor::update` 入口不变。

use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::DesignGenerationPlan;

#[path = "editor/canvas.rs"]
mod canvas;
#[path = "editor/handlers.rs"]
mod handlers;
#[path = "editor/logging.rs"]
mod logging;
#[path = "editor/parser.rs"]
mod parser;
#[path = "editor/prompts.rs"]
mod prompts;
#[path = "editor/tasks.rs"]
mod tasks;
#[cfg(test)]
#[path = "editor/tests.rs"]
mod tests;

#[derive(Debug, Clone)]
pub struct DesignPlanExecutionResult {
    pub plan: DesignGenerationPlan,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DesignModuleExecutionResult {
    pub doc: DesignDoc,
    pub logs: Vec<String>,
}

pub use handlers::update;

#[cfg(test)]
#[path = "editor/canvas_tests.rs"]
mod canvas_tests;

#[cfg(test)]
#[path = "editor/handlers_tests.rs"]
mod handlers_tests;

#[cfg(test)]
#[path = "editor/logging_tests.rs"]
mod logging_tests;

#[cfg(test)]
#[path = "editor/parser_tests.rs"]
mod parser_tests;

#[cfg(test)]
#[path = "editor/prompts_tests.rs"]
mod prompts_tests;

#[cfg(test)]
#[path = "editor/tasks_tests.rs"]
mod tasks_tests;
