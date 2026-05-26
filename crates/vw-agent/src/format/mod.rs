//! 格式化模块
//!
//! 本模块提供消息和内容的格式化功能，用于将不同来源的数据转换为统一的格式。
//! 主要功能包括：
//! - 消息格式化：将不同格式的消息转换为标准的代理消息格式
//! - 内容转换：处理文本、图片、代码等多种内容类型的格式转换
//! - 适配器支持：为不同的数据源提供格式适配
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::app::agent::format::{format_message, ContentFormatter};
//!
//! // 格式化消息
//! let formatted = format_message(raw_content)?;
//! ```

pub mod format;

pub use format::*;
