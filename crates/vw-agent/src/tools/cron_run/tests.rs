//! CronRun 工具强制执行测试。
//!
//! 验证手动触发计划任务时的历史记录、缺失任务错误、只读阻止、监督审批和
//! 速率限制，确保计划任务不会绕过通用安全策略。

use super::super::*;
use crate::app::agent::config::Config;
use crate::app::agent::cron::{add_job, list_runs, record_run};
use crate::app::agent::security::AutonomyLevel;
use serde_json::json;
use tempfile::TempDir;

async fn test_config(tmp: &TempDir) -> Arc<Config> {
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("vibewindow.json"),
        ..Config::default()
    };
    tokio::fs::create_dir_all(&config.workspace_dir).await.unwrap();
    Arc::new(config)
}

fn test_security(cfg: &Config) -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy::from_config(&cfg.autonomy, &cfg.workspace_dir))
}

#[tokio::test]
async fn force_runs_job_and_records_history() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(&tmp).await;
    let job = add_job(&cfg, "*/5 * * * *", "echo run-now").unwrap();
    let tool = CronRunTool::new(cfg.clone(), test_security(&cfg));

    let result = tool.execute(json!({ "job_id": job.id })).await.unwrap();
    assert!(result.success, "{:?}", result.error);

    let runs = list_runs(&cfg, &job.id, 10).unwrap();
    assert_eq!(runs.len(), 1);
}

#[tokio::test]
async fn errors_for_missing_job() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(&tmp).await;
    let tool = CronRunTool::new(cfg.clone(), test_security(&cfg));

    let result = tool.execute(json!({ "job_id": "missing-job-id" })).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap_or_default().contains("not found"));
}

#[tokio::test]
async fn blocks_run_in_read_only_mode() {
    let tmp = TempDir::new().unwrap();
    let mut config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("vibewindow.json"),
        ..Config::default()
    };
    config.autonomy.level = AutonomyLevel::ReadOnly;
    std::fs::create_dir_all(&config.workspace_dir).unwrap();
    let cfg = Arc::new(config);
    let job = add_job(&cfg, "*/5 * * * *", "echo run-now").unwrap();
    let tool = CronRunTool::new(cfg.clone(), test_security(&cfg));

    let result = tool.execute(json!({ "job_id": job.id })).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap_or_default().contains("read-only"));
}

#[tokio::test]
async fn shell_run_requires_approval_for_medium_risk() {
    let tmp = TempDir::new().unwrap();
    let mut config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("vibewindow.json"),
        ..Config::default()
    };
    config.autonomy.level = AutonomyLevel::Supervised;
    config.autonomy.allowed_commands = vec!["touch".into()];
    std::fs::create_dir_all(&config.workspace_dir).unwrap();
    let cfg = Arc::new(config);
    let job = add_job(&cfg, "*/5 * * * *", "touch cron-run-approval").unwrap();
    let tool = CronRunTool::new(cfg.clone(), test_security(&cfg));

    let denied = tool.execute(json!({ "job_id": job.id })).await.unwrap();
    assert!(!denied.success);
    assert!(denied.error.unwrap_or_default().contains("explicit approval"));

    // 显式 approved 标志模拟审批完成后的二次调用，验证安全门槛仍由工具执行。
    let approved = tool.execute(json!({ "job_id": job.id, "approved": true })).await.unwrap();
    assert!(approved.success, "{:?}", approved.error);
}

#[tokio::test]
async fn blocks_run_when_rate_limited() {
    let tmp = TempDir::new().unwrap();
    let mut config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("vibewindow.json"),
        ..Config::default()
    };
    config.autonomy.level = AutonomyLevel::Full;
    config.autonomy.max_actions_per_hour = 0;
    std::fs::create_dir_all(&config.workspace_dir).unwrap();
    let cfg = Arc::new(config);
    let job = add_job(&cfg, "*/5 * * * *", "echo run-now").unwrap();
    let tool = CronRunTool::new(cfg.clone(), test_security(&cfg));

    let result = tool.execute(json!({ "job_id": job.id })).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap_or_default().contains("Rate limit exceeded"));
    assert!(list_runs(&cfg, &job.id, 10).unwrap().is_empty());
}
