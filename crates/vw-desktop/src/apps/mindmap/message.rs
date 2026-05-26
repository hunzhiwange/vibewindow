//! 思维导图消息处理模块
//!
//! 本模块负责思维导图应用的消息定义、处理和持久化相关功能。
//! 采用模块化设计，将不同职责分离到各个子模块中。
//!
//! # 模块结构
//!
//! - [`persist`] - 消息持久化相关功能，负责保存和加载思维导图状态
//! - [`tabs`] - 标签页管理功能，包括创建新标签页和关闭标签页
//! - [`types`] - 消息类型定义，包含思维导图使用的所有消息枚举和结构体
//! - [`update`] - 消息更新处理逻辑，处理各种消息并更新应用状态

mod persist;
mod tabs;
mod types;
mod update;

#[cfg(test)]
mod persist_tests;
#[cfg(test)]
mod tabs_tests;
#[cfg(test)]
mod types_tests;

/// 从持久化存储加载已保存的文件
///
/// 该函数尝试从本地存储中恢复上次会话的思维导图状态。
pub use persist::load_persisted;

/// 关闭指定的标签页
///
/// # 参数
///
/// * 标签页标识符 - 用于定位要关闭的标签页
pub use tabs::close_tab;

/// 创建一个新的空白标签页
///
/// 返回新创建标签页的初始化状态。
pub use tabs::new_blank_tab;

/// 思维导图消息类型
///
/// 包含思维导图应用中所有可能的消息变体，用于驱动应用状态更新。
pub use types::MindMapMessage;

/// 消息处理函数
///
/// 根据接收到的消息类型更新应用状态。
pub use update::update;
