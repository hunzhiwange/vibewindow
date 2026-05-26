use std::collections::VecDeque;

use crate::app::agent::coordination::bus::InMemoryMessageBus;
use crate::app::agent::coordination::bus_helpers::decrement_correlation_count;
use crate::app::agent::coordination::errors::CoordinationError;
use crate::app::agent::coordination::types::SequencedEnvelope;
use crate::app::agent::coordination::util::normalized_non_empty;

#[cfg(test)]
#[path = "inbox_tests.rs"]
mod inbox_tests;

impl InMemoryMessageBus {
    /// 消费代理收件箱中的待处理信封。
    pub fn drain_for_agent(
        &self,
        agent: &str,
        max: usize,
    ) -> Result<Vec<SequencedEnvelope>, CoordinationError> {
        let mut state = self.lock_state();
        let agent_owned = agent.to_string();
        let inbox_len = state
            .inboxes
            .get(agent)
            .map(VecDeque::len)
            .ok_or_else(|| CoordinationError::UnknownAgent { agent: agent_owned.clone() })?;
        let drain_count = if max == 0 { inbox_len } else { max.min(inbox_len) };
        let mut drained = Vec::with_capacity(drain_count);

        for _ in 0..drain_count {
            let envelope = {
                let inbox = state
                    .inboxes
                    .get_mut(agent)
                    .expect("agent existence should be validated before drain");
                inbox.pop_front()
            };

            if let Some(envelope) = envelope {
                let correlation_counts =
                    state.inbox_correlation_counts.entry(agent_owned.clone()).or_default();
                decrement_correlation_count(correlation_counts, &envelope.envelope);
                drained.push(envelope);
            }
        }

        Ok(drained)
    }

    /// 获取代理收件箱中待处理消息数量。
    pub fn pending_for_agent(&self, agent: &str) -> Result<usize, CoordinationError> {
        let state = self.lock_state();
        state
            .inboxes
            .get(agent)
            .map(VecDeque::len)
            .ok_or_else(|| CoordinationError::UnknownAgent { agent: agent.to_string() })
    }

    /// 获取代理收件箱中指定相关 ID 的待处理消息数量。
    pub fn pending_for_agent_correlation(
        &self,
        agent: &str,
        correlation_id: &str,
    ) -> Result<usize, CoordinationError> {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return Ok(0);
        }

        let state = self.lock_state();
        if !state.inboxes.contains_key(agent) {
            return Err(CoordinationError::UnknownAgent { agent: agent.to_string() });
        }

        Ok(state
            .inbox_correlation_counts
            .get(agent)
            .and_then(|counts| counts.get(correlation_id).copied())
            .unwrap_or(0))
    }

    /// 查看代理收件箱中的待处理信封。
    pub fn peek_for_agent(
        &self,
        agent: &str,
        max: usize,
    ) -> Result<Vec<SequencedEnvelope>, CoordinationError> {
        self.peek_for_agent_with_offset(agent, 0, max)
    }

    /// 查看代理收件箱中的待处理信封，支持偏移量。
    pub fn peek_for_agent_with_offset(
        &self,
        agent: &str,
        offset: usize,
        max: usize,
    ) -> Result<Vec<SequencedEnvelope>, CoordinationError> {
        let state = self.lock_state();
        let inbox = state
            .inboxes
            .get(agent)
            .ok_or_else(|| CoordinationError::UnknownAgent { agent: agent.to_string() })?;
        let available = inbox.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        Ok(inbox.iter().skip(offset).take(take_count).cloned().collect())
    }

    /// 查看代理收件箱中匹配指定相关 ID 的信封，支持偏移量。
    pub fn peek_for_agent_correlation_with_offset(
        &self,
        agent: &str,
        correlation_id: &str,
        offset: usize,
        max: usize,
    ) -> Result<Vec<SequencedEnvelope>, CoordinationError> {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return Ok(Vec::new());
        }

        let state = self.lock_state();
        let inbox = state
            .inboxes
            .get(agent)
            .ok_or_else(|| CoordinationError::UnknownAgent { agent: agent.to_string() })?;
        let available = state
            .inbox_correlation_counts
            .get(agent)
            .and_then(|counts| counts.get(correlation_id).copied())
            .unwrap_or(0)
            .saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        Ok(inbox
            .iter()
            .filter(|entry| {
                normalized_non_empty(entry.envelope.correlation_id.as_deref())
                    .is_some_and(|value| value == correlation_id)
            })
            .skip(offset)
            .take(take_count)
            .cloned()
            .collect())
    }
}
