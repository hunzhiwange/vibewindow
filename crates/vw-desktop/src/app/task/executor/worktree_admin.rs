#![cfg_attr(target_arch = "wasm32", allow(dead_code, unused_imports))]

//! 任务执行器的 worktree_admin.rs 子模块。
//!
//! 该模块聚焦任务运行过程中的一个局部职责，供执行器入口组合调用。注释说明边界、错误传播和平台差异，避免调用方需要阅读完整执行链才能理解行为。

use super::git::{
    abort_git_in_progress_states, git_branch_name, git_has_staged_changes, git_is_clean,
    git_output_failure_detail, git_repo_root, run_git_logged, run_git_maintenance,
    run_git_maintenance_logged,
};
use super::process_utils::{emit_stderr_log, emit_stdout_log, normalize_path};
use super::state::{
    TaskLogStream, WorktreeClaimGuard, WorktreeState, claimed_worktrees, worktree_pools,
};
use super::worktree_pool::{
    acquire_task_worktree, ensure_repo_pool, is_managed_worktree_path, parse_worktree_list,
    prepare_task_worktree_for_execution, synchronized_repo_root,
};
use super::*;

fn valid_branch_name(branch: &str) -> bool {
    let branch = branch.trim();
    !branch.is_empty() && branch != "HEAD"
}

fn resolve_merge_target_branch(task: &Task, project_path: &str) -> Option<String> {
    task.merge_target_branch
        .as_deref()
        .map(str::trim)
        .filter(|branch| valid_branch_name(branch))
        .map(str::to_string)
        .or_else(|| git_branch_name(project_path).filter(|branch| valid_branch_name(branch)))
        .or_else(|| {
            git_repo_root(project_path)
                .and_then(|repo_root| git_branch_name(&repo_root))
                .filter(|branch| valid_branch_name(branch))
        })
}

/// 公开的 assign_task_execution_worktree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn assign_task_execution_worktree(
    project_path: &str,
    task: &Task,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<Option<String>, String> {
    let selected_path = task
        .selected_worktree_path
        .clone()
        .map(|path| normalize_path(&path))
        .filter(|path| Path::new(path).exists());
    if let Some(path) = selected_path {
        emit_stdout_log(
            sender,
            format!("[WORKTREE] 使用已选择工作区 task={} path={}", task.id, path),
        );
        return Ok(Some(path));
    }

    let effective_target_branch = resolve_merge_target_branch(task, project_path);

    if effective_target_branch.is_none() {
        emit_stdout_log(sender, format!("[WORKTREE] 任务无需池化 worktree task={}", task.id));
        return Ok(None);
    }

    let Some(slot) = acquire_task_worktree(project_path, &task.id, sender) else {
        return Err(format!(
            "当前没有可用 worktree 槽位 target={}",
            effective_target_branch.as_deref().unwrap_or("unknown")
        ));
    };

    emit_stdout_log(
        sender,
        format!(
            "[WORKTREE] 执行前预分配完成 task={} slot={} path={} target={}",
            task.id,
            slot.id,
            slot.path,
            effective_target_branch.as_deref().unwrap_or("unknown")
        ),
    );
    Ok(Some(slot.path))
}

/// 模块内部可见的 claim_worktree_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn claim_worktree_path(path: &str) -> bool {
    let normalized = normalize_path(path);
    let Ok(mut claimed) = claimed_worktrees().lock() else {
        return false;
    };
    if claimed.contains(&normalized) {
        return false;
    }
    claimed.insert(normalized);
    true
}

/// 模块内部可见的 release_claimed_worktree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn release_claimed_worktree(path: &str) {
    let normalized = normalize_path(path);
    if let Ok(mut claimed) = claimed_worktrees().lock() {
        claimed.remove(&normalized);
    }
}

fn mark_slot_state(
    repo_root: &str,
    slot_id: &str,
    state: WorktreeState,
    leased_task_id: Option<String>,
    taint_reason: Option<String>,
) {
    let Ok(mut pools) = worktree_pools().lock() else {
        return;
    };
    let Some(pool) = pools.get_mut(repo_root) else {
        return;
    };
    if let Some(slot) = pool.slots.iter_mut().find(|slot| slot.id == slot_id) {
        slot.state = state;
        slot.leased_task_id = leased_task_id;
        slot.taint_reason = taint_reason;
    }
}

fn mark_task_slot_tainted(project_path: &str, task_id: &str, reason: &str) {
    let Some(repo_root) = synchronized_repo_root(project_path, None) else {
        return;
    };
    let slot_id = {
        let Ok(pools) = worktree_pools().lock() else {
            return;
        };
        let Some(pool) = pools.get(&repo_root) else {
            return;
        };
        pool.task_slots.get(task_id).cloned()
    };
    if let Some(slot_id) = slot_id {
        mark_slot_state(
            &repo_root,
            &slot_id,
            WorktreeState::Tainted,
            None,
            Some(reason.to_string()),
        );
    }
}

fn release_task_worktree_slot(project_path: &str, task_id: &str) {
    let Some(repo_root) = synchronized_repo_root(project_path, None) else {
        return;
    };
    let Ok(mut pools) = worktree_pools().lock() else {
        return;
    };
    let Some(pool) = pools.get_mut(&repo_root) else {
        return;
    };
    let Some(slot_id) = pool.task_slots.remove(task_id) else {
        return;
    };
    if let Some(slot) = pool.slots.iter_mut().find(|slot| slot.id == slot_id) {
        match slot.state {
            WorktreeState::Tainted | WorktreeState::Dead => {}
            _ => {
                slot.state = WorktreeState::Idle;
                slot.leased_task_id = None;
                slot.taint_reason = None;
            }
        }
    }
}

/// 模块内部可见的 resolve_task_execution_workspace 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn resolve_task_execution_workspace(
    task: &Task,
    project_path: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<(super::state::SelectedExecutionWorkspace, WorktreeClaimGuard), String> {
    let selected_path = task
        .selected_worktree_path
        .clone()
        .map(|path| normalize_path(&path))
        .filter(|path| Path::new(path).exists());
    let normalized_project_path = normalize_path(project_path);

    let effective_target_branch = resolve_merge_target_branch(task, project_path);

    if effective_target_branch.is_some() {
        let use_managed_slot = selected_path
            .as_deref()
            .is_none_or(|path| is_managed_worktree_path(&normalized_project_path, path));
        if use_managed_slot {
            if let Some(slot) = acquire_task_worktree(project_path, &task.id, sender) {
                let prepared = match prepare_task_worktree_for_execution(&slot, &task.id, sender) {
                    Ok(prepared) => prepared,
                    Err(err) => {
                        mark_task_slot_tainted(project_path, &task.id, &err);
                        return Err(err);
                    }
                };
                let claimed = claim_worktree_path(&prepared.path);
                let guard = WorktreeClaimGuard::new(if claimed {
                    Some(prepared.path.clone())
                } else {
                    None
                });
                return Ok((
                    super::state::SelectedExecutionWorkspace {
                        slot_id: Some(prepared.id.clone()),
                        execution_path: prepared.path.clone(),
                        selected_worktree_path: Some(prepared.path.clone()),
                        selected_worktree_branch: Some(prepared.branch.clone()),
                        merge_target_branch: effective_target_branch.clone(),
                        project_path: normalized_project_path,
                    },
                    guard,
                ));
            }
            return Err(format!("缺少可用 worktree 槽位，无法执行任务: {}", task.id));
        }
    }

    let execution_path = selected_path.clone().unwrap_or_else(|| normalized_project_path.clone());
    let selected_worktree_branch = selected_path
        .as_deref()
        .and_then(git_branch_name)
        .filter(|branch| !branch.trim().is_empty() && branch != "HEAD")
        .or_else(|| {
            task.merge_source_branch
                .clone()
                .filter(|branch| !branch.trim().is_empty() && branch != "HEAD")
        });
    let claimed = selected_path.as_deref().map(claim_worktree_path).unwrap_or(false);
    let guard = WorktreeClaimGuard::new(if claimed { selected_path.clone() } else { None });

    Ok((
        super::state::SelectedExecutionWorkspace {
            slot_id: None,
            execution_path,
            selected_worktree_path: selected_path,
            selected_worktree_branch,
            merge_target_branch: effective_target_branch,
            project_path: normalized_project_path,
        },
        guard,
    ))
}

/// 公开的 recycle_task_worktree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn recycle_task_worktree(
    project_path: &str,
    task_id: &str,
    taint_reason: Option<String>,
) -> Result<(), String> {
    let Some(repo_root) = ensure_repo_pool(project_path, None) else {
        return Ok(());
    };

    let (slot_id, slot_path, base_branch) = {
        let Ok(mut pools) = worktree_pools().lock() else {
            return Err("worktree 池锁定失败".to_string());
        };
        let Some(pool) = pools.get_mut(&repo_root) else {
            return Ok(());
        };
        let Some(slot_id) = pool.task_slots.remove(task_id) else {
            return Ok(());
        };
        let Some(slot) = pool.slots.iter_mut().find(|slot| slot.id == slot_id) else {
            return Ok(());
        };
        slot.state =
            if taint_reason.is_some() { WorktreeState::Tainted } else { WorktreeState::Recycling };
        slot.taint_reason = taint_reason.clone();
        slot.leased_task_id = None;
        (slot_id, slot.path.clone(), slot.base_branch.clone())
    };

    abort_git_in_progress_states(&slot_path, None);
    let reset_output = run_git_maintenance(&slot_path, &["reset", "--hard", &base_branch])?;
    let clean_output = run_git_maintenance(&slot_path, &["clean", "-fd"])?;
    let clean = reset_output.status.success()
        && clean_output.status.success()
        && git_is_clean(&slot_path).unwrap_or(false);

    let Ok(mut pools) = worktree_pools().lock() else {
        return Err("worktree 池锁定失败".to_string());
    };
    let Some(pool) = pools.get_mut(&repo_root) else {
        return Ok(());
    };
    if let Some(slot) = pool.slots.iter_mut().find(|slot| slot.id == slot_id) {
        slot.state = if clean { WorktreeState::Idle } else { WorktreeState::Dead };
        slot.taint_reason = if clean {
            None
        } else {
            Some("worktree 清理失败，已标记为 dead".to_string())
        };
    }
    Ok(())
}

/// 公开的 recycle_task_worktree_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn recycle_task_worktree_async(
    project_path: String,
    task_id: String,
    taint_reason: Option<String>,
) -> (String, Result<(), String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id_for_error = task_id.clone();
        tokio::task::spawn_blocking(move || {
            let result = recycle_task_worktree(&project_path, &task_id, taint_reason);
            (task_id, result)
        })
        .await
        .unwrap_or_else(|error| {
            (task_id_for_error, Err(format!("worktree 回收线程异常: {}", error)))
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let result = recycle_task_worktree(&project_path, &task_id, taint_reason);
        (task_id, result)
    }
}

/// 公开的 release_task_worktree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn release_task_worktree(project_path: &str, task_id: &str) -> Result<(), String> {
    release_task_worktree_slot(project_path, task_id);
    Ok(())
}

/// 公开的 release_task_worktree_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn release_task_worktree_async(
    project_path: String,
    task_id: String,
) -> (String, Result<(), String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id_for_error = task_id.clone();
        tokio::task::spawn_blocking(move || {
            let result = release_task_worktree(&project_path, &task_id);
            (task_id, result)
        })
        .await
        .unwrap_or_else(|error| {
            (task_id_for_error, Err(format!("worktree 释放线程异常: {}", error)))
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let result = release_task_worktree(&project_path, &task_id);
        (task_id, result)
    }
}

fn reset_all_managed_worktrees_internal(
    project_path: &str,
    clean_untracked: bool,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<usize, String> {
    let Some(repo_root) = ensure_repo_pool(project_path, sender) else {
        return Ok(0);
    };

    let base_branch = git_branch_name(&repo_root)
        .filter(|branch| branch != "HEAD")
        .unwrap_or_else(|| "main".to_string());

    emit_stdout_log(
        sender,
        format!(
            "[WORKTREE CLEANUP] 开始一键清理 repo_root={} base_branch={} clean_untracked={}",
            repo_root, base_branch, clean_untracked
        ),
    );

    let output =
        run_git_maintenance_logged(sender, &repo_root, &["worktree", "list", "--porcelain"])?;
    if !output.status.success() {
        let error = format!("读取 worktree 列表失败: {}", git_output_failure_detail(&output));
        emit_stderr_log(sender, format!("[WORKTREE CLEANUP] {}", error));
        return Err(error);
    }

    let repo_root_normalized = normalize_path(&repo_root);
    let mut reset_count = 0usize;
    for entry in parse_worktree_list(&String::from_utf8_lossy(&output.stdout)) {
        let normalized = normalize_path(&entry.path);
        if normalized == repo_root_normalized || !Path::new(&normalized).is_dir() {
            continue;
        }
        if entry.branch.as_deref().is_none_or(|branch| branch == "HEAD") {
            continue;
        }

        emit_stdout_log(
            sender,
            format!(
                "[WORKTREE CLEANUP] 处理槽位 path={} branch={}",
                normalized,
                entry.branch.as_deref().unwrap_or("HEAD")
            ),
        );

        abort_git_in_progress_states(&normalized, sender);
        let reset_output =
            run_git_maintenance_logged(sender, &normalized, &["reset", "--hard", &base_branch])?;
        if !reset_output.status.success() {
            let error = format!(
                "重置 worktree 失败 path={} detail={}",
                normalized,
                git_output_failure_detail(&reset_output)
            );
            emit_stderr_log(sender, format!("[WORKTREE CLEANUP] {}", error));
            return Err(error);
        }

        if clean_untracked {
            let clean_output = run_git_maintenance_logged(sender, &normalized, &["clean", "-fd"])?;
            if !clean_output.status.success() {
                let error = format!(
                    "清理 worktree 失败 path={} detail={}",
                    normalized,
                    git_output_failure_detail(&clean_output)
                );
                emit_stderr_log(sender, format!("[WORKTREE CLEANUP] {}", error));
                return Err(error);
            }
        }
        emit_stdout_log(sender, format!("[WORKTREE CLEANUP] 槽位已重置 path={}", normalized));
        reset_count = reset_count.saturating_add(1);
    }

    let _ = ensure_repo_pool(project_path, sender);
    emit_stdout_log(
        sender,
        format!("[WORKTREE CLEANUP] 一键清理完成，共处理 {} 个槽位", reset_count),
    );
    Ok(reset_count)
}

/// 公开的 reset_all_managed_worktrees 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn reset_all_managed_worktrees(
    project_path: &str,
    clean_untracked: bool,
) -> Result<usize, String> {
    reset_all_managed_worktrees_internal(project_path, clean_untracked, None)
}

/// 公开的 reset_all_managed_worktrees_with_logs 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn reset_all_managed_worktrees_with_logs(
    project_path: &str,
    clean_untracked: bool,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<usize, String> {
    reset_all_managed_worktrees_internal(project_path, clean_untracked, sender)
}

/// 公开的 reset_all_managed_worktrees_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn reset_all_managed_worktrees_async(
    project_path: String,
    clean_untracked: bool,
) -> impl std::future::Future<Output = Result<usize, String>> {
    reset_all_managed_worktrees_async_with_logs(project_path, clean_untracked, None)
}

/// 公开的 reset_all_managed_worktrees_async_with_logs 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn reset_all_managed_worktrees_async_with_logs(
    project_path: String,
    clean_untracked: bool,
    log_sender: Option<Sender<TaskLogStream>>,
) -> Result<usize, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::task::spawn_blocking(move || {
            reset_all_managed_worktrees_with_logs(
                &project_path,
                clean_untracked,
                log_sender.as_ref(),
            )
        })
        .await
        .unwrap_or_else(|error| Err(format!("worktree 一键清理线程异常: {}", error)))
    }
    #[cfg(target_arch = "wasm32")]
    {
        reset_all_managed_worktrees_with_logs(
            &project_path,
            clean_untracked,
            log_sender.as_ref(),
        )
    }
}

fn delete_all_managed_worktrees_internal(
    project_path: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<usize, String> {
    let Some(repo_root) = ensure_repo_pool(project_path, sender) else {
        return Ok(0);
    };

    emit_stdout_log(
        sender,
        format!("[WORKTREE DELETE] 开始删除所有受管 worktree repo_root={}", repo_root),
    );

    let worktree_list_output =
        run_git_maintenance_logged(sender, &repo_root, &["worktree", "list", "--porcelain"])?;
    if !worktree_list_output.status.success() {
        let error =
            format!("读取 worktree 列表失败: {}", git_output_failure_detail(&worktree_list_output));
        emit_stderr_log(sender, format!("[WORKTREE DELETE] {}", error));
        return Err(error);
    }

    let managed_entries =
        parse_worktree_list(&String::from_utf8_lossy(&worktree_list_output.stdout))
            .into_iter()
            .filter_map(|entry| {
                let normalized = normalize_path(&entry.path);
                if !is_managed_worktree_path(&repo_root, &normalized)
                    || !Path::new(&normalized).exists()
                {
                    return None;
                }
                Some((normalized, entry.branch))
            })
            .collect::<Vec<_>>();

    if managed_entries.is_empty() {
        emit_stdout_log(sender, "[WORKTREE DELETE] 没有可删除的受管 worktree".to_string());
        return Ok(0);
    }

    {
        let Ok(pools) = worktree_pools().lock() else {
            return Err("worktree 池锁定失败".to_string());
        };
        if let Some(pool) = pools.get(&repo_root) {
            if !pool.task_slots.is_empty() {
                let error = format!(
                    "仍有 {} 个任务占用 worktree，请先停止执行并释放槽位后再删除",
                    pool.task_slots.len()
                );
                emit_stderr_log(sender, format!("[WORKTREE DELETE] {}", error));
                return Err(error);
            }
            if pool
                .slots
                .iter()
                .any(|slot| matches!(slot.state, WorktreeState::Busy | WorktreeState::Recycling))
            {
                let error = "存在执行中或回收中的 worktree，请稍后再试".to_string();
                emit_stderr_log(sender, format!("[WORKTREE DELETE] {}", error));
                return Err(error);
            }
        }
    }

    {
        let Ok(claimed) = claimed_worktrees().lock() else {
            return Err("worktree 占用锁定失败".to_string());
        };
        if managed_entries.iter().any(|(path, _)| claimed.contains(path)) {
            let error = "存在正在使用中的 worktree，请稍后再试".to_string();
            emit_stderr_log(sender, format!("[WORKTREE DELETE] {}", error));
            return Err(error);
        }
    }

    let mut deleted_count = 0usize;
    let deleted_paths = managed_entries.iter().map(|(path, _)| path.clone()).collect::<Vec<_>>();
    for (path, branch) in managed_entries {
        emit_stdout_log(
            sender,
            format!(
                "[WORKTREE DELETE] 删除槽位 path={} branch={}",
                path,
                branch.as_deref().unwrap_or("HEAD")
            ),
        );
        abort_git_in_progress_states(&path, sender);
        let remove_output =
            run_git_logged(sender, &repo_root, &["worktree", "remove", "--force", &path])?;
        if !remove_output.status.success() {
            let error = format!(
                "删除 worktree 失败 path={} detail={}",
                path,
                git_output_failure_detail(&remove_output)
            );
            emit_stderr_log(sender, format!("[WORKTREE DELETE] {}", error));
            return Err(error);
        }
        if let Some(branch) = branch.filter(|value| value != "HEAD" && !value.trim().is_empty()) {
            let branch_output = run_git_logged(sender, &repo_root, &["branch", "-D", &branch])?;
            if !branch_output.status.success() {
                emit_stderr_log(
                    sender,
                    format!(
                        "[WORKTREE DELETE] 删除分支失败 branch={} detail={}",
                        branch,
                        git_output_failure_detail(&branch_output)
                    ),
                );
            }
        }
        emit_stdout_log(sender, format!("[WORKTREE DELETE] 槽位已删除 path={}", path));
        deleted_count = deleted_count.saturating_add(1);
    }

    if let Ok(mut claimed) = claimed_worktrees().lock() {
        for path in &deleted_paths {
            claimed.remove(path);
        }
    }
    if let Ok(mut pools) = worktree_pools().lock()
        && let Some(pool) = pools.get_mut(&repo_root)
    {
        pool.slots.clear();
        pool.task_slots.clear();
        pool.merge_target_locks.clear();
        pool.last_synced_at_ms = 0;
    }

    emit_stdout_log(sender, format!("[WORKTREE DELETE] 删除完成，共处理 {} 个槽位", deleted_count));
    Ok(deleted_count)
}

/// 公开的 delete_all_managed_worktrees 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn delete_all_managed_worktrees(project_path: &str) -> Result<usize, String> {
    delete_all_managed_worktrees_internal(project_path, None)
}

/// 公开的 delete_all_managed_worktrees_with_logs 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn delete_all_managed_worktrees_with_logs(
    project_path: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<usize, String> {
    delete_all_managed_worktrees_internal(project_path, sender)
}

/// 公开的 delete_all_managed_worktrees_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn delete_all_managed_worktrees_async(
    project_path: String,
) -> impl std::future::Future<Output = Result<usize, String>> {
    delete_all_managed_worktrees_async_with_logs(project_path, None)
}

/// 公开的 delete_all_managed_worktrees_async_with_logs 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn delete_all_managed_worktrees_async_with_logs(
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> Result<usize, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::task::spawn_blocking(move || {
            delete_all_managed_worktrees_with_logs(&project_path, log_sender.as_ref())
        })
        .await
        .unwrap_or_else(|error| Err(format!("worktree 批量删除线程异常: {}", error)))
    }
    #[cfg(target_arch = "wasm32")]
    {
        delete_all_managed_worktrees_with_logs(&project_path, log_sender.as_ref())
    }
}

fn commit_merge_all_worktrees_internal(
    project_path: &str,
    tasks: &[Task],
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<usize, String> {
    let repo_root = git_repo_root(project_path)
        .ok_or_else(|| "当前目录不在 git 仓库中，无法批量合并".to_string())?;
    let repo_root = normalize_path(&repo_root);
    let base_branch = git_branch_name(&repo_root)
        .filter(|branch| branch != "HEAD")
        .unwrap_or_else(|| "main".to_string());

    emit_stdout_log(
        sender,
        format!(
            "[WORKTREE MERGE] 开始批量合并 repo_root={} base_branch={}",
            repo_root, base_branch
        ),
    );

    let worktree_list_output =
        run_git_maintenance_logged(sender, &repo_root, &["worktree", "list", "--porcelain"])?;
    if !worktree_list_output.status.success() {
        let error =
            format!("读取 worktree 列表失败: {}", git_output_failure_detail(&worktree_list_output));
        emit_stderr_log(sender, format!("[WORKTREE MERGE] {}", error));
        return Err(error);
    }
    let worktrees = parse_worktree_list(&String::from_utf8_lossy(&worktree_list_output.stdout));
    if worktrees.is_empty() {
        return Err("没有可用的 worktree".to_string());
    }

    let checkout_output = run_git_logged(sender, &repo_root, &["checkout", &base_branch])?;
    if !checkout_output.status.success() {
        let error = format!(
            "切换主分支失败 branch={} detail={}",
            base_branch,
            git_output_failure_detail(&checkout_output)
        );
        emit_stderr_log(sender, format!("[WORKTREE MERGE] {}", error));
        return Err(error);
    }

    let mut merged_branches = Vec::new();
    let mut failed_commit_branches = Vec::new();
    let mut failed_merge_branches = Vec::new();

    for entry in worktrees {
        let worktree_path = normalize_path(&entry.path);
        if worktree_path == repo_root || !Path::new(&worktree_path).is_dir() {
            continue;
        }

        let Some(branch_name) =
            entry.branch.filter(|branch| branch != "HEAD" && !branch.is_empty())
        else {
            continue;
        };

        emit_stdout_log(
            sender,
            format!("[WORKTREE MERGE] 处理分支 {} path={}", branch_name, worktree_path),
        );

        let add_output = run_git_logged(sender, &worktree_path, &["add", "."])?;
        if !add_output.status.success() {
            failed_commit_branches.push(format!(
                "{}(git add 失败: {})",
                branch_name,
                git_output_failure_detail(&add_output)
            ));
            continue;
        }

        if git_has_staged_changes(&worktree_path)? {
            let commit_message = tasks
                .iter()
                .find(|task| task.merge_source_branch.as_deref() == Some(branch_name.as_str()))
                .map(|task| build_task_commit_message(task, &format!("提交 {}", branch_name)))
                .unwrap_or_else(|| format!("提交 {}", branch_name));
            let commit_output = run_git_logged(
                sender,
                &worktree_path,
                &["commit", "--no-verify", "-m", &commit_message],
            )?;
            if !commit_output.status.success() {
                failed_commit_branches.push(format!(
                    "{}({})",
                    branch_name,
                    git_output_failure_detail(&commit_output)
                ));
                continue;
            }
        }

        let checkout_output = run_git_logged(sender, &repo_root, &["checkout", &base_branch])?;
        if !checkout_output.status.success() {
            let error = format!(
                "切回主分支失败 branch={} detail={}",
                base_branch,
                git_output_failure_detail(&checkout_output)
            );
            emit_stderr_log(sender, format!("[WORKTREE MERGE] {}", error));
            return Err(error);
        }

        let branch_ref = format!("refs/heads/{}", branch_name);
        let show_ref_output =
            run_git_logged(sender, &repo_root, &["show-ref", "--verify", "--quiet", &branch_ref])?;
        if !show_ref_output.status.success() {
            continue;
        }

        let merge_base_output = run_git_logged(
            sender,
            &repo_root,
            &["merge-base", "--is-ancestor", &branch_name, &base_branch],
        )?;
        if matches!(merge_base_output.status.code(), Some(0)) {
            continue;
        }
        if !matches!(merge_base_output.status.code(), Some(1)) {
            failed_merge_branches.push(format!(
                "{}(merge-base 检查失败: {})",
                branch_name,
                git_output_failure_detail(&merge_base_output)
            ));
            continue;
        }

        let merge_output = run_git_logged(
            sender,
            &repo_root,
            &["merge", "--no-verify", "--no-edit", "--no-stat", &branch_name],
        )?;
        if merge_output.status.success() {
            merged_branches.push(branch_name);
            continue;
        }

        failed_merge_branches.push(format!(
            "{}({})",
            branch_name,
            git_output_failure_detail(&merge_output)
        ));
        let abort_output = run_git_logged(sender, &repo_root, &["merge", "--abort"])?;
        if !abort_output.status.success() {
            let _ = run_git_logged(sender, &repo_root, &["reset", "--merge"]);
        }
    }

    let checkout_output = run_git_logged(sender, &repo_root, &["checkout", &base_branch])?;
    if !checkout_output.status.success() {
        let error = format!(
            "恢复主分支失败 branch={} detail={}",
            base_branch,
            git_output_failure_detail(&checkout_output)
        );
        emit_stderr_log(sender, format!("[WORKTREE MERGE] {}", error));
        return Err(error);
    }

    let _ = ensure_repo_pool(&repo_root, sender);
    if failed_commit_branches.is_empty() && failed_merge_branches.is_empty() {
        emit_stdout_log(
            sender,
            format!("[WORKTREE MERGE] 批量合并完成，共合并 {} 个分支", merged_branches.len()),
        );
        return Ok(merged_branches.len());
    }

    let error = format!(
        "批量合并完成但存在失败 | merged={} | failed_commit={} [{}] | failed_merge={} [{}] | base_branch={}",
        merged_branches.len(),
        failed_commit_branches.len(),
        failed_commit_branches.join(", "),
        failed_merge_branches.len(),
        failed_merge_branches.join(", "),
        base_branch
    );
    emit_stderr_log(sender, format!("[WORKTREE MERGE] {}", error));
    Err(error)
}

/// 公开的 commit_merge_all_worktrees 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn commit_merge_all_worktrees(project_path: &str, tasks: &[Task]) -> Result<usize, String> {
    commit_merge_all_worktrees_internal(project_path, tasks, None)
}

/// 公开的 commit_merge_all_worktrees_with_logs 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn commit_merge_all_worktrees_with_logs(
    project_path: &str,
    tasks: &[Task],
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<usize, String> {
    commit_merge_all_worktrees_internal(project_path, tasks, sender)
}

/// 公开的 commit_merge_all_worktrees_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn commit_merge_all_worktrees_async(
    project_path: String,
    tasks: Vec<Task>,
) -> impl std::future::Future<Output = Result<usize, String>> {
    commit_merge_all_worktrees_async_with_logs(project_path, tasks, None)
}

/// 公开的 commit_merge_all_worktrees_async_with_logs 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn commit_merge_all_worktrees_async_with_logs(
    project_path: String,
    tasks: Vec<Task>,
    log_sender: Option<Sender<TaskLogStream>>,
) -> Result<usize, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::task::spawn_blocking(move || {
            commit_merge_all_worktrees_with_logs(&project_path, &tasks, log_sender.as_ref())
        })
        .await
        .unwrap_or_else(|error| Err(format!("worktree 批量合并线程异常: {}", error)))
    }
    #[cfg(target_arch = "wasm32")]
    {
        commit_merge_all_worktrees_with_logs(&project_path, &tasks, log_sender.as_ref())
    }
}

/// 公开的 cleanup_worktree_slot 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[allow(dead_code)]
pub fn cleanup_worktree_slot(
    project_path: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<(), String> {
    let Some(repo_root) = ensure_repo_pool(project_path, sender) else {
        return Ok(());
    };

    let remove_candidate = {
        let Ok(pools) = worktree_pools().lock() else {
            return Err("worktree 池锁定失败".to_string());
        };
        let Some(pool) = pools.get(&repo_root) else {
            return Ok(());
        };
        let active_slots = pool.task_slots.len();
        let idle_slots = pool.slots.iter().filter(|slot| slot.state == WorktreeState::Idle).count();
        let desired_capacity = active_slots.max(1);
        let target_idle = desired_capacity;
        let max_worktrees = desired_capacity.saturating_mul(2);

        if pool.slots.len() <= max_worktrees && idle_slots <= target_idle {
            return Ok(());
        }

        pool.slots
            .iter()
            .find(|slot| slot.state == WorktreeState::Idle)
            .map(|slot| (slot.id.clone(), slot.path.clone(), slot.branch.clone()))
    };

    let Some((slot_id, path, branch)) = remove_candidate else {
        return Ok(());
    };

    let normalized_path = normalize_path(&path);
    let remove_output =
        run_git_logged(sender, &repo_root, &["worktree", "remove", "--force", &normalized_path])?;
    if !remove_output.status.success() {
        emit_stderr_log(
            sender,
            format!(
                "[WORKTREE] 移除空闲工作区失败 slot={} path={} branch={}",
                slot_id, normalized_path, branch
            ),
        );
        return Ok(());
    }

    let _ = run_git_logged(sender, &repo_root, &["branch", "-D", &branch]);

    if let Ok(mut pools) = worktree_pools().lock()
        && let Some(pool) = pools.get_mut(&repo_root)
    {
        pool.slots.retain(|slot| slot.id != slot_id);
        pool.task_slots.retain(|_, mapped_slot_id| mapped_slot_id != &slot_id);
    }

    Ok(())
}

#[cfg(test)]
#[path = "worktree_admin_tests.rs"]
mod worktree_admin_tests;
