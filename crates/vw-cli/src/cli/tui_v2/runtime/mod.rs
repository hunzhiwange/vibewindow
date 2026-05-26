//! TUI v2 runtime 模块。
//!
//! 本模块用于承接新的 gateway-first 运行时边界。
//! 在 Phase 1 的 S1-1 slice 中，仅提供 GatewayUiRuntime 的最小骨架，
//! 让 CLI 端具备直接构造 GatewayClient 并保存会话上下文入口的能力。
//!
//! 当前目录下各子模块的职责如下：
//! - `gateway`: GatewayUiRuntime 主封装与 stream terminal 语义
//! - `stream_adapter`: gateway 流事件到内部 runtime 事件的转换
//! - `session_store`: session_ui、path、scope 等访问封装
//! - `question_poller`: question/todo 的拉取与更新封装

pub(crate) mod gateway;
pub(crate) mod question_poller;
#[cfg(test)]
#[path = "question_poller_tests.rs"]
mod question_poller_tests;
pub(crate) mod session_store;
#[cfg(test)]
#[path = "session_store_tests.rs"]
mod session_store_tests;
pub(crate) mod stream_adapter;
#[cfg(test)]
#[path = "stream_adapter_tests.rs"]
mod stream_adapter_tests;

#[cfg(test)]
mod tests;
