//! # 内存消息总线模块
//!
//! 本模块提供确定性的内存协调消息总线实现，并将不同职责的方法拆分到私有子模块中，
//! 以便在不改变外部接口的前提下维持清晰边界。

mod agents;
mod context;
mod dead_letters;
mod inbox;
mod metadata;

#[cfg(test)]
#[path = "bus_tests.rs"]
mod bus_tests;

use std::sync::{Arc, Mutex, MutexGuard};

use crate::app::agent::coordination::CoordinationEnvelope;
use crate::app::agent::coordination::bus_dead_letters::push_dead_letter_locked;
use crate::app::agent::coordination::bus_publish::publish_envelope;
use crate::app::agent::coordination::errors::CoordinationError;
use crate::app::agent::coordination::state::BusState;
use crate::app::agent::coordination::types::{InMemoryMessageBusLimits, PublishReceipt};

/// 确定性内存协调消息总线。
#[derive(Debug, Clone)]
pub struct InMemoryMessageBus {
    inner: Arc<Mutex<BusState>>,
}

impl Default for InMemoryMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMessageBus {
    /// 创建新的消息总线，使用默认限制配置。
    pub fn new() -> Self {
        Self::with_limits(InMemoryMessageBusLimits::recommended())
    }

    /// 使用自定义限制配置创建消息总线。
    pub fn with_limits(limits: InMemoryMessageBusLimits) -> Self {
        Self { inner: Arc::new(Mutex::new(BusState::with_limits(limits))) }
    }

    /// 发布信封到消息总线。
    pub fn publish(
        &self,
        envelope: CoordinationEnvelope,
    ) -> Result<PublishReceipt, CoordinationError> {
        let mut state = self.lock_state();
        publish_envelope(&mut state, envelope)
    }

    /// 将信封添加到死信队列。
    pub(crate) fn push_dead_letter(&self, envelope: CoordinationEnvelope, reason: String) {
        let mut state = self.lock_state();
        push_dead_letter_locked(&mut state, envelope, reason);
    }

    fn lock_state(&self) -> MutexGuard<'_, BusState> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
