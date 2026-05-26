use crate::app::agent::coordination::bus::InMemoryMessageBus;
use crate::app::agent::coordination::types::{
    InMemoryMessageBusLimits, InMemoryMessageBusStats,
};

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;

impl InMemoryMessageBus {
    /// 获取所有已注册代理的快照。
    pub fn registered_agents(&self) -> Vec<String> {
        let state = self.lock_state();
        let mut agents = state.inboxes.keys().cloned().collect::<Vec<_>>();
        agents.sort();
        agents
    }

    /// 获取消息总线的限制配置。
    pub fn limits(&self) -> InMemoryMessageBusLimits {
        self.lock_state().limits
    }

    /// 获取消息总线的统计信息。
    pub fn stats(&self) -> InMemoryMessageBusStats {
        self.lock_state().stats
    }

    /// 获取订阅者数量。
    pub fn subscriber_count(&self) -> usize {
        self.lock_state().inboxes.len()
    }
}
