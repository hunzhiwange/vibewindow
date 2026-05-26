//! 新一代 CLI TUI 入口模块。
//!
//! 当前阶段已覆盖 Phase 1 到 Phase 3 的基础骨架：
//! - 为 CLI 建立直连 gateway client 的依赖边界
//! - 落地内部模型层、状态层与 snapshot round-trip
//! - 新建 tui_v2 根入口、controller/renderer 分层与 fullscreen layout slots
//!
//! 该模块暂不替换 legacy TUI 的交互主循环；
//! 现阶段的职责是先把 gateway-first 迁移所需的基础设施、内部状态与全屏宿主
//! 骨架立住，后续切换路径仍按 execution-runbook 的阶段顺序推进。

pub(crate) mod app;
#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
pub(crate) mod controller;
#[cfg(test)]
#[path = "controller_tests.rs"]
mod controller_tests;
pub(crate) mod input;
pub(crate) mod model;
pub(crate) mod render;
pub(crate) mod runtime;
pub(crate) mod state;

pub(crate) use app::{TuiRunMode, run_tui_v2};

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;

#[cfg(test)]
mod tests;
