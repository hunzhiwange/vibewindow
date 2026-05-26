use std::collections::VecDeque;

use crate::app::agent::coordination::bus::InMemoryMessageBus;
use crate::app::agent::coordination::types::DeadLetter;

#[cfg(test)]
#[path = "dead_letters_tests.rs"]
mod dead_letters_tests;

impl InMemoryMessageBus {
    /// 获取死信队列中的条目，按时间倒序并支持偏移量。
    pub fn dead_letters_recent(&self, offset: usize, max: usize) -> Vec<DeadLetter> {
        let state = self.lock_state();
        let available = state.dead_letters.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        state.dead_letters.iter().rev().skip(offset).take(take_count).cloned().collect()
    }

    /// 获取指定相关 ID 的死信条目，按时间倒序并支持偏移量。
    pub fn dead_letters_recent_for_correlation(
        &self,
        correlation_id: &str,
        offset: usize,
        max: usize,
    ) -> Vec<DeadLetter> {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return Vec::new();
        }

        let state = self.lock_state();
        let Some(entries) = state.dead_letters_by_correlation.get(correlation_id) else {
            return Vec::new();
        };

        let available = entries.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        entries.iter().rev().skip(offset).take(take_count).cloned().collect()
    }

    /// 获取死信队列中的条目总数。
    pub fn dead_letter_count(&self) -> usize {
        self.lock_state().dead_letters.len()
    }

    /// 获取指定相关 ID 的死信条目数。
    pub fn dead_letter_count_for_correlation(&self, correlation_id: &str) -> usize {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return 0;
        }

        let state = self.lock_state();
        state.dead_letters_by_correlation.get(correlation_id).map(VecDeque::len).unwrap_or(0)
    }

    /// 获取死信队列的完整快照。
    pub fn dead_letters(&self) -> Vec<DeadLetter> {
        self.lock_state().dead_letters.clone()
    }
}
