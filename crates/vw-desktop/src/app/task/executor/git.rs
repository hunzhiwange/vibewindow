//! 实现任务执行器的命令调度、进程输出和辅助处理。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::process_utils::{
    build_command_failure_detail, emit_stderr_log, emit_stdout_log, exit_status_signal,
    tail_chars, to_shell_command,
};
#[cfg(not(target_arch = "wasm32"))]
use super::process_utils::normalize_path;
use super::state::{GIT_MAINTENANCE_COMMAND_TIMEOUT_SECS, TaskLogStream};
#[cfg(not(target_arch = "wasm32"))]
use super::worktree_pool::parse_worktree_list;
use super::*;

/// 执行 abort_git_in_progress_states 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn abort_git_in_progress_states(cwd: &str, sender: Option<&Sender<TaskLogStream>>) {
    for args in [["merge", "--abort"], ["rebase", "--abort"], ["cherry-pick", "--abort"]] {
        let args_vec = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        match run_git(cwd, &args) {
            Ok(output) if output.status.success() => {
                emit_stdout_log(
                    sender,
                    format!(
                        "[GIT] cwd={} cmd={} result=aborted",
                        cwd,
                        to_shell_command("git", &args_vec)
                    ),
                );
            }
            Ok(output) => {
                if is_benign_abort_failure(&args, &output) {
                    continue;
                }
                let stdout = String::from_utf8_lossy(&output.stdout)
                    .replace("\r\n", "\n")
                    .replace('\r', "\n");
                let stderr = String::from_utf8_lossy(&output.stderr)
                    .replace("\r\n", "\n")
                    .replace('\r', "\n");
                emit_stderr_log(
                    sender,
                    format!(
                        "[GIT] cwd={} cmd={} result=abort_failed code={:?}",
                        cwd,
                        to_shell_command("git", &args_vec),
                        output.status.code()
                    ),
                );
                if !stdout.trim().is_empty() {
                    emit_stdout_log(sender, format!("[GIT STDOUT] {}", tail_chars(&stdout, 2000)));
                }
                if !stderr.trim().is_empty() {
                    emit_stderr_log(sender, format!("[GIT STDERR] {}", tail_chars(&stderr, 2000)));
                }
            }
            Err(error) => {
                emit_stderr_log(
                    sender,
                    format!(
                        "[GIT] cwd={} cmd={} result=abort_error err={}",
                        cwd,
                        to_shell_command("git", &args_vec),
                        error
                    ),
                );
            }
        }
    }
}

fn is_benign_abort_failure(args: &[&str], output: &std::process::Output) -> bool {
    let code = output.status.code();
    if !matches!(code, Some(1) | Some(128)) {
        return false;
    }
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    match args {
        ["merge", "--abort"] => {
            stderr.contains("merge_head")
                || stderr.contains("no merge to abort")
                || stderr.contains("there is no merge to abort")
                || stderr.contains("没有要终止的合并")
        }
        ["rebase", "--abort"] => {
            stderr.contains("no rebase in progress") || stderr.contains("没有正在进行的变基")
        }
        ["cherry-pick", "--abort"] => {
            stderr.contains("no cherry-pick or revert in progress")
                || stderr.contains("cherry-pick is not in progress")
                || stderr.contains("拣选或还原操作并未进行")
        }
        _ => false,
    }
}

fn git_maintenance_timeout() -> Duration {
    Duration::from_secs(GIT_MAINTENANCE_COMMAND_TIMEOUT_SECS)
}

/// 执行 run_git_maintenance_logged 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn run_git_maintenance_logged(
    sender: Option<&Sender<TaskLogStream>>,
    cwd: &str,
    args: &[&str],
) -> Result<std::process::Output, String> {
    let (output, _) = run_git_logged_with_timeout(sender, cwd, args, git_maintenance_timeout())?;
    Ok(output)
}

/// 执行 run_git_maintenance 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn run_git_maintenance(
    cwd: &str,
    args: &[&str],
) -> Result<std::process::Output, String> {
    let (output, _) = run_git_with_timeout(cwd, args, git_maintenance_timeout())?;
    Ok(output)
}

/// 执行 run_git 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn run_git(cwd: &str, args: &[&str]) -> Result<std::process::Output, String> {
    git_std_command()
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| format!("git 执行失败 cwd={} args={:?} err={}", cwd, args, e))
}

/// 执行 run_git_with_timeout 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn run_git_with_timeout(
    cwd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(std::process::Output, Duration), String> {
    let started_at = std::time::Instant::now();
    let mut child = git_std_command()
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git 启动失败 cwd={} args={:?} err={}", cwd, args, e))?;

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child.wait_with_output().map_err(|e| {
                    format!("git 读取输出失败 cwd={} args={:?} err={}", cwd, args, e)
                })?;
                return Ok((output, started_at.elapsed()));
            }
            Ok(None) => {
                if started_at.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "git 执行超时 cwd={} timeout={}s args={:?}",
                        cwd,
                        timeout.as_secs(),
                        args
                    ));
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("git 等待失败 cwd={} args={:?} err={}", cwd, args, e));
            }
        }
    }
}

/// 执行 run_git_logged 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn run_git_logged(
    sender: Option<&Sender<TaskLogStream>>,
    cwd: &str,
    args: &[&str],
) -> Result<std::process::Output, String> {
    let args_vec = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    emit_stdout_log(
        sender,
        format!("[GIT] cwd={} cmd={}", cwd, to_shell_command("git", &args_vec)),
    );

    let output = run_git(cwd, args)?;
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n").replace('\r', "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n").replace('\r', "\n");
    if !stdout.trim().is_empty() {
        emit_stdout_log(sender, format!("[GIT STDOUT] {}", tail_chars(&stdout, 4000)));
    }
    if !stderr.trim().is_empty() {
        emit_stderr_log(sender, format!("[GIT STDERR] {}", tail_chars(&stderr, 4000)));
    }

    emit_stdout_log(
        sender,
        format!("[GIT EXIT] success={} code={:?}", output.status.success(), output.status.code()),
    );
    Ok(output)
}

/// 执行 run_git_logged_with_timeout 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn run_git_logged_with_timeout(
    sender: Option<&Sender<TaskLogStream>>,
    cwd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(std::process::Output, Duration), String> {
    let args_vec = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    emit_stdout_log(
        sender,
        format!(
            "[GIT] cwd={} timeout={}s cmd={}",
            cwd,
            timeout.as_secs(),
            to_shell_command("git", &args_vec)
        ),
    );

    let (output, elapsed) = run_git_with_timeout(cwd, args, timeout).map_err(|error| {
        emit_stderr_log(sender, format!("[GIT TIMEOUT/ERROR] {}", error));
        error
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n").replace('\r', "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n").replace('\r', "\n");
    if !stdout.trim().is_empty() {
        emit_stdout_log(sender, format!("[GIT STDOUT] {}", tail_chars(&stdout, 4000)));
    }
    if !stderr.trim().is_empty() {
        emit_stderr_log(sender, format!("[GIT STDERR] {}", tail_chars(&stderr, 4000)));
    }

    emit_stdout_log(
        sender,
        format!(
            "[GIT EXIT] success={} code={:?} elapsed_ms={}",
            output.status.success(),
            output.status.code(),
            elapsed.as_millis()
        ),
    );
    Ok((output, elapsed))
}

/// 执行 git_branch_name 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn git_branch_name(cwd: &str) -> Option<String> {
    let output = run_git(cwd, &["rev-parse", "--abbrev-ref", "HEAD"]).ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

/// 执行 git_repo_root 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn git_repo_root(cwd: &str) -> Option<String> {
    let output = run_git(cwd, &["rev-parse", "--show-toplevel"]).ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 git_worktree_path_for_branch 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn git_worktree_path_for_branch(cwd: &str, branch: &str) -> Option<String> {
    let branch = branch.trim();
    if branch.is_empty() || branch == "HEAD" {
        return None;
    }

    let repo_root = git_repo_root(cwd)?;
    let output = run_git_maintenance(&repo_root, &["worktree", "list", "--porcelain"]).ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_worktree_list(&stdout)
        .into_iter()
        .find(|entry| entry.branch.as_deref() == Some(branch))
        .map(|entry| normalize_path(&entry.path))
}

/// 执行 git_is_clean 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn git_is_clean(cwd: &str) -> Result<bool, String> {
    let output = run_git(cwd, &["status", "--porcelain=v1"])?;
    if !output.status.success() {
        return Err("git status 返回失败".to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().is_empty())
}

/// 执行 git_has_staged_changes 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn git_has_staged_changes(cwd: &str) -> Result<bool, String> {
    let output = run_git(cwd, &["diff", "--cached", "--quiet"])?;
    match output.status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(format!("git diff --cached --quiet 失败 code={:?}", output.status.code())),
    }
}

/// 执行 git_output_failure_detail 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn git_output_failure_detail(output: &std::process::Output) -> String {
    build_command_failure_detail(
        output.status.code(),
        exit_status_signal(&output.status),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
        false,
    )
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 verify_local_branch_refs 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn verify_local_branch_refs(
    cwd: &str,
    branches: &[(&str, &str)],
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<(), String> {
    for (label, branch) in branches {
        let branch_ref = format!("refs/heads/{}", branch);
        match run_git_logged(sender, cwd, &["show-ref", "--verify", &branch_ref]) {
            Ok(output) if output.status.success() => {}
            Ok(_) => return Err(format!("仓库缺少 {} 分支引用: {}", label, branch_ref)),
            Err(err) => return Err(format!("检查 {} 分支引用失败 {}: {}", label, branch_ref, err)),
        }
    }
    Ok(())
}

/// 执行 build_review_diff_context 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub fn build_review_diff_context(
    project_path: &str,
    source_branch: Option<&str>,
    target_branch: Option<&str>,
) -> Result<(String, String, String, String, String), String> {
    const EMPTY_TREE_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

    let source_ref =
        source_branch.map(str::trim).filter(|v| !v.is_empty()).unwrap_or("HEAD").to_string();
    let target_ref =
        target_branch.map(str::trim).filter(|v| !v.is_empty()).unwrap_or("HEAD").to_string();

    let source_commit_output = run_git(project_path, &["rev-parse", &source_ref])?;
    if !source_commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&source_commit_output.stderr).trim().to_string();
        return Err(format!("git rev-parse 失败 source={} err={}", source_ref, stderr));
    }
    let commit2 = String::from_utf8_lossy(&source_commit_output.stdout).trim().to_string();
    if commit2.is_empty() {
        return Err(format!("git rev-parse 未返回提交 source={}", source_ref));
    }

    let parent_output = run_git(project_path, &["rev-list", "--parents", "-n", "1", &commit2])?;
    if !parent_output.status.success() {
        let stderr = String::from_utf8_lossy(&parent_output.stderr).trim().to_string();
        return Err(format!("git rev-list 失败 commit={} err={}", commit2, stderr));
    }
    let parent_line = String::from_utf8_lossy(&parent_output.stdout).trim().to_string();
    if parent_line.is_empty() {
        return Err(format!("git rev-list 未返回提交 commit={}", commit2));
    }

    let parents = parent_line.split_whitespace().collect::<Vec<_>>();
    let commit1 =
        if parents.len() >= 2 { parents[1].to_string() } else { EMPTY_TREE_HASH.to_string() };

    let diff_output = run_git(
        project_path,
        &[
            "-c",
            "core.autocrlf=false",
            "-c",
            "core.quotepath=false",
            "diff",
            "--no-ext-diff",
            "--unified=3",
            &commit1,
            &commit2,
            "--",
            ".",
        ],
    )?;
    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr).trim().to_string();
        return Err(format!(
            "git diff 失败 commit1={} commit2={} err={}",
            commit1, commit2, stderr
        ));
    }
    let diff =
        String::from_utf8_lossy(&diff_output.stdout).replace("\r\n", "\n").replace('\r', "\n");

    Ok((source_ref, target_ref, commit1, commit2, diff))
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod git_tests;
