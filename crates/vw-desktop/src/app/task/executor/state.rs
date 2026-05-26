#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

//! 任务执行器的 state.rs 子模块。
//!
//! 该模块聚焦任务运行过程中的一个局部职责，供执行器入口组合调用。注释说明边界、错误传播和平台差异，避免调用方需要阅读完整执行链才能理解行为。

use super::*;

/// 公开的 TaskLogStream 枚举，描述该模块支持的一组离散状态或事件。
#[derive(Debug, Clone)]
pub enum TaskLogStream {
    Stdout(String),
    Stderr(String),
    ExitStatus { success: bool, code: Option<i32>, signal: Option<i32> },
}

/// 公开的 TaskExecutorState 结构体，承载该模块边界内传递的结构化状态。
pub struct TaskExecutorState {
    pub running_tasks: Vec<String>,
    pub max_concurrent: u32,
    pub simulation_delay_ms: u64,
    pub log_receivers: HashMap<String, Receiver<TaskLogStream>>,
    log_senders: HashMap<String, Sender<TaskLogStream>>,
}

/// 公开的 WorktreeState 枚举，描述该模块支持的一组离散状态或事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeState {
    Idle,
    Busy,
    Tainted,
    Recycling,
    Dead,
}

/// 模块内部可见的 WorktreeSlot 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone)]
pub(super) struct WorktreeSlot {
    pub(super) id: String,
    pub(super) path: String,
    pub(super) base_branch: String,
    pub(super) branch: String,
    pub(super) state: WorktreeState,
    pub(super) leased_task_id: Option<String>,
    pub(super) taint_reason: Option<String>,
}

/// 模块内部可见的 RepoWorktreePool 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone)]
pub(super) struct RepoWorktreePool {
    pub(super) repo_root: String,
    pub(super) base_branch: String,
    pub(super) slots: Vec<WorktreeSlot>,
    pub(super) task_slots: HashMap<String, String>,
    pub(super) merge_target_locks: HashMap<String, String>,
    pub(super) last_synced_at_ms: u64,
}

/// 公开的 WorktreePoolSnapshot 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone)]
pub struct WorktreePoolSnapshot {
    pub repo_root: String,
    pub pool_root: String,
    pub base_branch: String,
    pub idle_count: usize,
    pub busy_count: usize,
    pub tainted_count: usize,
    pub recycling_count: usize,
    pub dead_count: usize,
    pub merge_target_locks: Vec<(String, String)>,
    pub slots: Vec<WorktreeSlotSnapshot>,
}

/// 公开的 WorktreeSlotSnapshot 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone)]
pub struct WorktreeSlotSnapshot {
    pub id: String,
    pub path: String,
    pub branch: String,
    pub state: WorktreeState,
    pub leased_task_id: Option<String>,
    pub taint_reason: Option<String>,
}

/// 模块内部可见的 WorktreeEntry 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone)]
pub(super) struct WorktreeEntry {
    pub(super) path: String,
    pub(super) branch: Option<String>,
}

/// 模块内部可见的 SelectedExecutionWorkspace 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(super) struct SelectedExecutionWorkspace {
    pub(super) slot_id: Option<String>,
    pub(super) execution_path: String,
    pub(super) selected_worktree_path: Option<String>,
    pub(super) selected_worktree_branch: Option<String>,
    pub(super) merge_target_branch: Option<String>,
    pub(super) project_path: String,
}

/// 模块内部可见的 WorktreeClaimGuard 结构体，承载该模块边界内传递的结构化状态。
pub(super) struct WorktreeClaimGuard {
    claimed_path: Option<String>,
}

impl WorktreeClaimGuard {
    /// 模块内部可见的 new 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn new(claimed_path: Option<String>) -> Self {
        Self { claimed_path }
    }
}

impl Drop for WorktreeClaimGuard {
    fn drop(&mut self) {
        if let Some(path) = self.claimed_path.take() {
            super::worktree_admin::release_claimed_worktree(&path);
        }
    }
}

static CLAIMED_WORKTREES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static WORKTREE_POOLS: OnceLock<Mutex<HashMap<String, RepoWorktreePool>>> = OnceLock::new();

/// 模块内部可见的 GIT_MERGE_COMMAND_TIMEOUT_SECS 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_MERGE_COMMAND_TIMEOUT_SECS: u64 = 120;
/// 模块内部可见的 GIT_MERGE_RETRY_DELAY_SECS 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_MERGE_RETRY_DELAY_SECS: u64 = 1;
/// 模块内部可见的 GIT_MAINTENANCE_COMMAND_TIMEOUT_SECS 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_MAINTENANCE_COMMAND_TIMEOUT_SECS: u64 = 30;
/// 模块内部可见的 GIT_SUMMARY_TAG 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_SUMMARY_TAG: &str = "__VW_GIT_SUMMARY__";
/// 模块内部可见的 GIT_SOURCE_BRANCH_TAG 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_SOURCE_BRANCH_TAG: &str = "__VW_GIT_SOURCE_BRANCH__";
/// 模块内部可见的 GIT_TARGET_BRANCH_TAG 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_TARGET_BRANCH_TAG: &str = "__VW_GIT_TARGET_BRANCH__";
/// 模块内部可见的 GIT_WORKTREE_PATH_TAG 常量，集中保存该模块复用的稳定取值。
pub(super) const GIT_WORKTREE_PATH_TAG: &str = "__VW_GIT_WORKTREE_PATH__";
/// 模块内部可见的 WORKTREE_POOL_REFRESH_INTERVAL_MS 常量，集中保存该模块复用的稳定取值。
pub(super) const WORKTREE_POOL_REFRESH_INTERVAL_MS: u64 = 3_000;

/// 模块内部可见的 verbose_merge_logging 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn verbose_merge_logging() -> bool {
    std::env::var("VW_VERBOSE_MERGE_LOG")
        .map(|value| matches!(value.as_str(), "1" | "true"))
        .unwrap_or(false)
}

/// 模块内部可见的 claimed_worktrees 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn claimed_worktrees() -> &'static Mutex<HashSet<String>> {
    CLAIMED_WORKTREES.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 模块内部可见的 worktree_pools 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn worktree_pools() -> &'static Mutex<HashMap<String, RepoWorktreePool>> {
    WORKTREE_POOLS.get_or_init(|| Mutex::new(HashMap::new()))
}

impl Default for TaskExecutorState {
    fn default() -> Self {
        Self {
            running_tasks: Vec::new(),
            max_concurrent: 3,
            simulation_delay_ms: 2000,
            log_receivers: HashMap::new(),
            log_senders: HashMap::new(),
        }
    }
}

impl TaskExecutorState {
    /// 公开的 new 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn new() -> Self {
        Self::default()
    }

    /// 公开的 can_start_more 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn can_start_more(&self) -> bool {
        (self.running_tasks.len() as u32) < self.max_concurrent
    }

    /// 公开的 is_running 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn is_running(&self, task_id: &str) -> bool {
        self.running_tasks.contains(&task_id.to_string())
    }

    /// 公开的 start_task 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn start_task(&mut self, task_id: &str) {
        if !self.running_tasks.contains(&task_id.to_string()) {
            self.running_tasks.push(task_id.to_string());
        }
    }

    /// 公开的 finish_task 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn finish_task(&mut self, task_id: &str) {
        self.running_tasks.retain(|id| id != task_id);
        self.log_receivers.remove(task_id);
        self.log_senders.remove(task_id);
    }

    /// 公开的 register_log_channel 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn register_log_channel(&mut self, task_id: String) {
        let (tx, rx) = mpsc::channel();
        self.log_senders.insert(task_id.clone(), tx);
        self.log_receivers.insert(task_id, rx);
    }

    /// 公开的 get_log_sender 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn get_log_sender(&self, task_id: &str) -> Option<Sender<TaskLogStream>> {
        self.log_senders.get(task_id).cloned()
    }

    /// 公开的 poll_task_logs 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn poll_task_logs(&mut self, task_id: &str) -> Vec<TaskLogStream> {
        self.poll_task_logs_all(task_id)
    }

    /// 公开的 poll_task_logs_all 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn poll_task_logs_all(&mut self, task_id: &str) -> Vec<TaskLogStream> {
        let mut logs = Vec::new();
        if let Some(receiver) = self.log_receivers.get(task_id) {
            while let Ok(log) = receiver.try_recv() {
                logs.push(log);
            }
        }
        logs
    }

    /// 公开的 get_all_running_logs 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub fn get_all_running_logs(&mut self) -> Vec<(String, Vec<TaskLogStream>)> {
        let running = self.running_tasks.clone();
        running
            .into_iter()
            .map(|task_id| {
                let logs = self.poll_task_logs(&task_id);
                (task_id, logs)
            })
            .filter(|(_, logs)| !logs.is_empty())
            .collect()
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
