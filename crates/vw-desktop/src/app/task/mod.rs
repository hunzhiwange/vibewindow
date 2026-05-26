//! 任务管理模块
//!
//! 本模块提供了 VibeWindow 的任务调度与执行系统的核心功能。任务系统负责
//! 管理异步任务的生命周期，包括任务的创建、执行、状态跟踪和清理。
//!
//! # 架构概览
//!
//! 任务系统由三个核心子系统组成：
//!
//! - **执行器（executor）**：负责任务的实际执行、工作树管理和并发控制
//! - **数据模型（models）**：定义任务相关的数据结构和状态枚举
//! - **存储层（store）**：提供任务的持久化存储和检索功能
//!
//! # 核心概念
//!
//! ## 任务生命周期
//!
//! 任务从创建到完成经历以下状态：
//! 1. **草稿（Draft）**：任务正在编辑，尚未就绪
//! 2. **待执行（Pending）**：任务已就绪，等待调度
//! 3. **执行中（Running）**：任务正在被处理
//! 4. **已完成（Completed）**：任务成功完成
//! 5. **已取消（Cancelled）**：任务被用户中止
//!
//! ## 工作树管理
//!
//! 每个执行中的任务可以拥有一个独立的 Git 工作树，实现任务间的隔离。
//! 工作树池（Worktree Pool）负责管理工作树的创建、复用和清理。
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use app::task::{Task, TaskExecutorState, execute_task_async};
//!
//! // 创建任务草稿
//! let draft = TaskDraft::new(/* ... */);
//!
//! // 异步执行任务
//! let result = execute_task_async(&state, draft).await?;
//! ```
//!
//! # 模块组织
//!
//! - [`executor`]：任务执行器和工作树管理
//! - [`models`]：任务数据模型定义
//! - [`store`]：任务持久化存储

/// 任务执行器子系统
///
/// 提供任务调度、执行、工作树管理和并发控制功能。
#[path = "executor/mod.rs"]
pub mod executor;

/// 任务数据模型
///
/// 定义任务相关的核心数据结构，包括任务实体、状态枚举和配置。
pub mod models;

/// 任务存储层
///
/// 提供任务的持久化存储、检索和查询功能。
pub mod store;

// =============================================================================
// 重新导出：执行器相关类型和函数
// =============================================================================
//
// 导出的类型：
// - ExecutorCommand：执行器命令类型，用于控制执行器的行为
// - ExecutorEvent：执行器事件，表示执行过程中发生的异步事件
// - TaskExecutorState：任务执行器的运行时状态
// - WorktreePoolSnapshot：工作树池的快照，包含所有槽位的当前状态
// - WorktreeSlotSnapshot：单个工作树槽位的快照信息
// - WorktreeState：单个工作树的运行时状态
//
// 导出的函数：
// - build_executor_command：构建执行器命令
// - build_review_diff_context：构建代码审查的差异上下文
// - can_dispatch_merge_task：检查是否可以调度合并任务
// - count_running_tasks：统计当前正在运行的任务数量
// - current_task_worktree_path：获取当前任务的工作树路径
// - execute_task_async：异步执行任务
// - execute_task_command：执行任务的命令行版本
// - execute_task_merge_async：异步执行任务合并操作
// - execute_task_review_async：异步执行任务代码审查
// - force_unlock_task_merge_target：强制解锁任务的合并目标
// - get_next_tasks_for_execution：获取下一批待执行的任务
// - get_pool_and_pending_count：获取工作树池和待处理任务计数
// - get_total_task_count：获取任务总数
// - maintain_worktree_pool：维护工作树池（清理、回收等）
// - recycle_task_worktree：回收任务使用的工作树
// - simulate_task_execution_step：模拟任务执行的单个步骤（用于测试）
// - task_has_live_worktree：检查任务是否拥有活跃的工作树
// - task_merge_lock_holder：获取任务合并锁的持有者
// - worktree_pool_needs_maintenance：检查工作树池是否需要维护
// - worktree_pool_snapshot：获取工作树池的当前快照

pub use executor::{
    ExecutorCommand, ExecutorEvent, TaskExecutorState, TaskLogStream, WorktreePoolSnapshot,
    WorktreeSlotSnapshot, WorktreeState, assign_task_execution_worktree, build_executor_command,
    build_review_diff_context, can_dispatch_merge_task, commit_merge_all_worktrees,
    commit_merge_all_worktrees_async, commit_merge_all_worktrees_async_with_logs,
    commit_merge_all_worktrees_with_logs, count_running_tasks, current_task_worktree_path,
    delete_all_managed_worktrees, delete_all_managed_worktrees_async,
    delete_all_managed_worktrees_async_with_logs, delete_all_managed_worktrees_with_logs,
    execute_gateway_prompt_with_streaming, execute_task_async, execute_task_command,
    execute_task_command_with_streaming, execute_task_merge_async, execute_task_review_async,
    force_unlock_task_merge_target, get_next_tasks_for_execution, get_pool_and_pending_count,
    get_total_task_count, maintain_worktree_pool, recycle_task_worktree,
    recycle_task_worktree_async, release_task_worktree, release_task_worktree_async,
    reset_all_managed_worktrees, reset_all_managed_worktrees_async,
    reset_all_managed_worktrees_async_with_logs, reset_all_managed_worktrees_with_logs,
    simulate_task_execution_step, task_has_live_worktree, task_merge_lock_holder,
    worktree_pool_needs_maintenance, worktree_pool_snapshot,
};

// =============================================================================
// 重新导出：数据模型
// =============================================================================
//
// 导出的类型：
// - SubTask：子任务实体，表示任务的细分工作项
// - Task：任务实体，表示一个完整的工作单元
// - TaskBoardSettings：任务看板配置
// - TaskDraft：任务草稿，用于创建新任务
// - TaskExecutorBackend：任务执行器后端类型
// - TaskImportPromptFormat：任务导入时的提示词格式
// - TaskIndex：任务索引，用于快速检索
// - TaskLogEntry：任务日志条目
// - TaskStatus：任务状态枚举

pub use models::{
    CLAUDE_DEFAULT_MODEL_ALIAS, CLAUDE_SUPPORTED_MODEL_ALIASES, SubTask, TASK_MODEL_AUTO, Task,
    TaskBoardSettings, TaskDraft, TaskExecutorBackend, TaskImportPromptFormat, TaskIndex,
    TaskLogEntry, TaskStatus, claude_model_alias, legacy_executor_to_task_acp_agent,
    normalize_task_acp_agent_input, normalize_task_model_input,
};

// =============================================================================
// 重新导出：存储层
// =============================================================================
//
// 重新导出 store 模块中的所有公共项，包括：
// - 任务存储接口和实现
// - 任务查询和过滤功能
// - 存储后端配置

pub use store::*;

#[cfg(test)]
mod tests;
