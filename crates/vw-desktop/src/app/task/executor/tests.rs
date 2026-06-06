//! 任务执行器的 tests.rs 子模块。
//!
//! 该模块聚焦任务运行过程中的一个局部职责，供执行器入口组合调用。注释说明边界、错误传播和平台差异，避免调用方需要阅读完整执行链才能理解行为。

use super::process_utils::{build_command_failure_detail, normalize_path};
use super::programs::{
    ExecutorCommand, claude_binary_name, select_claude_program,
    select_opencode_program_and_prefix_args,
};
use super::runner::resolve_task_execution_acp_agent;
use super::worktree_admin::assign_task_execution_worktree;
use super::worktree_admin::resolve_task_execution_workspace;
use super::{normalize_commit_title, task_session_id};
use crate::app::task::Task;
use std::sync::{LazyLock, Mutex, MutexGuard};

static TASK_EXECUTOR_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn task_executor_env_lock() -> MutexGuard<'static, ()> {
    TASK_EXECUTOR_ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn run_git_test(cwd: &std::path::Path, args: &[&str]) {
    let output = vw_shared::shell::git_std_command()
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("git command should start");
    assert!(
        output.status.success(),
        "git {:?} failed: stdout={} stderr={}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set_os(key: &'static str, value: &std::ffi::OsStr) -> Self {
        let original = std::env::var_os(key);
        unsafe { std::env::set_var(key, value) };
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

#[test]
fn resolve_claude_program_prefers_resolved_profile_path() {
    let temp = tempfile::TempDir::new().expect("temp dir should be created");
    let bin_dir = temp.path().join("profile-bin");
    std::fs::create_dir_all(&bin_dir).expect("bin dir should be created");
    let claude_path = bin_dir.join(claude_binary_name());
    std::fs::write(&claude_path, b"#!/bin/sh\nexit 0\n").expect("claude stub should be written");

    let resolved = select_claude_program(None, None, Some(claude_path.clone()));
    assert_eq!(resolved, claude_path.to_string_lossy().to_string());
}

#[test]
fn opencode_command_uses_prompt_arg_without_stdin_pipe() {
    let cmd = ExecutorCommand::for_opencode("/tmp/project", "auto", "hello\nworld");

    let run_index = cmd
        .args
        .iter()
        .position(|arg| arg == "run")
        .expect("opencode command should include run subcommand");
    assert!(run_index <= 2);
    assert!(cmd.args.iter().all(|arg| arg != "-"));
    assert_eq!(cmd.args.last().map(String::as_str), Some("hello\nworld"));
    assert!(cmd.stdin_content.is_none());
}

#[test]
fn opencode_command_falls_back_to_bunx_package_when_binary_missing() {
    let home = tempfile::TempDir::new().expect("temp home should be created");
    let bin_dir = home.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("bin dir should be created");

    let bunx_path = bin_dir.join(if cfg!(windows) { "bunx.exe" } else { "bunx" });
    std::fs::write(&bunx_path, b"#!/bin/sh\nexit 0\n").expect("bunx stub should be written");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms =
            std::fs::metadata(&bunx_path).expect("bunx metadata should exist").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bunx_path, perms).expect("bunx permissions should be updated");
    }

    let (program, args) = select_opencode_program_and_prefix_args(
        None,
        Some(home.path()),
        None,
        Some(bunx_path.to_string_lossy().to_string()),
        None,
    );
    let cmd =
        ExecutorCommand::for_opencode_resolved("/tmp/project", "auto", "hello", program, args);

    assert_eq!(cmd.program, bunx_path.to_string_lossy().to_string());
    assert!(cmd.args.starts_with(&["opencode-ai@latest".to_string(), "run".to_string(),]));
}

#[test]
fn build_command_failure_detail_prefers_signal_text() {
    let detail = build_command_failure_detail(None, Some(9), "", "", false);
    assert_eq!(detail, "signal=9(SIGKILL)");
}

#[test]
fn normalize_commit_title_flattens_whitespace_and_limits_length() {
    let title =
        normalize_commit_title("  修复\nworktree\t合并  标题  ").expect("title should exist");
    assert_eq!(title, "修复 worktree 合并 标题");

    let long = "a".repeat(100);
    let normalized = normalize_commit_title(&long).expect("long title should exist");
    assert_eq!(normalized.chars().count(), 72);
}

#[test]
fn task_session_id_uses_task_board_prefix() {
    assert_eq!(task_session_id("123"), "task-board-123");
}

#[test]
fn resolve_task_execution_acp_agent_prefers_selected_agent() {
    let mut task = Task::default();
    task.acp_agent = Some("claude".to_string());

    let acp_agent = resolve_task_execution_acp_agent(&task);

    assert_eq!(acp_agent.as_deref(), Some("claude"));
}

#[test]
fn resolve_task_execution_acp_agent_keeps_unknown_agent() {
    let mut task = Task::default();
    task.acp_agent = Some("custom-acp".to_string());

    let acp_agent = resolve_task_execution_acp_agent(&task);

    assert_eq!(acp_agent.as_deref(), Some("custom-acp"));
}

#[test]
fn resolve_task_execution_acp_agent_respects_explicit_internal_selection() {
    let mut task = Task::default();
    task.acp_agent = Some("internal".to_string());

    let acp_agent = resolve_task_execution_acp_agent(&task);

    assert_eq!(acp_agent, None);
}

#[test]
fn resolve_task_execution_acp_agent_does_not_fallback_when_missing() {
    let task = Task::default();

    let acp_agent = resolve_task_execution_acp_agent(&task);

    assert_eq!(acp_agent, None);
}

#[test]
fn start_execution_preserves_target_branch_and_worktree_path() {
    let mut task = Task::default();
    task.merge_source_branch = Some("feature/old".to_string());
    task.merge_target_branch = Some("main".to_string());
    task.selected_worktree_path = Some("/tmp/worktree-path".to_string());

    task.start_execution("开始执行任务".to_string());

    assert_eq!(task.merge_source_branch, None);
    assert_eq!(task.merge_target_branch.as_deref(), Some("main"));
    assert_eq!(task.selected_worktree_path.as_deref(), Some("/tmp/worktree-path"));
}

#[test]
fn resolve_task_execution_workspace_reads_branch_from_selected_worktree() {
    let _env_lock = task_executor_env_lock();
    let temp = tempfile::TempDir::new().expect("temp repo should be created");
    let repo_dir = temp.path().join("repo");
    let worktree_dir = temp.path().join("repo-feature");

    std::fs::create_dir_all(&repo_dir).expect("repo dir should be created");
    run_git_test(&repo_dir, &["init", "--initial-branch=main"]);
    run_git_test(&repo_dir, &["config", "user.name", "VibeWindow Test"]);
    run_git_test(&repo_dir, &["config", "user.email", "test@vibewindow.local"]);

    std::fs::write(repo_dir.join("README.md"), "seed\n").expect("seed file should be written");
    run_git_test(&repo_dir, &["add", "."]);
    run_git_test(&repo_dir, &["commit", "-m", "init"]);
    run_git_test(&repo_dir, &["branch", "feature/task-worktree"]);
    run_git_test(
        &repo_dir,
        &["worktree", "add", worktree_dir.to_string_lossy().as_ref(), "feature/task-worktree"],
    );

    let mut task = Task::default();
    task.selected_worktree_path = Some(worktree_dir.to_string_lossy().to_string());
    task.merge_target_branch = Some("main".to_string());

    let (workspace, _guard) =
        resolve_task_execution_workspace(&task, repo_dir.to_string_lossy().as_ref(), None)
            .expect("workspace should resolve");

    assert_eq!(
        workspace.selected_worktree_path.as_deref(),
        Some(normalize_path(worktree_dir.to_string_lossy().as_ref()).as_str())
    );
    assert_eq!(workspace.execution_path, normalize_path(worktree_dir.to_string_lossy().as_ref()));
    assert_eq!(workspace.selected_worktree_branch.as_deref(), Some("feature/task-worktree"));
    assert_eq!(workspace.merge_target_branch.as_deref(), Some("main"));
}

#[test]
fn assign_task_execution_worktree_uses_repo_head_when_target_branch_missing() {
    let _env_lock = task_executor_env_lock();
    let temp = tempfile::TempDir::new().expect("temp repo should be created");
    let home = tempfile::TempDir::new().expect("temp home should be created");
    let repo_dir = temp.path().join("repo");

    std::fs::create_dir_all(&repo_dir).expect("repo dir should be created");
    run_git_test(&repo_dir, &["init", "--initial-branch=main"]);
    run_git_test(&repo_dir, &["config", "user.name", "VibeWindow Test"]);
    run_git_test(&repo_dir, &["config", "user.email", "test@vibewindow.local"]);

    std::fs::write(repo_dir.join("README.md"), "seed\n").expect("seed file should be written");
    run_git_test(&repo_dir, &["add", "."]);
    run_git_test(&repo_dir, &["commit", "-m", "init"]);

    let _home_guard = EnvGuard::set_os("HOME", home.path().as_os_str());

    let task = Task::default();
    let assigned = assign_task_execution_worktree(repo_dir.to_string_lossy().as_ref(), &task, None)
        .expect("worktree should be assigned");

    let assigned = assigned.expect("managed worktree path should exist");
    let assigned_path = std::path::Path::new(&assigned);
    assert!(assigned_path.is_dir());
    assert!(
        assigned_path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("slot-"))
    );
    assert_eq!(
        assigned_path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|value| value.to_str()),
        Some("task-worktrees")
    );
    assert_eq!(
        assigned_path
            .parent()
            .and_then(|parent| parent.parent())
            .and_then(|parent| parent.file_name())
            .and_then(|value| value.to_str()),
        Some(".vibewindow")
    );
}

#[test]
fn resolve_task_execution_workspace_uses_repo_head_when_target_branch_missing() {
    let _env_lock = task_executor_env_lock();
    let temp = tempfile::TempDir::new().expect("temp repo should be created");
    let home = tempfile::TempDir::new().expect("temp home should be created");
    let repo_dir = temp.path().join("repo");

    std::fs::create_dir_all(&repo_dir).expect("repo dir should be created");
    run_git_test(&repo_dir, &["init", "--initial-branch=main"]);
    run_git_test(&repo_dir, &["config", "user.name", "VibeWindow Test"]);
    run_git_test(&repo_dir, &["config", "user.email", "test@vibewindow.local"]);

    std::fs::write(repo_dir.join("README.md"), "seed\n").expect("seed file should be written");
    run_git_test(&repo_dir, &["add", "."]);
    run_git_test(&repo_dir, &["commit", "-m", "init"]);

    let _home_guard = EnvGuard::set_os("HOME", home.path().as_os_str());

    let task = Task::default();
    let (workspace, _guard) =
        resolve_task_execution_workspace(&task, repo_dir.to_string_lossy().as_ref(), None)
            .expect("workspace should resolve");

    assert!(workspace.slot_id.is_some());
    assert_ne!(workspace.execution_path, normalize_path(repo_dir.to_string_lossy().as_ref()));
    assert!(std::path::Path::new(&workspace.execution_path).is_dir());
    assert!(
        workspace
            .selected_worktree_branch
            .as_deref()
            .is_some_and(|branch| branch.starts_with("vw/task/"))
    );
    assert_eq!(workspace.merge_target_branch.as_deref(), Some("main"));
    assert_eq!(
        workspace.selected_worktree_path.as_deref(),
        Some(workspace.execution_path.as_str())
    );
}
