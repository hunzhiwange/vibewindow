//! 定时任务调度器测试模块
//!
//! 本模块提供了调度器核心功能的单元测试和集成测试，包括：
//! - Shell 命令执行与安全策略验证
//! - Agent 任务执行与权限检查
//! - 任务重试机制
//! - 任务结果持久化
//! - 投递配置处理
//! - 组件健康状态标记
//!
//! 所有测试都遵循 VibeWindow 的安全默认策略，确保定时任务在受控环境下运行。

use crate::app::agent::config::Config;
use crate::app::agent::cron;
use crate::app::agent::cron::scheduler::{
    cron_agent_prompt, cron_full_access_config, deliver_announcement, deliver_if_configured,
    execute_job_with_retry, notify_schedule_changed, persist_job_result, process_due_jobs,
    run_agent_job, run_job_command, run_job_command_with_timeout, wait_for_schedule_scan,
};
use crate::app::agent::cron::{CronJob, DeliveryConfig, JobType, Schedule, SessionTarget};
use crate::app::agent::security::{SecurityPolicy, ShellRedirectPolicy};
use chrono::{Duration as ChronoDuration, Utc};
use std::sync::Arc;
use std::sync::OnceLock;
use tempfile::TempDir;
use tokio::time::Duration;

/// 获取测试环境全局锁
///
/// 用于确保涉及环境变量修改的测试不会并发执行，避免测试间的状态干扰。
/// 这是一个静态的 Tokio 互斥锁，在首次调用时初始化。
///
/// # 返回值
///
/// 返回互斥锁的守卫，在守卫生命周期内持有锁
async fn env_lock() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(())).lock().await
}

/// 环境变量守卫
///
/// 用于在测试中临时操作环境变量，并在测试结束后自动恢复原始状态。
/// 这确保了测试的隔离性和可重复性。
///
/// # 字段说明
///
/// - `key`: 环境变量名称
/// - `original`: 原始环境变量值（如果存在）
struct EnvGuard {
    key: &'static str,
    original: Option<String>,
}

impl EnvGuard {
    /// 创建一个环境变量守卫并清除指定变量
    ///
    /// 该方法会保存原始值并在守卫被丢弃时恢复。
    ///
    /// # 参数
    ///
    /// - `key`: 要清除的环境变量名称
    ///
    /// # 返回值
    ///
    /// 返回一个 `EnvGuard` 实例，它会记住原始值以便恢复
    ///
    /// # 安全性
    ///
    /// 使用 `unsafe` 块是因为在多线程环境中修改环境变量不是线程安全的，
    /// 但在测试场景中配合 `env_lock()` 使用是可接受的。
    fn unset(key: &'static str) -> Self {
        let original = std::env::var(key).ok();
        unsafe {
            std::env::remove_var(key);
        }
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    /// 析构时恢复环境变量原始状态
    ///
    /// 如果原始值存在则恢复，否则清除该环境变量。
    fn drop(&mut self) {
        match self.original.as_ref() {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

/// 创建测试用的配置对象
///
/// 为每个测试生成独立的临时工作空间，确保测试之间完全隔离。
///
/// # 参数
///
/// - `tmp`: 临时目录引用
///
/// # 返回值
///
/// 返回配置好的 `Config` 实例
async fn test_config(tmp: &TempDir) -> Config {
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("vibewindow.json"),
        ..Config::default()
    };
    tokio::fs::create_dir_all(&config.workspace_dir).await.unwrap();
    config
}

/// 创建测试用的定时任务对象
///
/// 生成一个标准的测试任务，每分钟执行一次指定的命令。
///
/// # 参数
///
/// - `command`: 要执行的 shell 命令
///
/// # 返回值
///
/// 返回配置好的 `CronJob` 实例
fn test_job(command: &str) -> CronJob {
    CronJob {
        id: "test-job".into(),
        expression: "* * * * *".into(),
        schedule: Schedule::Cron { expr: "* * * * *".into(), tz: None },
        command: command.into(),
        prompt: None,
        name: None,
        job_type: JobType::Shell,
        session_target: SessionTarget::Isolated,
        model: None,
        agent: None,
        acp_agent: None,
        project_path: None,
        wake: false,
        fallbacks: Vec::new(),
        full_access: false,
        task_pool: false,
        enabled: true,
        delivery: DeliveryConfig::default(),
        delete_after_run: false,
        created_at: Utc::now(),
        next_run: Utc::now(),
        last_run: None,
        last_status: None,
        last_output: None,
    }
}

/// 生成唯一的组件标识符
///
/// 用于在测试中创建具有唯一名称的组件，避免并发测试间的标识符冲突。
///
/// # 参数
///
/// - `prefix`: 组件名称前缀
///
/// # 返回值
///
/// 返回格式为 `{prefix}-{uuid}` 的唯一标识符
fn unique_component(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

#[tokio::test]
async fn schedule_change_notification_interrupts_poll_wait() {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    interval.tick().await;

    notify_schedule_changed();

    tokio::time::timeout(Duration::from_millis(100), wait_for_schedule_scan(&mut interval))
        .await
        .expect("schedule change notification should wake scheduler scan wait");
}

/// 测试 Shell 命令成功执行
///
/// 验证基本的命令执行功能：
/// - 命令能够正常执行并返回成功状态
/// - 输出中包含预期的内容
/// - 状态码为 0（成功退出）
#[tokio::test]
async fn run_job_command_success() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let job = test_job("echo scheduler-ok");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(success);
    assert!(output.contains("scheduler-ok"));
    assert!(output.contains("status=exit status: 0"));
}

/// 测试 Shell 命令执行失败
///
/// 验证命令执行失败时的处理：
/// - 命令失败时返回 false
/// - 输出中包含错误信息
/// - 状态码表示失败（非零）
#[tokio::test]
async fn run_job_command_failure() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let job = test_job("ls definitely_missing_file_for_scheduler_test");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("definitely_missing_file_for_scheduler_test"));
    assert!(output.contains("status=exit status:"));
}

/// 测试命令执行超时
///
/// 验证命令超时机制：
/// - 长时间运行的命令会被强制终止
/// - 返回超时错误标识
/// - 输出中包含超时消息
#[tokio::test]
async fn run_job_command_times_out() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["sleep".into()];
    let job = test_job("sleep 1");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) =
        run_job_command_with_timeout(&config, &security, &job, Duration::from_millis(50)).await;
    assert!(!success);
    assert!(output.contains("job timed out after"));
}

/// 测试阻止未授权命令执行
///
/// 验证命令白名单机制：
/// - 只有在 allowed_commands 列表中的命令才能执行
/// - 未授权命令会被安全策略阻止
/// - 输出中包含明确的阻止原因
#[tokio::test]
async fn run_job_command_blocks_disallowed_command() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["echo".into()];
    let job = test_job("curl https://evil.example");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("command not allowed"));
}

/// 测试阻止禁止路径参数
///
/// 验证路径安全检查：
/// - 系统关键路径（如 /etc/passwd）被禁止访问
/// - 安全策略会检测命令参数中的路径
/// - 返回明确的禁止路径错误
#[tokio::test]
async fn run_job_command_blocks_forbidden_path_argument() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["cat".into()];
    let job = test_job("cat /etc/passwd");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("forbidden path argument"));
    assert!(output.contains("/etc/passwd"));
}

/// 测试阻止选项赋值形式的禁止路径参数
///
/// 验证路径安全检查能够识别 --option=value 形式的路径参数：
/// - 检测长选项赋值中的路径（如 --file=/etc/passwd）
/// - 阻止通过选项绕过路径安全检查的尝试
#[tokio::test]
async fn run_job_command_blocks_forbidden_option_assignment_path_argument() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["grep".into()];
    let job = test_job("grep --file=/etc/passwd root ./src");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("forbidden path argument"));
    assert!(output.contains("/etc/passwd"));
}

/// 测试阻止短选项紧连形式的禁止路径参数
///
/// 验证路径安全检查能够识别 -ovalue 形式的路径参数：
/// - 检测短选项紧连值中的路径（如 -f/etc/passwd）
/// - 阻止通过短选项格式绕过路径检查的尝试
#[tokio::test]
async fn run_job_command_blocks_forbidden_short_option_attached_path_argument() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["grep".into()];
    let job = test_job("grep -f/etc/passwd root ./src");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("forbidden path argument"));
    assert!(output.contains("/etc/passwd"));
}

/// 测试阻止波浪号用户路径参数
///
/// 验证路径安全检查能够识别用户主目录路径：
/// - 检测 ~user 形式的路径（如 ~root/.ssh/id_rsa）
/// - 防止通过波浪号语法访问其他用户的文件
#[tokio::test]
async fn run_job_command_blocks_tilde_user_path_argument() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["cat".into()];
    let job = test_job("cat ~root/.ssh/id_rsa");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("forbidden path argument"));
    assert!(output.contains("~root/.ssh/id_rsa"));
}

/// 测试阻止输入重定向绕过
///
/// 验证 shell 重定向安全策略：
/// - 输入重定向（<）被视为 shell 特性而被阻止
/// - 防止通过重定向语法绕过路径安全检查
#[tokio::test]
async fn run_job_command_blocks_input_redirection_path_bypass() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["cat".into()];
    let job = test_job("cat </etc/passwd");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("command not allowed"));
}

/// 测试 Strip 重定向策略标准化常见 stderr 重定向
///
/// 验证 ShellRedirectPolicy::Strip 策略：
/// - 该策略会移除命令中的重定向语法
/// - 移除后命令仍能正常执行
/// - 常见的 2>&1 重定向会被正确处理
#[tokio::test]
async fn run_job_command_strip_policy_normalizes_common_stderr_redirects() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.allowed_commands = vec!["echo".into()];
    config.autonomy.shell_redirect_policy = ShellRedirectPolicy::Strip;
    let job = test_job("echo scheduler-strip 2>&1");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(success);
    assert!(output.contains("scheduler-strip"));
}

/// 测试只读模式阻止命令执行
///
/// 验证 AutonomyLevel::ReadOnly 安全级别：
/// - 在只读模式下，所有命令执行都会被阻止
/// - 返回明确的只读模式错误信息
#[tokio::test]
async fn run_job_command_blocks_readonly_mode() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.level = crate::app::agent::security::AutonomyLevel::ReadOnly;
    let job = test_job("echo should-not-run");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("read-only"));
}

/// 测试速率限制阻止命令执行
///
/// 验证每小时最大操作数限制：
/// - 当 max_actions_per_hour 为 0 时，命令执行被阻止
/// - 返回速率限制超出的错误信息
#[tokio::test]
async fn run_job_command_blocks_rate_limited() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.max_actions_per_hour = 0;
    let job = test_job("echo should-not-run");
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_job_command(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("rate limit exceeded"));
}

/// 测试任务重试机制在首次失败后恢复
///
/// 验证 scheduler_retries 配置：
/// - 任务在失败后会被重试
/// - 重试时使用退避延迟（provider_backoff_ms）
/// - 成功的重试会被正确识别并返回成功
#[tokio::test]
async fn execute_job_with_retry_recovers_after_first_failure() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.reliability.scheduler_retries = 1;
    config.reliability.provider_backoff_ms = 1;
    config.autonomy.allowed_commands = vec!["sh".into()];
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    // 创建一个脚本，首次执行失败，第二次执行成功
    tokio::fs::write(
        config.workspace_dir.join("retry-once.sh"),
        "#!/bin/sh\nif [ -f retry-ok.flag ]; then\n  echo recovered\n  exit 0\nfi\ntouch retry-ok.flag\nexit 1\n",
    )
    .await
    .unwrap();
    let job = test_job("sh ./retry-once.sh");

    let (success, output) = execute_job_with_retry(&config, &security, &job).await;
    assert!(success);
    assert!(output.contains("recovered"));
}

/// 测试任务重试机制耗尽所有尝试
///
/// 验证当所有重试都失败时的处理：
/// - 重试次数耗尽后返回失败
/// - 输出中包含失败信息
#[tokio::test]
async fn execute_job_with_retry_exhausts_attempts() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.reliability.scheduler_retries = 1;
    config.reliability.provider_backoff_ms = 1;
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let job = test_job("ls always_missing_for_retry_test");

    let (success, output) = execute_job_with_retry(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("always_missing_for_retry_test"));
}

/// 测试 Agent 任务在没有 API 密钥时返回错误
///
/// 验证 Agent 类型任务对 API 密钥的依赖：
/// - 没有配置任何 API 密钥时，Agent 任务会失败
/// - 使用环境变量守卫确保测试隔离
#[tokio::test]
async fn run_agent_job_returns_error_without_provider_key() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let _env = env_lock().await;
    let _generic = EnvGuard::unset("VIBEWINDOW_API_KEY");
    let _fallback = EnvGuard::unset("API_KEY");
    let _openrouter = EnvGuard::unset("OPENROUTER_API_KEY");
    let mut job = test_job("");
    job.job_type = JobType::Agent;
    job.prompt = Some("Say hello".into());
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_agent_job(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("agent job failed:"));
}

/// 测试 Agent 任务在只读模式下被阻止
///
/// 验证只读模式同样适用于 Agent 任务：
/// - AutonomyLevel::ReadOnly 会阻止 Agent 任务执行
/// - 返回明确的只读模式错误
#[tokio::test]
async fn run_agent_job_blocks_readonly_mode() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.level = crate::app::agent::security::AutonomyLevel::ReadOnly;
    let mut job = test_job("");
    job.job_type = JobType::Agent;
    job.prompt = Some("Say hello".into());
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_agent_job(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("read-only"));
}

/// 测试 Agent 任务在速率限制下被阻止
///
/// 验证速率限制同样适用于 Agent 任务：
/// - max_actions_per_hour 为 0 时，Agent 任务被阻止
/// - 返回速率限制错误
#[tokio::test]
async fn run_agent_job_blocks_rate_limited() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.max_actions_per_hour = 0;
    let mut job = test_job("");
    job.job_type = JobType::Agent;
    job.prompt = Some("Say hello".into());
    let security = SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir);

    let (success, output) = run_agent_job(&config, &security, &job).await;
    assert!(!success);
    assert!(output.contains("blocked by security policy"));
    assert!(output.contains("rate limit exceeded"));
}

#[tokio::test]
async fn full_access_agent_job_uses_trusted_runtime_config() {
    let tmp = TempDir::new().unwrap();
    let mut config = test_config(&tmp).await;
    config.autonomy.always_ask = vec!["shell".into()];
    config.autonomy.non_cli_excluded_tools = vec!["shell".into()];
    let run_config = cron_full_access_config(&config);

    assert_eq!(run_config.autonomy.level, crate::app::agent::security::AutonomyLevel::Full);
    assert!(!run_config.autonomy.workspace_only);
    assert_eq!(run_config.autonomy.allowed_commands, vec!["*"]);
    assert_eq!(run_config.autonomy.allowed_roots, vec!["/"]);
    assert!(run_config.autonomy.always_ask.is_empty());
    assert!(run_config.autonomy.non_cli_excluded_tools.is_empty());
    assert!(run_config.autonomy.allow_unsafe_shell_patterns);
}

#[test]
fn full_access_agent_prompt_includes_automation_instruction() {
    let mut job = test_job("");
    job.job_type = JobType::Agent;
    job.full_access = true;

    let prompt = cron_agent_prompt(&job, "daily", "写入报告");
    assert!(prompt.contains("我是自动化脚本，无法回答你的问题"));
    assert!(prompt.contains("写入报告"));
}

/// 测试 process_due_jobs 在空闲时仍标记组件健康
///
/// 验证调度器组件健康状态管理：
/// - 即使没有待处理任务，也会将组件标记为健康
/// - 清除之前存在的错误状态
/// - 记录健康检查时间戳
#[tokio::test]
async fn process_due_jobs_marks_component_ok_even_when_idle() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let security = Arc::new(SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir));
    let component = unique_component("scheduler-idle");

    // 先标记组件为错误状态
    crate::app::agent::health::mark_component_error(&component, "pre-existing error");
    // 处理空任务列表
    process_due_jobs(&config, &security, Vec::new(), &component).await;

    let snapshot = crate::app::agent::health::snapshot_json();
    let entry = &snapshot["components"][component.as_str()];
    assert_eq!(entry["status"], "ok");
    assert!(entry["last_ok"].as_str().is_some());
    assert!(entry["last_error"].is_null());
}

/// 测试 process_due_jobs 任务失败不影响组件健康状态
///
/// 验证组件健康状态与单个任务执行结果的独立性：
/// - 单个任务执行失败不会将整个调度器组件标记为不健康
/// - 这允许调度器继续处理后续任务
#[tokio::test]
async fn process_due_jobs_failure_does_not_mark_component_unhealthy() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let job = test_job("ls definitely_missing_file_for_scheduler_component_health_test");
    let security = Arc::new(SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir));
    let component = unique_component("scheduler-fail");

    // 先标记组件为健康状态
    crate::app::agent::health::mark_component_ok(&component);
    // 处理会失败的任务
    process_due_jobs(&config, &security, vec![job], &component).await;

    let snapshot = crate::app::agent::health::snapshot_json();
    let entry = &snapshot["components"][component.as_str()];
    assert_eq!(entry["status"], "ok");
}

/// 测试任务结果持久化并重新调度 Shell 任务
///
/// 验证 persist_job_result 函数的核心功能：
/// - 记录任务执行运行历史
/// - 更新任务的最后状态和输出
/// - 为周期性任务计算下次运行时间
#[tokio::test]
async fn persist_job_result_records_run_and_reschedules_shell_job() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let job = cron::add_job(&config, "*/5 * * * *", "echo ok").unwrap();
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    let success = persist_job_result(&config, &job, true, "ok", started, finished).await;
    assert!(success);

    // 验证运行记录被保存
    let runs = cron::list_runs(&config, &job.id, 10).unwrap();
    assert_eq!(runs.len(), 1);
    // 验证任务状态被更新
    let updated = cron::get_job(&config, &job.id).unwrap();
    assert_eq!(updated.last_status.as_deref(), Some("ok"));
}

/// 测试一次性任务成功后被删除
///
/// 验证 delete_after_run 标志的功能：
/// - Agent 类型的一次性任务在成功执行后会被自动删除
/// - 使用 Schedule::At 指定一次性执行时间
#[tokio::test]
async fn persist_job_result_success_deletes_one_shot() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let at = Utc::now() + ChronoDuration::minutes(10);
    // 创建一个一次性 Agent 任务，delete_after_run 为 true
    let job = cron::add_agent_job(
        &config,
        Some("one-shot".into()),
        Schedule::At { at },
        "Hello",
        SessionTarget::Isolated,
        None,
        None,
        true,
    )
    .unwrap();
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    let success = persist_job_result(&config, &job, true, "ok", started, finished).await;
    assert!(success);
    // 验证任务已被删除
    let lookup = cron::get_job(&config, &job.id);
    assert!(lookup.is_err());
}

/// 测试一次性任务失败后被禁用
///
/// 验证一次性任务失败时的处理：
/// - 任务失败后不会被删除，而是被禁用
/// - 最后状态被设置为 "error"
/// - 允许管理员检查失败原因后决定是否重新启用
#[tokio::test]
async fn persist_job_result_failure_disables_one_shot() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let at = Utc::now() + ChronoDuration::minutes(10);
    let job = cron::add_agent_job(
        &config,
        Some("one-shot".into()),
        Schedule::At { at },
        "Hello",
        SessionTarget::Isolated,
        None,
        None,
        true,
    )
    .unwrap();
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    let success = persist_job_result(&config, &job, false, "boom", started, finished).await;
    assert!(!success);
    // 验证任务被禁用但未删除
    let updated = cron::get_job(&config, &job.id).unwrap();
    assert!(!updated.enabled);
    assert_eq!(updated.last_status.as_deref(), Some("error"));
}

/// 测试一次性 Shell 任务成功后被删除
///
/// 验证 add_once_at 创建的任务在成功后被删除：
/// - Shell 类型的一次性任务同样支持自动删除
/// - delete_after_run 标志默认为 true
#[tokio::test]
async fn persist_job_result_success_deletes_one_shot_shell_job() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let at = Utc::now() + ChronoDuration::minutes(10);
    let job = cron::add_once_at(&config, at, "echo one-shot-shell").unwrap();
    assert!(job.delete_after_run);
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    let success = persist_job_result(&config, &job, true, "ok", started, finished).await;
    assert!(success);
    // 验证任务已被删除
    let lookup = cron::get_job(&config, &job.id);
    assert!(lookup.is_err());
}

/// 测试一次性 Shell 任务失败后被禁用
///
/// 验证 Shell 类型一次性任务失败时的处理：
/// - 与 Agent 类型一致，失败后禁用而非删除
/// - 保留任务记录以便排障
#[tokio::test]
async fn persist_job_result_failure_disables_one_shot_shell_job() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let at = Utc::now() + ChronoDuration::minutes(10);
    let job = cron::add_once_at(&config, at, "echo one-shot-shell").unwrap();
    assert!(job.delete_after_run);
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    let success = persist_job_result(&config, &job, false, "boom", started, finished).await;
    assert!(!success);
    let updated = cron::get_job(&config, &job.id).unwrap();
    assert!(!updated.enabled);
    assert_eq!(updated.last_status.as_deref(), Some("error"));
}

/// 测试投递失败在非尽力模式下标记任务错误
///
/// 验证 DeliveryConfig.best_effort=false 的行为：
/// - 任务本身成功但投递失败时，整体标记为失败
/// - 任务保持启用状态以便重试
/// - 运行记录中状态为 "error"
#[tokio::test]
async fn persist_job_result_delivery_failure_non_best_effort_marks_error() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    // 创建带有投递配置的任务，best_effort 为 false
    let job = cron::add_agent_job(
        &config,
        Some("announce-job".into()),
        Schedule::Cron { expr: "*/5 * * * *".into(), tz: None },
        "deliver this",
        SessionTarget::Isolated,
        None,
        Some(DeliveryConfig {
            mode: "announce".into(),
            channel: Some("telegram".into()),
            to: Some("123456".into()),
            best_effort: false,
        }),
        false,
    )
    .unwrap();
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    // 由于没有配置 Telegram，投递会失败，且 best_effort=false
    let success = persist_job_result(&config, &job, true, "ok", started, finished).await;
    assert!(!success);

    let updated = cron::get_job(&config, &job.id).unwrap();
    assert!(updated.enabled);
    assert_eq!(updated.last_status.as_deref(), Some("error"));

    let runs = cron::list_runs(&config, &job.id, 10).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "error");
}

/// 测试投递失败在尽力模式下保持成功
///
/// 验证 DeliveryConfig.best_effort=true 的行为：
/// - 任务本身成功时，投递失败不影响整体结果
/// - 任务状态保持为成功
/// - 适用于非关键通知场景
#[tokio::test]
async fn persist_job_result_delivery_failure_best_effort_keeps_success() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    // 创建带有投递配置的任务，best_effort 为 true
    let job = cron::add_agent_job(
        &config,
        Some("announce-job-best-effort".into()),
        Schedule::Cron { expr: "*/5 * * * *".into(), tz: None },
        "deliver this",
        SessionTarget::Isolated,
        None,
        Some(DeliveryConfig {
            mode: "announce".into(),
            channel: Some("telegram".into()),
            to: Some("123456".into()),
            best_effort: true,
        }),
        false,
    )
    .unwrap();
    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);

    let success = persist_job_result(&config, &job, true, "ok", started, finished).await;
    assert!(success);

    let updated = cron::get_job(&config, &job.id).unwrap();
    assert!(updated.enabled);
    assert_eq!(updated.last_status.as_deref(), Some("ok"));

    let runs = cron::list_runs(&config, &job.id, 10).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "ok");
}

/// 测试 At 调度类型且 delete_after_run=false 时不会被删除
///
/// 验证 Schedule::At 与 delete_after_run 的独立性：
/// - At 调度类型本身不会自动设置 delete_after_run
/// - 只有显式设置 delete_after_run=true 才会在成功后删除
/// - 允许创建定期执行但不自动删除的 At 类型任务
#[tokio::test]
async fn persist_job_result_at_schedule_without_delete_after_run_is_not_deleted() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let at = Utc::now() + ChronoDuration::minutes(10);
    // 创建 At 类型任务，但 delete_after_run 为 false
    let job = cron::add_agent_job(
        &config,
        Some("at-no-autodelete".into()),
        Schedule::At { at },
        "Hello",
        SessionTarget::Isolated,
        None,
        None,
        false,
    )
    .unwrap();
    assert!(!job.delete_after_run);

    let started = Utc::now();
    let finished = started + ChronoDuration::milliseconds(10);
    let success = persist_job_result(&config, &job, true, "ok", started, finished).await;
    assert!(success);

    // 任务仍然存在且保持启用
    let updated = cron::get_job(&config, &job.id).unwrap();
    assert!(updated.enabled);
    assert_eq!(updated.last_status.as_deref(), Some("ok"));
}

/// 测试 deliver_if_configured 处理无配置和无效通道
///
/// 验证投递函数的健壮性：
/// - 无投递配置时返回 Ok
/// - 无效通道时返回错误
/// - 错误消息清晰指明原因
#[tokio::test]
async fn deliver_if_configured_handles_none_and_invalid_channel() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp).await;
    let mut job = test_job("echo ok");

    // 无投递配置时应该成功
    assert!(deliver_if_configured(&config, &job, "x").await.is_ok());

    // 配置无效通道时应该失败
    job.delivery = DeliveryConfig {
        mode: "announce".into(),
        channel: Some("invalid".into()),
        to: Some("target".into()),
        best_effort: true,
    };
    let err = deliver_if_configured(&config, &job, "x").await.unwrap_err();
    assert!(err.to_string().contains("unsupported delivery channel"));
}
