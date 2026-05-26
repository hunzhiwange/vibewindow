//! 协调模块单元测试
//!
//! 本模块包含 `InMemoryMessageBus` 及相关协调组件的测试，覆盖消息验证、
//! 去重与投递、共享上下文、收件箱、死信队列和关联索引。

use super::*;

mod context;
mod dead_letters;
mod delivery;
mod envelope;
mod inbox;
