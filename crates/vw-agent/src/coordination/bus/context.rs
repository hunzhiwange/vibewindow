use std::collections::{HashMap, VecDeque};

use crate::app::agent::coordination::bus::InMemoryMessageBus;
use crate::app::agent::coordination::types::SharedContextEntry;

#[cfg(test)]
#[path = "context_tests.rs"]
mod context_tests;

impl InMemoryMessageBus {
    /// 获取共享上下文的完整快照。
    pub fn context_snapshot(&self) -> HashMap<String, SharedContextEntry> {
        self.lock_state().context.clone()
    }

    /// 获取共享上下文条目，按写入时间倒序。
    pub fn context_entries_recent(&self, max: usize) -> Vec<(String, SharedContextEntry)> {
        self.context_entries_recent_with_offset(0, max)
    }

    /// 获取共享上下文条目，按写入时间倒序并支持偏移量。
    pub fn context_entries_recent_with_offset(
        &self,
        offset: usize,
        max: usize,
    ) -> Vec<(String, SharedContextEntry)> {
        let state = self.lock_state();
        let available = state.context_order.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        state
            .context_order
            .iter()
            .rev()
            .skip(offset)
            .take(take_count)
            .filter_map(|key| state.context.get(key).cloned().map(|entry| (key.clone(), entry)))
            .collect()
    }

    /// 获取指定相关 ID 的共享上下文条目，按写入时间倒序。
    pub fn context_entries_recent_for_correlation(
        &self,
        correlation_id: &str,
        max: usize,
    ) -> Vec<(String, SharedContextEntry)> {
        self.context_entries_recent_for_correlation_with_offset(correlation_id, 0, max)
    }

    /// 获取指定相关 ID 的共享上下文条目，按写入时间倒序并支持偏移量。
    pub fn context_entries_recent_for_correlation_with_offset(
        &self,
        correlation_id: &str,
        offset: usize,
        max: usize,
    ) -> Vec<(String, SharedContextEntry)> {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return Vec::new();
        }

        let state = self.lock_state();
        let Some(order) = state.context_order_by_correlation.get(correlation_id) else {
            return Vec::new();
        };

        let available = order.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        order
            .iter()
            .rev()
            .skip(offset)
            .take(take_count)
            .filter_map(|key| state.context.get(key).cloned().map(|entry| (key.clone(), entry)))
            .collect()
    }

    /// 获取共享上下文条目总数。
    pub fn context_count(&self) -> usize {
        self.lock_state().context.len()
    }

    /// 获取指定相关 ID 的共享上下文条目数。
    pub fn context_count_for_correlation(&self, correlation_id: &str) -> usize {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return 0;
        }

        let state = self.lock_state();
        state.context_order_by_correlation.get(correlation_id).map(VecDeque::len).unwrap_or(0)
    }

    /// 获取 delegate/ 前缀共享上下文条目，按写入时间倒序并支持偏移量。
    pub fn delegate_context_entries_recent_with_offset(
        &self,
        offset: usize,
        max: usize,
    ) -> Vec<(String, SharedContextEntry)> {
        let state = self.lock_state();
        let available = state.delegate_context_order.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        state
            .delegate_context_order
            .iter()
            .rev()
            .skip(offset)
            .take(take_count)
            .filter_map(|key| state.context.get(key).cloned().map(|entry| (key.clone(), entry)))
            .collect()
    }

    /// 获取指定相关 ID 的 delegate/ 前缀共享上下文条目，按写入时间倒序并支持偏移量。
    pub fn delegate_context_entries_recent_for_correlation_with_offset(
        &self,
        correlation_id: &str,
        offset: usize,
        max: usize,
    ) -> Vec<(String, SharedContextEntry)> {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return Vec::new();
        }

        let state = self.lock_state();
        let Some(order) = state.delegate_context_order_by_correlation.get(correlation_id) else {
            return Vec::new();
        };

        let available = order.len().saturating_sub(offset);
        let take_count = if max == 0 { available } else { max.min(available) };

        order
            .iter()
            .rev()
            .skip(offset)
            .take(take_count)
            .filter_map(|key| state.context.get(key).cloned().map(|entry| (key.clone(), entry)))
            .collect()
    }

    /// 获取 delegate/ 前缀共享上下文条目总数。
    pub fn delegate_context_count(&self) -> usize {
        self.lock_state().delegate_context_order.len()
    }

    /// 获取指定相关 ID 的 delegate/ 前缀共享上下文条目数。
    pub fn delegate_context_count_for_correlation(&self, correlation_id: &str) -> usize {
        let correlation_id = correlation_id.trim();
        if correlation_id.is_empty() {
            return 0;
        }

        let state = self.lock_state();
        state
            .delegate_context_order_by_correlation
            .get(correlation_id)
            .map(VecDeque::len)
            .unwrap_or(0)
    }

    /// 获取单个共享上下文条目。
    pub fn context_entry(&self, key: &str) -> Option<SharedContextEntry> {
        self.lock_state().context.get(key).cloned()
    }
}
