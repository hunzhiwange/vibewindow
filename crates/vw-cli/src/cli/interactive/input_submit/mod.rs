//! CLI 交互模式输入提交模块
//!
//! 本模块是 CLI 交互模式下用户输入处理的入口点，负责协调用户输入的完整处理流程。
//!
//! ## 模块结构
//!
//! - [`commands`] - 内联命令处理（如 `/help`、`/quit`、`/clear` 等）
//! - [`flow`] - 主提交流程编排，处理用户输入并路由到相应处理器
//!
//! ## 主要功能
//!
//! 1. **输入路由**：根据输入类型和当前状态，将用户输入分发到合适的处理器
//! 2. **命令处理**：识别并执行以 `/` 开头的内联命令
//! 3. **会话管理**：协调会话历史、转录记录和统计信息的更新
//! 4. **流式响应**：处理和渲染来自 Agent 的流式响应
//!
//! ## 架构位置
//!
//! ```text
//! cli/interactive/
//! ├── input/              -- 输入处理（键盘事件、历史等）
//! ├── input_submit/       <-- 本模块：输入提交和流程编排
//! │   ├── mod.rs          -- 模块入口（本文件）
//! │   ├── commands.rs     -- 命令处理
//! │   └── flow.rs         -- 主流程
//! └── loop_.rs            -- 主事件循环
//! ```
//!
//! ## 导出项
//!
//! - [`SubmitOutcome`](flow::SubmitOutcome) - 提交处理结果（继续或退出）
//! - [`handle_submit_result`](flow::handle_submit_result) - 核心提交处理函数

mod commands;
#[cfg(test)]
#[path = "commands_tests.rs"]
mod commands_tests;
mod flow;
#[cfg(test)]
#[path = "flow_tests.rs"]
mod flow_tests;

pub(crate) use flow::{SubmitOutcome, handle_submit_result};
