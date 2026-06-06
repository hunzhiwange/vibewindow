//! ACP actor 生命周期状态记录。
//!
//! 本模块只维护 actor 句柄、可复用会话 id 和代理进程生命周期快照，避免这些
//! 状态更新散落在 actor 调度逻辑中。

use super::*;
use crate::session_runtime::AgentLifecycleExit;

impl AcpClient {
    pub(super) fn invalidate_actor(&self) {
        let mut state = self.actor_state.lock();
        state.handle = None;
        state.reusable_session_id = None;
        state.lifecycle.pid = None;
    }

    /// 判断指定会话是否正被当前 actor 运行时复用。
    pub fn has_reusable_session(&self, session_id: &str) -> bool {
        self.actor_state
            .lock()
            .reusable_session_id
            .as_deref()
            .is_some_and(|active_session_id| active_session_id == session_id)
    }

    /// 获取当前代理进程生命周期快照。
    pub fn get_agent_lifecycle_snapshot(&self) -> AgentLifecycleSnapshot {
        self.actor_state.lock().lifecycle.clone()
    }

    pub(super) fn record_actor_start(&self, pid: Option<u32>) {
        let mut state = self.actor_state.lock();
        state.lifecycle.pid = pid;
        state.lifecycle.started_at = Some(iso_now());
        state.lifecycle.last_exit = None;
    }

    pub(super) fn record_actor_exit(
        &self,
        exit: ChildExitSummary,
        reason: Option<&str>,
        unexpected_during_prompt: bool,
    ) {
        let mut state = self.actor_state.lock();
        state.lifecycle.pid = None;
        state.lifecycle.last_exit = Some(AgentLifecycleExit {
            exit_code: exit.exit_code,
            signal: exit.signal,
            exited_at: Some(iso_now()),
            reason: reason.map(ToOwned::to_owned),
            unexpected_during_prompt,
        });
    }

    pub(super) fn store_reusable_session(&self, session_id: Option<String>) {
        self.actor_state.lock().reusable_session_id = session_id;
    }
}
