//!
//! # command 模块
//!
//! 命令系统模块，提供代理可执行命令的定义、管理和调度能力。
//!
//! ## 模块职责
//!
//! - **命令定义**：定义命令的结构、元数据和模板
//! - **命令注册**：管理和维护可用命令的注册表
//! - **命令查找**：按名称或列表查询命令信息
//! - **事件发布**：发布命令执行事件到事件总线
//!
//! ## 架构说明
//!
//! 本模块采用模块化设计：
//! - `mod.rs`：模块入口，负责导出公共接口
//! - `command.rs`：核心实现，包含命令数据结构和业务逻辑
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use crate::app::agent::command::{get, list, Info};
//!
//! // 获取单个命令
//! if let Some(cmd) = get("init").await {
//!     println!("命令名称: {}", cmd.name);
//! }
//!
//! // 列出所有命令
//! let all_commands = list().await;
//! for cmd in all_commands {
//!     println!("- {}", cmd.name);
//! }
//! ```
//!
//! ## 相关模块
//!
//! - [`crate::app::agent::skill`]：技能系统，命令可以从技能中加载
//! - [`crate::app::agent::bus`]：事件总线，用于发布命令执行事件
//! - [`crate::app::agent::config`]：配置系统，命令加载时读取配置
//!

/// 命令核心实现子模块
///
/// 该子模块包含命令系统的核心功能实现：
/// - 命令信息结构 `Info`
/// - 命令状态管理 `State`
/// - 命令来源枚举 `Source`
/// - 命令执行事件 `ExecutedEvent`
/// - 命令查询和发布函数
pub mod command;
#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;

/// 重新导出 command 子模块的所有公共项
///
/// 通过此导出，外部模块可以直接使用：
/// ```rust,ignore
/// use crate::app::agent::command::{Info, State, get, list};
/// ```
///
/// 而无需显式引用子模块路径：
/// ```rust,ignore
/// use crate::app::agent::command::command::{Info, State, get, list};
/// ```
pub use command::*;
