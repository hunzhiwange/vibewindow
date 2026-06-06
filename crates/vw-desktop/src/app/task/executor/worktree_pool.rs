#![cfg_attr(target_arch = "wasm32", allow(dead_code, unused_imports))]

//! 任务执行器的 worktree_pool.rs 子模块。
//!
//! 该模块聚焦任务运行过程中的一个局部职责，供执行器入口组合调用。注释说明边界、错误传播和平台差异，避免调用方需要阅读完整执行链才能理解行为。

use super::git::{
    abort_git_in_progress_states, git_branch_name, git_output_failure_detail, git_repo_root,
    run_git_logged, run_git_maintenance_logged,
};
use super::process_utils::{emit_stderr_log, emit_stdout_log, normalize_path};
use super::state::{
    RepoWorktreePool, TaskLogStream, WORKTREE_POOL_REFRESH_INTERVAL_MS, WorktreeEntry,
    WorktreePoolSnapshot, WorktreeSlot, WorktreeSlotSnapshot, WorktreeState, worktree_pools,
};
use super::*;

/// 模块内部可见的 parse_worktree_list 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn parse_worktree_list(output: &str) -> Vec<WorktreeEntry> {
    let mut entries = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_branch: Option<String> = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(path) = current_path.take() {
                entries.push(WorktreeEntry { path, branch: current_branch.take() });
            }
            current_path = Some(path.trim().to_string());
            current_branch = None;
            continue;
        }
        if let Some(branch) = line.strip_prefix("branch ") {
            current_branch = branch
                .trim()
                .strip_prefix("refs/heads/")
                .map(|s| s.to_string())
                .or_else(|| Some(branch.trim().to_string()));
            continue;
        }
        if line.trim().is_empty() {
            if let Some(path) = current_path.take() {
                entries.push(WorktreeEntry { path, branch: current_branch.take() });
            }
            current_branch = None;
        }
    }

    if let Some(path) = current_path.take() {
        entries.push(WorktreeEntry { path, branch: current_branch.take() });
    }
    entries
}

/// 模块内部可见的 now_ms 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn now_ms() -> u64 {
    crate::app::time::now_ms()
}

/// 模块内部可见的 worktree_pool_root 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn worktree_pool_root(repo_root: &str) -> PathBuf {
    let home = super::programs::user_home_dir().unwrap_or_else(|| PathBuf::from(repo_root));
    home.join(".vibewindow").join("task-worktrees")
}

/// 模块内部可见的 is_managed_worktree_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn is_managed_worktree_path(repo_root: &str, path: &str) -> bool {
    let root = normalize_path(&worktree_pool_root(repo_root).to_string_lossy());
    normalize_path(path).starts_with(&root)
}

/// 模块内部可见的 normalized_repo_root 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn normalized_repo_root(project_path: &str) -> Option<String> {
    git_repo_root(project_path).map(|repo_root| normalize_path(&repo_root))
}

/// 模块内部可见的 synchronized_repo_root 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn synchronized_repo_root(
    project_path: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Option<String> {
    ensure_repo_pool(project_path, sender).or_else(|| normalized_repo_root(project_path))
}

/// 公开的 current_task_worktree_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn current_task_worktree_path(project_path: &str, task_id: &str) -> Option<String> {
    task_worktree_path(project_path, task_id)
}

/// 公开的 task_has_live_worktree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn task_has_live_worktree(project_path: &str, task_id: &str) -> bool {
    task_worktree_path(project_path, task_id).map(|path| Path::new(&path).exists()).unwrap_or(false)
}

/// 公开的 worktree_pool_snapshot 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn worktree_pool_snapshot(project_path: &str) -> Option<WorktreePoolSnapshot> {
    let repo_root = normalized_repo_root(project_path)?;
    let pool =
        if let Ok(pools) = worktree_pools().lock() { pools.get(&repo_root).cloned() } else { None }
            .or_else(|| {
                let _ = ensure_repo_pool(project_path, None)?;
                let pools = worktree_pools().lock().ok()?;
                pools.get(&repo_root).cloned()
            })?;

    let mut idle_count = 0usize;
    let mut busy_count = 0usize;
    let mut tainted_count = 0usize;
    let mut recycling_count = 0usize;
    let mut dead_count = 0usize;
    for slot in &pool.slots {
        match slot.state {
            WorktreeState::Idle => idle_count += 1,
            WorktreeState::Busy => busy_count += 1,
            WorktreeState::Tainted => tainted_count += 1,
            WorktreeState::Recycling => recycling_count += 1,
            WorktreeState::Dead => dead_count += 1,
        }
    }

    let mut merge_target_locks = pool
        .merge_target_locks
        .iter()
        .map(|(target, task_id)| (target.clone(), task_id.clone()))
        .collect::<Vec<_>>();
    merge_target_locks.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut slots = pool
        .slots
        .iter()
        .map(|slot| WorktreeSlotSnapshot {
            id: slot.id.clone(),
            path: slot.path.clone(),
            branch: slot.branch.clone(),
            state: slot.state,
            leased_task_id: slot.leased_task_id.clone(),
            taint_reason: slot.taint_reason.clone(),
        })
        .collect::<Vec<_>>();
    slots.sort_by(|a, b| a.id.cmp(&b.id));

    Some(WorktreePoolSnapshot {
        repo_root: pool.repo_root.clone(),
        pool_root: normalize_path(&worktree_pool_root(&repo_root).to_string_lossy()),
        base_branch: pool.base_branch.clone(),
        idle_count,
        busy_count,
        tainted_count,
        recycling_count,
        dead_count,
        merge_target_locks,
        slots,
    })
}

/// 公开的 task_merge_lock_holder 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn task_merge_lock_holder(project_path: &str, task: &Task) -> Option<String> {
    let target_branch = task.merge_target_branch.as_deref()?.trim();
    if target_branch.is_empty() {
        return None;
    }
    let repo_root = synchronized_repo_root(project_path, None)?;
    let pools = worktree_pools().lock().ok()?;
    pools.get(&repo_root)?.merge_target_locks.get(target_branch).cloned()
}

/// 公开的 force_unlock_task_merge_target 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn force_unlock_task_merge_target(project_path: &str, task: &Task) {
    unlock_merge_target(project_path, task);
}

/// 模块内部可见的 sanitize_branch_token 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn sanitize_branch_token(value: &str) -> String {
    value
        .chars()
        .map(
            |ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') { ch } else { '-' }
            },
        )
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// 模块内部可见的 ensure_repo_pool 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn ensure_repo_pool(
    project_path: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Option<String> {
    let repo_root = git_repo_root(project_path)?;
    let repo_root = normalize_path(&repo_root);

    let base_branch = git_branch_name(&repo_root)
        .filter(|branch| branch != "HEAD")
        .unwrap_or_else(|| "main".to_string());
    let current_ms = now_ms();

    if let Ok(mut pools) = worktree_pools().lock()
        && let Some(existing) = pools.get_mut(&repo_root)
    {
        existing.base_branch = base_branch.clone();
        if current_ms.saturating_sub(existing.last_synced_at_ms) < WORKTREE_POOL_REFRESH_INTERVAL_MS
        {
            return Some(repo_root);
        }
    }

    let list_output = match run_git_logged(sender, &repo_root, &["worktree", "list", "--porcelain"])
    {
        Ok(output) => output,
        Err(err) => {
            emit_stderr_log(sender, format!("[WORKTREE] 读取 worktree 池失败: {}", err));
            return Some(repo_root);
        }
    };
    if !list_output.status.success() {
        return Some(repo_root);
    }

    let stdout = String::from_utf8_lossy(&list_output.stdout).to_string();
    let entries = parse_worktree_list(&stdout);

    let existing_slots = worktree_pools()
        .lock()
        .ok()
        .and_then(|pools| pools.get(&repo_root).map(|pool| pool.slots.clone()))
        .unwrap_or_default();
    let existing_slots_by_id =
        existing_slots.into_iter().map(|slot| (slot.id.clone(), slot)).collect::<HashMap<_, _>>();

    let mut slots = Vec::new();
    for entry in entries {
        let normalized = normalize_path(&entry.path);
        if !is_managed_worktree_path(&repo_root, &normalized) {
            continue;
        }

        let branch = entry.branch.clone().unwrap_or_else(|| "HEAD".to_string());
        let slot_id = Path::new(&normalized)
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| sanitize_branch_token(&branch));

        let (state, leased_task_id, taint_reason) =
            if let Some(existing_slot) = existing_slots_by_id.get(&slot_id) {
                match existing_slot.state {
                    WorktreeState::Busy | WorktreeState::Recycling => (
                        existing_slot.state,
                        existing_slot.leased_task_id.clone(),
                        existing_slot.taint_reason.clone(),
                    ),
                    WorktreeState::Tainted | WorktreeState::Dead => {
                        (existing_slot.state, None, existing_slot.taint_reason.clone())
                    }
                    WorktreeState::Idle => (WorktreeState::Idle, None, None),
                }
            } else {
                (WorktreeState::Idle, None, None)
            };
        slots.push(WorktreeSlot {
            id: slot_id,
            path: normalized,
            base_branch: base_branch.clone(),
            branch,
            state,
            leased_task_id,
            taint_reason,
        });
    }

    if let Ok(mut pools) = worktree_pools().lock() {
        let existing = pools.entry(repo_root.clone()).or_insert_with(|| RepoWorktreePool {
            repo_root: repo_root.clone(),
            base_branch: base_branch.clone(),
            slots: Vec::new(),
            task_slots: HashMap::new(),
            merge_target_locks: HashMap::new(),
            last_synced_at_ms: 0,
        });
        existing.base_branch = base_branch;
        existing.last_synced_at_ms = current_ms;
        existing.slots = slots;
    }

    Some(repo_root)
}

fn active_task_slot_count(repo_root: &str) -> usize {
    let Ok(pools) = worktree_pools().lock() else {
        return 0;
    };
    pools.get(repo_root).map(|pool| pool.task_slots.len()).unwrap_or(0)
}

fn create_managed_worktree(
    repo_root: &str,
    base_branch: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<WorktreeSlot, String> {
    let slot_id = format!("slot-{}", now_ms());
    let branch = format!("vw/task/{}", sanitize_branch_token(&slot_id));
    let path = worktree_pool_root(repo_root).join(&slot_id);

    std::fs::create_dir_all(worktree_pool_root(repo_root))
        .map_err(|err| format!("创建 worktree 目录失败: {}", err))?;

    let path_string = path.to_string_lossy().to_string();
    let output = run_git_logged(
        sender,
        repo_root,
        &["worktree", "add", "-b", &branch, &path_string, base_branch],
    )?;
    if !output.status.success() {
        return Err(format!("创建 worktree 失败 branch={} path={}", branch, path_string));
    }

    Ok(WorktreeSlot {
        id: slot_id,
        path: normalize_path(&path_string),
        base_branch: base_branch.to_string(),
        branch,
        state: WorktreeState::Idle,
        leased_task_id: None,
        taint_reason: None,
    })
}

/// 公开的 maintain_worktree_pool 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn maintain_worktree_pool(
    project_path: &str,
    running_tasks: usize,
) -> Result<(usize, usize), String> {
    let Some(repo_root) = ensure_repo_pool(project_path, None) else {
        return Ok((0, 0));
    };

    let desired_capacity = running_tasks.max(1);
    let target_idle = desired_capacity;
    let max_worktrees = desired_capacity.saturating_mul(2);

    loop {
        let (idle_count, total_count, base_branch) = {
            let Ok(pools) = worktree_pools().lock() else {
                return Err("worktree 池锁定失败".to_string());
            };
            let Some(pool) = pools.get(&repo_root) else {
                return Ok((0, target_idle));
            };
            (
                pool.slots.iter().filter(|slot| slot.state == WorktreeState::Idle).count(),
                pool.slots.len(),
                pool.base_branch.clone(),
            )
        };

        if idle_count >= target_idle || total_count >= max_worktrees {
            return Ok((idle_count, target_idle));
        }

        let slot = create_managed_worktree(&repo_root, &base_branch, None)?;
        let Ok(mut pools) = worktree_pools().lock() else {
            return Err("worktree 池锁定失败".to_string());
        };
        if let Some(pool) = pools.get_mut(&repo_root) {
            pool.slots.push(slot);
        }
    }
}

/// 公开的 worktree_pool_needs_maintenance 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn worktree_pool_needs_maintenance(project_path: &str, running_tasks: usize) -> bool {
    let normalized_project_path = normalize_path(project_path);
    if let Ok(pools) = worktree_pools().lock()
        && let Some(pool) = pools
            .iter()
            .find(|(repo_root, _)| path_matches_repo_root(&normalized_project_path, repo_root))
            .map(|(_, pool)| pool)
    {
        return pool_needs_maintenance(pool, running_tasks);
    }

    let Some(repo_root) = normalized_repo_root(project_path) else {
        return false;
    };
    let Ok(pools) = worktree_pools().lock() else {
        return false;
    };
    let Some(pool) = pools.get(&repo_root) else {
        return true;
    };
    pool_needs_maintenance(pool, running_tasks)
}

fn pool_needs_maintenance(pool: &RepoWorktreePool, running_tasks: usize) -> bool {
    let desired_capacity = running_tasks.max(1);
    let target_idle = desired_capacity;
    let max_worktrees = desired_capacity.saturating_mul(2);
    let idle_count = pool.slots.iter().filter(|slot| slot.state == WorktreeState::Idle).count();
    idle_count < target_idle && pool.slots.len() < max_worktrees
}

fn path_matches_repo_root(path: &str, repo_root: &str) -> bool {
    path == repo_root || path.strip_prefix(repo_root).is_some_and(|suffix| suffix.starts_with('/'))
}

/// 模块内部可见的 acquire_task_worktree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn acquire_task_worktree(
    project_path: &str,
    task_id: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Option<WorktreeSlot> {
    let repo_root = ensure_repo_pool(project_path, sender)?;

    if let Ok(mut pools) = worktree_pools().lock()
        && let Some(pool) = pools.get_mut(&repo_root)
        && let Some(slot_id) = pool.task_slots.get(task_id).cloned()
        && let Some(slot) = pool.slots.iter_mut().find(|slot| slot.id == slot_id)
    {
        slot.state = WorktreeState::Busy;
        slot.leased_task_id = Some(task_id.to_string());
        emit_stdout_log(
            sender,
            format!(
                "[WORKTREE] 复用已分配槽位 task={} slot={} path={} base_branch={}",
                task_id, slot.id, slot.path, slot.base_branch
            ),
        );
        return Some(slot.clone());
    }

    let _ = maintain_worktree_pool(project_path, active_task_slot_count(&repo_root));
    let Ok(mut pools) = worktree_pools().lock() else {
        return None;
    };
    let pool = pools.get_mut(&repo_root)?;
    let Some(slot) = pool.slots.iter_mut().find(|slot| slot.state == WorktreeState::Idle) else {
        emit_stderr_log(
            sender,
            format!("[WORKTREE] 没有空闲槽位可分配 task={} repo_root={}", task_id, repo_root),
        );
        return None;
    };
    slot.state = WorktreeState::Busy;
    slot.leased_task_id = Some(task_id.to_string());
    pool.task_slots.insert(task_id.to_string(), slot.id.clone());
    emit_stdout_log(
        sender,
        format!(
            "[WORKTREE] 分配新槽位 task={} slot={} path={} base_branch={}",
            task_id, slot.id, slot.path, slot.base_branch
        ),
    );
    Some(slot.clone())
}

/// 模块内部可见的 prepare_task_worktree_for_execution 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn prepare_task_worktree_for_execution(
    slot: &WorktreeSlot,
    task_id: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<WorktreeSlot, String> {
    let task_branch = format!("vw/task/{}", sanitize_branch_token(task_id));

    emit_stdout_log(
        sender,
        format!(
            "[WORKTREE] 准备任务工作区 slot={} path={} base_branch={} task_branch={}",
            slot.id, slot.path, slot.base_branch, task_branch
        ),
    );

    abort_git_in_progress_states(&slot.path, sender);
    let reset_target = slot.base_branch.clone();

    let reset_output =
        run_git_maintenance_logged(sender, &slot.path, &["reset", "--hard", &reset_target])?;
    if !reset_output.status.success() {
        return Err(format!(
            "重置 worktree 失败 path={} target={} detail={}",
            slot.path,
            reset_target,
            git_output_failure_detail(&reset_output)
        ));
    }

    let clean_output = run_git_maintenance_logged(sender, &slot.path, &["clean", "-fd"])?;
    if !clean_output.status.success() {
        return Err(format!(
            "清理 worktree 失败 path={} detail={}",
            slot.path,
            git_output_failure_detail(&clean_output)
        ));
    }

    let checkout_output = run_git_maintenance_logged(
        sender,
        &slot.path,
        &["checkout", "-B", &task_branch, &reset_target],
    )?;
    if !checkout_output.status.success() {
        return Err(format!(
            "创建任务分支失败 path={} branch={} base={} detail={}",
            slot.path,
            task_branch,
            reset_target,
            git_output_failure_detail(&checkout_output)
        ));
    }

    let mut prepared = slot.clone();
    prepared.branch = task_branch;
    Ok(prepared)
}

/// 模块内部可见的 task_worktree_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn task_worktree_path(project_path: &str, task_id: &str) -> Option<String> {
    let repo_root = synchronized_repo_root(project_path, None)?;
    let Ok(pools) = worktree_pools().lock() else {
        return None;
    };
    let pool = pools.get(&repo_root)?;
    let slot_id = pool.task_slots.get(task_id)?;
    pool.slots.iter().find(|slot| &slot.id == slot_id).map(|slot| slot.path.clone())
}

/// 公开的 can_dispatch_merge_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn can_dispatch_merge_task(project_path: &str, task: &Task) -> bool {
    let target_branch = task.merge_target_branch.as_deref().unwrap_or("").trim();
    if target_branch.is_empty() {
        return true;
    }
    let Some(repo_root) = synchronized_repo_root(project_path, None) else {
        return true;
    };
    let Ok(pools) = worktree_pools().lock() else {
        return false;
    };
    let Some(pool) = pools.get(&repo_root) else {
        return true;
    };
    match pool.merge_target_locks.get(target_branch) {
        Some(task_id) => task_id == &task.id,
        None => true,
    }
}

/// 模块内部可见的 lock_merge_target 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn lock_merge_target(project_path: &str, task: &Task) {
    let target_branch = task.merge_target_branch.as_deref().unwrap_or("").trim();
    if target_branch.is_empty() {
        return;
    }
    let Some(repo_root) = synchronized_repo_root(project_path, None) else {
        return;
    };
    if let Ok(mut pools) = worktree_pools().lock()
        && let Some(pool) = pools.get_mut(&repo_root)
    {
        pool.merge_target_locks.insert(target_branch.to_string(), task.id.clone());
    }
}

/// 模块内部可见的 unlock_merge_target 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn unlock_merge_target(project_path: &str, task: &Task) {
    let target_branch = task.merge_target_branch.as_deref().unwrap_or("").trim();
    if target_branch.is_empty() {
        return;
    }
    let Some(repo_root) = synchronized_repo_root(project_path, None) else {
        return;
    };
    if let Ok(mut pools) = worktree_pools().lock()
        && let Some(pool) = pools.get_mut(&repo_root)
        && pool.merge_target_locks.get(target_branch) == Some(&task.id)
    {
        pool.merge_target_locks.remove(target_branch);
    }
}

#[cfg(test)]
#[path = "worktree_pool_tests.rs"]
mod worktree_pool_tests;
