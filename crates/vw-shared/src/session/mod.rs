//! 会话相关共享模块。
//!
//! 该模块聚合代理会话的元数据定义、UI 会话结构、路径规则与持久化辅助逻辑，
//! 用于在不同 crate 之间共享统一的会话表示。
//!
//! # 子模块
//!
//! - `info`：代理会话元信息
//! - `path`：会话文件与数据库路径计算
//! - `session_utils`：会话标题与 slug 辅助函数
//! - `ui_store`：UI 会话与代理会话的存取实现
//! - `ui_types`：桌面端会话列表和消息展示结构

pub mod info;
pub mod path;
pub mod session_utils;
pub mod ui_store;
pub mod ui_types;

#[cfg(test)]
mod tests;
