use crate::app::agent::coordination::bus::InMemoryMessageBus;

#[cfg(test)]
#[path = "agents_tests.rs"]
mod agents_tests;
use crate::app::agent::coordination::errors::CoordinationError;
use crate::app::agent::coordination::util::require_non_empty;

impl InMemoryMessageBus {
    /// 注册代理收件箱。
    pub fn register_agent(&self, agent: impl Into<String>) -> Result<(), CoordinationError> {
        let agent = agent.into();
        require_non_empty(&agent, "agent")?;

        let mut state = self.lock_state();
        state.inboxes.entry(agent.clone()).or_default();
        state.inbox_correlation_counts.entry(agent).or_default();
        Ok(())
    }

    /// 注销代理收件箱。
    pub fn unregister_agent(&self, agent: &str) -> bool {
        let mut state = self.lock_state();
        let removed = state.inboxes.remove(agent).is_some();
        state.inbox_correlation_counts.remove(agent);
        removed
    }
}
