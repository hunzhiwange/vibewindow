//! Dify Workflow 后端执行模块。
//!
//! 当前实现面向桌面端已能编辑/加载的 Dify YAML，提供最小可用的本地执行闭环。

mod code_runner;
mod conditions;
mod model;
mod runner;
mod template;
mod variables;

pub use runner::{WorkflowRuntime, run_workflow};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
