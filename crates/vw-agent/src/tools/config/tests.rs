//! Config 工具行为测试。
//!
//! 覆盖完整快照读取、旧版分区桥接、单项配置读写和结构化 call 结果，确保
//! 兼容路径与新的 setting/value 接口同时工作。

use super::ConfigTool;
use crate::app::agent::config::Config;
use crate::app::agent::config::schema::ConfigExt;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::Tool;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

async fn test_config() -> (TempDir, Arc<Config>) {
    let tmp = TempDir::new().expect("tempdir should create");
    let mut config = Config::default();
    config.workspace_dir = tmp.path().join("workspace");
    config.config_path = tmp.path().join("vibewindow.json");
    // 每个测试使用独立配置文件，避免设置写入影响其他用例。
    std::fs::create_dir_all(&config.workspace_dir).expect("workspace should create");
    config.save().await.expect("config file should write");
    (tmp, Arc::new(config))
}

#[tokio::test]
async fn config_returns_snapshot_for_all_section() {
    let (_tmp, config) = test_config().await;
    let tool = ConfigTool::new(config, Arc::new(SecurityPolicy::default()));
    let result = tool.execute(json!({})).await.expect("config should succeed");
    let output: serde_json::Value =
        serde_json::from_str(&result.output).expect("snapshot output should be json");

    assert!(result.success);
    assert_eq!(output["operation"].as_str(), Some("get"));
    assert!(output["value"]["workspace_dir"].is_string());
    assert!(output["value"]["config_path"].is_string());
}

#[tokio::test]
async fn config_bridges_proxy_section() {
    let (_tmp, config) = test_config().await;
    let tool = ConfigTool::new(config, Arc::new(SecurityPolicy::default()));
    let result = tool
        .execute(json!({
            "section": "proxy",
            "payload": {
                "action": "get"
            }
        }))
        .await
        .expect("proxy bridge should succeed");

    assert!(result.success);
    assert!(result.output.contains("scope"));
}

#[tokio::test]
async fn config_reads_supported_setting() {
    let (_tmp, config) = test_config().await;
    let tool = ConfigTool::new(config, Arc::new(SecurityPolicy::default()));

    let result = tool
        .execute(json!({
            "setting": "browser.enabled"
        }))
        .await
        .expect("config get should succeed");

    let output: serde_json::Value =
        serde_json::from_str(&result.output).expect("config output should be json");
    assert!(result.success);
    assert_eq!(output["operation"].as_str(), Some("get"));
    assert_eq!(output["setting"].as_str(), Some("browser.enabled"));
    assert_eq!(output["value"], json!(false));
}

#[tokio::test]
async fn config_sets_supported_setting_and_persists() {
    let (_tmp, config) = test_config().await;
    let tool = ConfigTool::new(config.clone(), Arc::new(SecurityPolicy::default()));

    let result = tool
        .execute(json!({
            "setting": "browser.enabled",
            "value": true
        }))
        .await
        .expect("config set should succeed");

    let output: serde_json::Value =
        serde_json::from_str(&result.output).expect("config output should be json");
    let saved: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&config.config_path).expect("saved config should exist"),
    )
    .expect("saved config should be json");

    assert!(result.success);
    assert_eq!(output["operation"].as_str(), Some("set"));
    assert_eq!(output["setting"].as_str(), Some("browser.enabled"));
    assert_eq!(output["previousValue"], json!(false));
    assert_eq!(output["newValue"], json!(true));
    assert_eq!(saved["browser"]["enabled"], json!(true));
}

#[tokio::test]
async fn config_call_returns_structured_result_for_setting_reads() {
    let (_tmp, config) = test_config().await;
    let tool = ConfigTool::new(config, Arc::new(SecurityPolicy::default()));

    let result = tool
        .call(json!({
            "setting": "defaultTemperature"
        }))
        .await
        .expect("structured call should succeed");

    assert_eq!(result.data["operation"].as_str(), Some("get"));
    assert_eq!(result.data["setting"].as_str(), Some("defaultTemperature"));
    assert!(result.render_hint.is_some());
}
