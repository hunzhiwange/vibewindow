//! Composio 工具提供方
//!
//! 可选的托管工具集成，支持 1000+ OAuth 集成（Gmail、Notion、GitHub、Slack 等）。
//! 通过 Composio API 执行操作，无需本地存储原始 OAuth 令牌。
//!
//! # 架构设计
//!
//! 本模块采用分层架构：
//! - `api`: Composio API 客户端，负责与 Composio 服务的 HTTP 通信
//! - `core`: 核心工具实现，包含 ComposioTool 的主要逻辑
//! - `tool`: 工具 trait 实现，定义工具接口
//! - `types`: 数据类型定义，包括操作、连接链接等
//! - `util`: 工具函数，提供辅助功能
//!
//! # 安全性
//!
//! - 采用选择性启用模式，用户可完全跳过此集成
//! - Composio API 密钥存储在加密的密钥库中
//! - 无需本地存储原始 OAuth 令牌
//!
//! # 使用示例
//!
//! ```ignore
//! use vibe_agent::tools::composio::{ComposioTool, ComposioAction};
//!
//! // 创建 Composio 工具实例
//! let tool = ComposioTool::new(api_key)?;
//!
//! // 执行 Gmail 发送邮件操作
//! let action = ComposioAction::new("GMAIL_SEND_EMAIL", params);
//! let result = tool.execute(action).await?;
//! ```

/// Composio API 客户端模块
///
/// 提供与 Composio 服务进行 HTTP 通信的能力，包括：
/// - 认证和授权
/// - 操作列表查询
/// - 操作执行
/// - 连接管理
pub(crate) mod api;

/// 核心工具实现模块
///
/// 包含 ComposioTool 的核心实现逻辑，负责：
/// - 工具生命周期管理
/// - 操作调度和执行
/// - 错误处理和重试
mod core;

/// 工具 trait 实现模块
///
/// 实现 VibeWindow 的 Tool trait，定义标准工具接口
mod tool;

/// 数据类型定义模块
///
/// 定义 Composio 相关的所有数据类型：
/// - `ComposioAction`: 表示一个可执行的操作
/// - `ComposioConnectionLink`: OAuth 连接链接信息
pub(crate) mod types;

/// 工具函数模块
///
/// 提供辅助功能，包括：
/// - 参数验证和转换
/// - 响应解析
/// - 错误处理辅助函数
pub(crate) mod util;

/// 重新导出 ComposioTool 核心工具
///
/// 这是本模块的主要公共接口，用于创建和使用 Composio 工具实例
pub use core::ComposioTool;

/// 重新导出 Composio 相关类型
///
/// - `ComposioAction`: 表示 Composio 平台上的一个可执行操作
/// - `ComposioConnectionLink`: 用于建立 OAuth 连接的链接信息
pub use types::{ComposioAction, ComposioConnectionLink};

/// Composio 工具测试模块
///
/// 测试文件位于 `../tests/composio.rs`，包含：
/// - API 客户端测试
/// - 工具执行测试
/// - 类型序列化/反序列化测试
#[cfg(test)]
#[path = "../tests/composio.rs"]
mod tests;
