//! 任务执行器的 runner.rs 子模块。
//!
//! 该模块聚焦任务运行过程中的一个局部职责，供执行器入口组合调用。注释说明边界、错误传播和平台差异，避免调用方需要阅读完整执行链才能理解行为。

#[cfg(not(target_arch = "wasm32"))]
use super::backend_output::build_model_prompt;
#[cfg(not(target_arch = "wasm32"))]
use super::git::{
    abort_git_in_progress_states, git_has_staged_changes, git_output_failure_detail, git_repo_root,
    git_worktree_path_for_branch, run_git_logged, run_git_logged_with_timeout,
    run_git_maintenance_logged, verify_local_branch_refs,
};
#[cfg(not(target_arch = "wasm32"))]
use super::process_utils::{emit_stderr_log, emit_stdout_log};
use super::state::TaskLogStream;
#[cfg(not(target_arch = "wasm32"))]
use super::state::{
    GIT_MERGE_COMMAND_TIMEOUT_SECS, GIT_MERGE_RETRY_DELAY_SECS, GIT_SOURCE_BRANCH_TAG,
    GIT_SUMMARY_TAG, GIT_TARGET_BRANCH_TAG, GIT_WORKTREE_PATH_TAG, SelectedExecutionWorkspace,
};
#[cfg(not(target_arch = "wasm32"))]
use super::worktree_admin::resolve_task_execution_workspace;
#[cfg(not(target_arch = "wasm32"))]
use super::worktree_pool::{
    lock_merge_target, task_merge_lock_holder, task_worktree_path, unlock_merge_target,
};
use super::*;
use crate::app::task::normalize_task_acp_agent_input;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;

#[cfg(not(target_arch = "wasm32"))]
fn flush_gateway_output_lines(
    pending: &mut String,
    sender: Option<&Sender<TaskLogStream>>,
    force: bool,
) {
    while let Some(pos) = pending.find('\n') {
        let line = pending[..pos].trim_end_matches('\r').to_string();
        pending.drain(..pos + 1);
        if !line.trim().is_empty() {
            emit_stdout_log(sender, line);
        }
    }

    if force {
        let line = pending.trim();
        if !line.is_empty() {
            emit_stdout_log(sender, line.to_string());
        }
        pending.clear();
    }
}

/// 公开的 execute_gateway_prompt_with_streaming 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn execute_gateway_prompt_with_streaming(
    session_id: &str,
    execution_path: &str,
    model: &str,
    prompt: &str,
    acp_agent: Option<String>,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<String, String> {
    let endpoint = crate::app::config::gateway_client_endpoint();
    let acp_enabled = acp_agent.is_some();
    let route_label = if acp_enabled { "acp" } else { "model" };
    let selected_agent = acp_agent.clone().unwrap_or_else(|| "disabled".to_string());

    emit_stdout_log(
        sender,
        format!(
            "[GATEWAY] endpoint={} session={} route={} agent={} model={} cwd={}",
            endpoint.describe(),
            session_id,
            route_label,
            selected_agent,
            model,
            execution_path
        ),
    );

    let mut output = String::new();
    let mut pending = String::new();
    let mut stream_error: Option<String> = None;
    let mut options = serde_json::Map::new();
    options.insert("acp_test".to_string(), json!(acp_enabled));
    options.insert("cwd".to_string(), json!(execution_path));
    if acp_enabled {
        options.insert("acp_agent".to_string(), json!(acp_agent));
        options.insert("acp_force_new_session".to_string(), json!(true));
        options.insert("acp_history_strategy".to_string(), json!("discard"));
        options.insert("acp_history_recent_count".to_string(), json!(1));
    }
    let request = vw_gateway_client::GatewayChatStreamRequest {
        session_id: Some(session_id.into()),
        messages: vec![json!({ "role": "user", "content": prompt })],
        system: None,
        model: (model != "auto").then(|| model.to_string()),
        agent: None,
        allowed_tools: None,
        acp_agent: acp_enabled.then_some(acp_agent).flatten(),
        acp_allowed_tools: None,
        options: Some(serde_json::Value::Object(options)),
    };

    let result = vw_gateway_client::GatewayClient::stream_chat_blocking(
        &endpoint,
        Some(execution_path),
        &request,
        |event| match event {
            vw_gateway_client::GatewayChatStreamEvent::Delta(delta) => {
                output.push_str(&delta);
                pending.push_str(&delta);
                flush_gateway_output_lines(&mut pending, sender, false);
                true
            }
            vw_gateway_client::GatewayChatStreamEvent::Done { finish_reason, .. } => {
                flush_gateway_output_lines(&mut pending, sender, true);
                emit_stdout_log(
                    sender,
                    format!(
                        "[GATEWAY] done finish_reason={}",
                        finish_reason.unwrap_or_else(|| "unknown".to_string())
                    ),
                );
                true
            }
            vw_gateway_client::GatewayChatStreamEvent::Error(error) => {
                flush_gateway_output_lines(&mut pending, sender, true);
                emit_stderr_log(sender, format!("[GATEWAY] error {}", error));
                stream_error = Some(error);
                false
            }
            vw_gateway_client::GatewayChatStreamEvent::Other(payload) => {
                match payload.get("type").and_then(serde_json::Value::as_str) {
                    Some("chat.step_start") => {
                        let step_index = payload
                            .get("step_index")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or_default();
                        emit_stdout_log(sender, format!("[GATEWAY] step_start {}", step_index));
                    }
                    Some("chat.step_finish") => {
                        let step_index = payload
                            .get("step_index")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or_default();
                        emit_stdout_log(sender, format!("[GATEWAY] step_finish {}", step_index));
                    }
                    _ => {}
                }
                true
            }
        },
    );

    if let Err(error) = result {
        let _ = sender.map(|stream| {
            stream.send(TaskLogStream::ExitStatus { success: false, code: None, signal: None })
        });
        return Err(format!("gateway 调度失败: {}", error));
    }

    if let Some(error) = stream_error {
        let _ = sender.map(|stream| {
            stream.send(TaskLogStream::ExitStatus { success: false, code: None, signal: None })
        });
        return Err(error);
    }

    flush_gateway_output_lines(&mut pending, sender, true);
    let _ = sender.map(|stream| {
        stream.send(TaskLogStream::ExitStatus { success: true, code: Some(0), signal: None })
    });
    Ok(output)
}

/// 公开的 execute_gateway_prompt_with_streaming 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn execute_gateway_prompt_with_streaming(
    _session_id: &str,
    _execution_path: &str,
    _model: &str,
    _prompt: &str,
    _acp_agent: Option<String>,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<String, String> {
    let _ = sender.map(|stream| {
        stream.send(TaskLogStream::ExitStatus { success: false, code: None, signal: None })
    });
    Err("WASM 平台暂不支持通过网关执行流式任务".to_string())
}

/// 模块内部可见的 resolve_task_execution_acp_agent 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn resolve_task_execution_acp_agent(task: &Task) -> Option<String> {
    task.acp_agent.as_deref().and_then(normalize_task_acp_agent_input)
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_with_selected_backend(
    task: &Task,
    execution_path: &str,
    model: &str,
    prompt: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<String, String> {
    execute_gateway_prompt_with_streaming(
        &task_session_id(&task.id),
        execution_path,
        model,
        prompt,
        resolve_task_execution_acp_agent(task),
        sender,
    )
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct GitPrepareOutcome {
    summary: String,
    source_branch: Option<String>,
    target_branch: Option<String>,
    worktree_path: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn format_git_summary(add: &str, commit: &str, merge: &str) -> String {
    format!("Git动作摘要: add={}; commit={}; merge={}", add, commit, merge)
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_git_prepare_actions(
    task: &Task,
    workspace: &SelectedExecutionWorkspace,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<GitPrepareOutcome, String> {
    let mut _add_status = "跳过".to_string();
    let mut commit_status = "跳过".to_string();
    let mut merge_status = "跳过".to_string();

    if git_repo_root(&workspace.execution_path).is_none() {
        emit_stdout_log(
            sender,
            "[GIT] 当前执行目录不是 git 仓库，跳过 add/commit/merge".to_string(),
        );
        _add_status = "跳过(非git仓库)".to_string();
        commit_status = "跳过(非git仓库)".to_string();
        merge_status = "跳过(非git仓库)".to_string();
        return Ok(GitPrepareOutcome {
            summary: format_git_summary(&_add_status, &commit_status, &merge_status),
            source_branch: None,
            target_branch: workspace.merge_target_branch.clone(),
            worktree_path: workspace.selected_worktree_path.clone(),
        });
    }

    emit_stdout_log(sender, "[GIT] 开始执行 add/commit 自动流程".to_string());

    let add_output = run_git_logged(sender, &workspace.execution_path, &["add", "."])?;
    if !add_output.status.success() {
        _add_status = "失败".to_string();
        return Err(format!(
            "git add . 失败: {} | {}",
            git_output_failure_detail(&add_output),
            format_git_summary(&_add_status, &commit_status, &merge_status)
        ));
    }
    _add_status = "成功".to_string();

    if !git_has_staged_changes(&workspace.execution_path)? {
        emit_stdout_log(sender, "[GIT] 未检测到可提交变更，跳过 commit".to_string());
        commit_status = "跳过(无可提交变更)".to_string();
        merge_status = "跳过(无提交)".to_string();
        return Ok(GitPrepareOutcome {
            summary: format_git_summary(&_add_status, &commit_status, &merge_status),
            source_branch: workspace.selected_worktree_branch.clone(),
            target_branch: workspace.merge_target_branch.clone(),
            worktree_path: workspace.selected_worktree_path.clone(),
        });
    }

    let commit_message = build_task_commit_message(task, &format!("task({})", task.id));
    emit_stdout_log(sender, "[GIT] 提交将跳过 hooks (--no-verify)".to_string());
    let commit_output = run_git_logged(
        sender,
        &workspace.execution_path,
        &["commit", "--no-verify", "-m", &commit_message],
    )?;
    if !commit_output.status.success() {
        commit_status = "失败".to_string();
        return Err(format!(
            "git commit 失败: {} | {}",
            git_output_failure_detail(&commit_output),
            format_git_summary(&_add_status, &commit_status, &merge_status)
        ));
    }
    commit_status = format!("成功({})", commit_message);
    merge_status = "跳过(等待审核后再合并)".to_string();

    Ok(GitPrepareOutcome {
        summary: format_git_summary(&_add_status, &commit_status, &merge_status),
        source_branch: workspace.selected_worktree_branch.clone(),
        target_branch: workspace.merge_target_branch.clone(),
        worktree_path: workspace.selected_worktree_path.clone(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    let sender_ref = log_sender.as_ref();
    let (workspace, _claim_guard) =
        match resolve_task_execution_workspace(&task, &project_path, sender_ref) {
            Ok(result) => result,
            Err(error) => return (task.id, Err(error)),
        };

    let model = if task.model == "auto" { "auto".to_string() } else { task.model.clone() };
    let model_prompt = build_model_prompt(&task, &task.prompt);
    let execution_result = execute_task_with_selected_backend(
        &task,
        &workspace.execution_path,
        &model,
        &model_prompt,
        sender_ref,
    );

    let final_result = match execution_result {
        Ok(exec) => match execute_git_prepare_actions(&task, &workspace, sender_ref) {
            Ok(git) => {
                let mut output = format!("{}\n{}:{}", exec, GIT_SUMMARY_TAG, git.summary);
                if let Some(source) = git.source_branch {
                    output.push_str(&format!("\n{}:{}", GIT_SOURCE_BRANCH_TAG, source));
                }
                if let Some(target) = git.target_branch {
                    output.push_str(&format!("\n{}:{}", GIT_TARGET_BRANCH_TAG, target));
                }
                if let Some(worktree_path) = git.worktree_path {
                    output.push_str(&format!("\n{}:{}", GIT_WORKTREE_PATH_TAG, worktree_path));
                }
                Ok(output)
            }
            Err(git_err) => Err(format!("任务执行成功，但 Git 流程失败: {}", git_err)),
        },
        Err(exec_err) => {
            emit_stdout_log(
                sender_ref,
                format!("[GIT] 跳过 add/commit/merge，原因=执行失败: {}", exec_err),
            );
            Err(exec_err)
        }
    };

    (task.id, final_result)
}

/// 公开的 execute_task_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn execute_task_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || {
            execute_task_blocking(task, project_path, log_sender)
        })
        .await
        .unwrap_or_else(|error| (task_id, Err(format!("任务执行线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Task execution not supported on Web".to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_review_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    let sender_ref = log_sender.as_ref();
    let model = if task.model == "auto" { "auto".to_string() } else { task.model.clone() };
    let model_prompt = build_model_prompt(&task, &task.prompt);
    let execution_result = execute_task_with_selected_backend(
        &task,
        &project_path,
        &model,
        &model_prompt,
        sender_ref,
    );
    (task.id, execution_result)
}

/// 公开的 execute_task_review_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn execute_task_review_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || {
            execute_task_review_blocking(task, project_path, log_sender)
        })
        .await
        .unwrap_or_else(|error| (task_id, Err(format!("代码审核线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Code review not supported on Web".to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_merge_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    struct MergeTargetLockGuard<'a> {
        project_path: &'a str,
        task: &'a Task,
        sender: Option<&'a Sender<TaskLogStream>>,
        target_branch: &'a str,
    }

    impl Drop for MergeTargetLockGuard<'_> {
        fn drop(&mut self) {
            unlock_merge_target(self.project_path, self.task);
            emit_stdout_log(
                self.sender,
                format!(
                    "[GIT MERGE] lock released task_id={} target={} holder_after_release={}",
                    self.task.id,
                    self.target_branch,
                    task_merge_lock_holder(self.project_path, self.task)
                        .unwrap_or_else(|| "none".to_string())
                ),
            );
        }
    }

    let sender_ref = log_sender.as_ref();
    let task_id = task.id.clone();
    lock_merge_target(&project_path, &task);
    let source_branch = task.merge_source_branch.clone().unwrap_or_default();
    let target_branch = task.merge_target_branch.clone().unwrap_or_default();
    let _merge_lock_guard = MergeTargetLockGuard {
        project_path: &project_path,
        task: &task,
        sender: sender_ref,
        target_branch: &target_branch,
    };
    let merge_project_path = git_worktree_path_for_branch(&project_path, &target_branch)
        .or_else(|| task_worktree_path(&project_path, &task.id))
        .unwrap_or(project_path.clone());
    emit_stdout_log(
        sender_ref,
        format!(
            "[GIT MERGE] start task={} {} -> {} timeout={}s",
            task.id, source_branch, target_branch, GIT_MERGE_COMMAND_TIMEOUT_SECS
        ),
    );

    let merge_result = if source_branch.trim().is_empty() || target_branch.trim().is_empty() {
        emit_stdout_log(sender_ref, "[GIT MERGE] 未发现可合并分支，跳过".to_string());
        Ok("合并跳过: 无可合并分支".to_string())
    } else if source_branch == target_branch || source_branch == "HEAD" {
        emit_stdout_log(
            sender_ref,
            format!(
                "[GIT MERGE] 合并分支无效，跳过 source={} target={}",
                source_branch, target_branch
            ),
        );
        Ok(format!("合并跳过: source={} target={}", source_branch, target_branch))
    } else {
        emit_stdout_log(
            sender_ref,
            format!(
                "[GIT MERGE] selected workspace={} source={} target={}",
                merge_project_path, source_branch, target_branch
            ),
        );

        if let Err(err) = verify_local_branch_refs(
            &merge_project_path,
            &[("source", &source_branch), ("target", &target_branch)],
            sender_ref,
        ) {
            Err(err)
        } else {
            match run_git_logged(
                sender_ref,
                &merge_project_path,
                &["merge-base", "--is-ancestor", &source_branch, &target_branch],
            ) {
                Ok(output) if output.status.success() => {
                    emit_stdout_log(
                        sender_ref,
                        format!(
                            "[GIT MERGE] source 已在 target 中，跳过 source={} target={} workspace={}",
                            source_branch, target_branch, merge_project_path
                        ),
                    );
                    Ok(format!(
                        "合并跳过: source={} 已包含于 target={} workspace={}",
                        source_branch, target_branch, merge_project_path
                    ))
                }
                Ok(_) => {
                    if super::state::verbose_merge_logging() {
                        emit_stdout_log(
                            sender_ref,
                            format!("[GIT MERGE] merge_before cwd={}", merge_project_path),
                        );
                        let _ = run_git_logged(
                            sender_ref,
                            &merge_project_path,
                            &["status", "--short", "--branch"],
                        );
                        let _ = run_git_logged(
                            sender_ref,
                            &merge_project_path,
                            &["branch", "--show-current"],
                        );
                        let _ = run_git_logged(
                            sender_ref,
                            &merge_project_path,
                            &["rev-parse", "--short", "HEAD"],
                        );
                    }

                    let mut last_error: Option<String> = None;
                    for attempt in 1..=3 {
                        abort_git_in_progress_states(&merge_project_path, sender_ref);
                        let checkout_output = run_git_maintenance_logged(
                            sender_ref,
                            &merge_project_path,
                            &["checkout", &target_branch],
                        );
                        let attempt_result = match checkout_output {
                            Ok(output) if output.status.success() => {
                                let merge_message = format!(
                                    "chore(task): merge {} into {}",
                                    source_branch, target_branch
                                );
                                match run_git_logged_with_timeout(
                                    sender_ref,
                                    &merge_project_path,
                                    &[
                                        "merge",
                                        "--no-verify",
                                        "--no-edit",
                                        "--no-stat",
                                        "-m",
                                        &merge_message,
                                        &source_branch,
                                    ],
                                    Duration::from_secs(GIT_MERGE_COMMAND_TIMEOUT_SECS),
                                ) {
                                    Ok((output, elapsed)) if output.status.success() => {
                                        emit_stdout_log(
                                            sender_ref,
                                            format!(
                                                "[GIT MERGE] merge 成功 workspace={} source={} target={} elapsed_ms={}",
                                                merge_project_path,
                                                source_branch,
                                                target_branch,
                                                elapsed.as_millis()
                                            ),
                                        );
                                        Ok(format!(
                                            "Git动作摘要: add=跳过; commit=跳过; merge=成功({}->{}) workspace={}",
                                            source_branch, target_branch, merge_project_path
                                        ))
                                    }
                                    Ok((output, elapsed)) => {
                                        abort_git_in_progress_states(
                                            &merge_project_path,
                                            sender_ref,
                                        );
                                        Err(format!(
                                            "自动合并失败 workspace={} source={} target={} elapsed_ms={} detail={}",
                                            merge_project_path,
                                            source_branch,
                                            target_branch,
                                            elapsed.as_millis(),
                                            git_output_failure_detail(&output)
                                        ))
                                    }
                                    Err(err) => {
                                        abort_git_in_progress_states(
                                            &merge_project_path,
                                            sender_ref,
                                        );
                                        Err(format!(
                                            "自动合并异常 workspace={} source={} target={} err={}",
                                            merge_project_path, source_branch, target_branch, err
                                        ))
                                    }
                                }
                            }
                            Ok(output) => Err(format!(
                                "切回目标分支失败 target={} detail={}",
                                target_branch,
                                git_output_failure_detail(&output)
                            )),
                            Err(err) => Err(err),
                        };
                        match attempt_result {
                            Ok(success) => {
                                let _last_error: Option<String> = None;
                                return (task_id.clone(), Ok(success));
                            }
                            Err(err) => {
                                last_error = Some(err.clone());
                                emit_stderr_log(sender_ref, format!("[GIT MERGE] {}", err));
                                if attempt < 3 {
                                    emit_stdout_log(
                                        sender_ref,
                                        format!(
                                            "[GIT MERGE] 合并失败，{}秒后重试 attempt={}/3 error={}",
                                            GIT_MERGE_RETRY_DELAY_SECS, attempt, err
                                        ),
                                    );
                                    thread::sleep(Duration::from_secs(GIT_MERGE_RETRY_DELAY_SECS));
                                }
                            }
                        }
                    }
                    Err(last_error.unwrap_or_else(|| "合并失败".to_string()))
                }
                Err(err) => Err(format!(
                    "merge-base 检查失败 workspace={} source={} target={} err={}",
                    merge_project_path, source_branch, target_branch, err
                )),
            }
        }
    };

    if let Err(error) = &merge_result {
        emit_stderr_log(
            sender_ref,
            format!(
                "[GIT MERGE] completed task_id={} source={} target={} result=error err={}",
                task_id, source_branch, target_branch, error
            ),
        );
    }

    (task_id, merge_result)
}

/// 公开的 execute_task_merge_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn execute_task_merge_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || {
            execute_task_merge_blocking(task, project_path, log_sender)
        })
        .await
        .unwrap_or_else(|error| (task_id, Err(format!("代码合并线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Task merge not supported on Web".to_string()))
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod runner_tests;
