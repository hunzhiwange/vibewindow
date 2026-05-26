//! # 协调模块 (Coordination)
//!
//! 本模块提供 Agent 系统内部的协调与通信机制，实现组件之间的解耦消息传递。
//!
//! ## 核心功能
//!
//! - **消息总线**: 基于内存的发布-订阅消息总线，支持多订阅者和消息路由
//! - **信封机制**: 消息封装与投递范围控制
//! - **死信处理**: 无法投递消息的存储与恢复机制
//! - **共享上下文**: 跨组件的上下文共享与状态管理
//!
//! ## 架构说明
//!
//! 本模块采用分层架构：
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           公共 API 层                    │
//! │  (InMemoryMessageBus, Envelope, Types)  │
//! └─────────────────────────────────────────┘
//!                     │
//! ┌─────────────────────────────────────────┐
//! │           消息总线核心                   │
//! │  (bus, bus_publish, bus_inbox)          │
//! └─────────────────────────────────────────┘
//!                     │
//! ┌─────────────────────────────────────────┐
//! │           支撑子系统                     │
//! │  (state, dead_letters, context)         │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use crate::app::agent::coordination::{InMemoryMessageBus, CoordinationEnvelope};
//!
//! // 创建消息总线
//! let bus = InMemoryMessageBus::new(Default::default());
//!
//! // 发布消息
//! let envelope = CoordinationEnvelope::new(payload, scope);
//! bus.publish(envelope).await?;
//! ```
//!
//! ## 子模块说明
//!
//! - `bus`: 内存消息总线核心实现
//! - `bus_context`: 总线共享上下文管理
//! - `bus_dead_letters`: 死信队列处理
//! - `bus_helpers`: 总线辅助工具函数
//! - `bus_inbox`: 订阅者收件箱实现
//! - `bus_publish`: 消息发布逻辑
//! - `envelope`: 消息信封与载荷封装
//! - `errors`: 协调相关错误类型
//! - `state`: 总线状态管理
//! - `types`: 公共类型定义
//! - `util`: 内部工具函数

// 子模块声明
mod bus;
mod bus_context;
mod bus_dead_letters;
mod bus_helpers;
mod bus_inbox;
mod bus_publish;
mod envelope;
mod errors;
mod state;
mod types;
mod util;

#[cfg(test)]
#[path = "bus_context_tests.rs"]
mod bus_context_tests;
#[cfg(test)]
#[path = "bus_dead_letters_tests.rs"]
mod bus_dead_letters_tests;
#[cfg(test)]
#[path = "bus_helpers_tests.rs"]
mod bus_helpers_tests;
#[cfg(test)]
#[path = "bus_inbox_tests.rs"]
mod bus_inbox_tests;
#[cfg(test)]
#[path = "bus_publish_tests.rs"]
mod bus_publish_tests;
#[cfg(test)]
#[path = "envelope_tests.rs"]
mod envelope_tests;
#[cfg(test)]
#[path = "errors_tests.rs"]
mod errors_tests;
#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
#[cfg(test)]
#[path = "util_tests.rs"]
mod util_tests;

// 公共接口导出

/// 内存消息总线实现
///
/// 提供基于内存的发布-订阅消息传递机制，支持：
/// - 多订阅者路由
/// - 消息序列化与持久化
/// - 背压控制与限流
/// - 死信队列处理
pub use bus::InMemoryMessageBus;

/// 消息信封与投递相关类型
///
/// - `CoordinationEnvelope`: 消息信封，包含元数据与载荷
/// - `CoordinationPayload`: 消息载荷类型
/// - `DeliveryScope`: 消息投递范围（单播/广播/主题）
pub use envelope::{CoordinationEnvelope, CoordinationPayload, DeliveryScope};

/// 协调模块错误类型
///
/// 涵盖消息发布、订阅、投递等操作中可能发生的错误
pub use errors::CoordinationError;

/// 协调模块公共类型定义
///
/// - `DeadLetter`: 死信消息表示
/// - `InMemoryMessageBusLimits`: 总线资源限制配置
/// - `InMemoryMessageBusStats`: 总线运行统计信息
/// - `PublishReceipt`: 消息发布回执
/// - `SequencedEnvelope`: 带序列号的消息信封
/// - `SharedContextEntry`: 共享上下文条目
pub use types::{
    DeadLetter, InMemoryMessageBusLimits, InMemoryMessageBusStats, PublishReceipt,
    SequencedEnvelope, SharedContextEntry,
};

// 单元测试模块
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
