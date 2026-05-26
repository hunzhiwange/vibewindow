//! 代理配置工具测试模块
//!
//! 本模块包含 ProxyConfigTool 的单元测试，验证代理配置的各种操作场景：
//! - 列出已知的代理服务键
//! - 设置服务级别的代理配置
//! - 代理设置的读取和清除操作
//! - 配置项的往返测试

use super::super::*;
use crate::Config;
use crate::app::agent::config::schema::ConfigExt;
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use serde_json::{Value, json};
use tempfile::TempDir;

/// 创建测试用的安全策略
///
/// 返回一个配置为监督模式（Supervised）的安全策略实例，
/// 使用系统临时目录作为工作空间。
fn test_security() -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy {
        autonomy: AutonomyLevel::Supervised,
        workspace_dir: std::env::temp_dir(),
        ..SecurityPolicy::default()
    })
}

/// 创建测试用的配置实例
///
/// # 参数
/// - `tmp`: 临时目录引用，用于创建配置文件和工作空间
///
/// # 返回值
/// 返回一个已保存的配置实例的 Arc 智能指针
async fn test_config(tmp: &TempDir) -> Arc<Config> {
    let config: Config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("vibewindow.json"),
        ..Config::default()
    };
    config.save().await.unwrap();
    Arc::new(config)
}

/// 测试 list_services 操作是否能返回已知的服务键
///
/// 验证当执行 "list_services" 动作时，返回的结果中包含预期的服务标识符。
#[tokio::test]
async fn list_services_action_returns_known_keys() {
    // 创建临时测试环境
    let tmp = TempDir::new().unwrap();
    let tool = ProxyConfigTool::new(test_config(&tmp).await, test_security());

    // 执行列出服务操作
    let result = tool.execute(json!({"action": "list_services"})).await.unwrap();

    // 验证操作成功且包含预期的服务键
    assert!(result.success);
    assert!(result.output.contains("provider.openai"));
    assert!(result.output.contains("tool.http_request"));
}

/// 测试使用 services 作用域时必须提供 services 条目
///
/// 验证当 scope 设置为 "services" 时，如果 services 数组为空，
/// 操作应该失败并返回相应的错误信息。
#[tokio::test]
async fn set_scope_services_requires_services_entries() {
    let tmp = TempDir::new().unwrap();
    let tool = ProxyConfigTool::new(test_config(&tmp).await, test_security());

    // 尝试设置服务级别代理，但 services 数组为空
    let result = tool
        .execute(json!({
            "action": "set",
            "enabled": true,
            "scope": "services",
            "http_proxy": "http://127.0.0.1:7890",
            "services": []
        }))
        .await
        .unwrap();

    // 验证操作失败且错误信息提到 scope='services'
    assert!(!result.success);
    assert!(result.error.unwrap_or_default().contains("proxy.scope='services'"));
}

/// 测试代理设置的往返（set 和 get）操作
///
/// 验证：
/// 1. 设置服务级别代理配置能够成功
/// 2. 通过 get 操作能够读取到之前设置的配置
/// 3. 读取的配置中包含预期的服务标识符
#[tokio::test]
async fn set_and_get_round_trip_proxy_scope() {
    let tmp = TempDir::new().unwrap();
    let tool = ProxyConfigTool::new(test_config(&tmp).await, test_security());

    // 设置服务级别的代理配置
    let set_result = tool
        .execute(json!({
            "action": "set",
            "scope": "services",
            "http_proxy": "http://127.0.0.1:7890",
            "services": ["provider.openai", "tool.http_request"]
        }))
        .await
        .unwrap();
    assert!(set_result.success, "{:?}", set_result.error);

    // 读取配置并验证
    let get_result = tool.execute(json!({"action": "get"})).await.unwrap();
    assert!(get_result.success);
    assert!(get_result.output.contains("provider.openai"));
    assert!(get_result.output.contains("services"));
}

/// 测试设置 null 值可以清除已存在的代理配置
///
/// 验证：
/// 1. 能够成功设置 http_proxy
/// 2. 通过设置 http_proxy 为 null 可以清除现有值
/// 3. 清除后读取配置，确认代理值确实为 null
#[tokio::test]
async fn set_null_proxy_url_clears_existing_value() {
    let tmp = TempDir::new().unwrap();
    let tool = ProxyConfigTool::new(test_config(&tmp).await, test_security());

    // 首先设置一个代理值
    let set_result = tool
        .execute(json!({
            "action": "set",
            "http_proxy": "http://127.0.0.1:7890"
        }))
        .await
        .unwrap();
    assert!(set_result.success, "{:?}", set_result.error);

    // 清除代理值（设置为 null）
    let clear_result = tool
        .execute(json!({
            "action": "set",
            "http_proxy": null
        }))
        .await
        .unwrap();
    assert!(clear_result.success, "{:?}", clear_result.error);

    // 验证配置已被清除
    let get_result = tool.execute(json!({"action": "get"})).await.unwrap();
    assert!(get_result.success);

    // 解析 JSON 输出并验证代理值为 null
    let parsed: Value = serde_json::from_str(&get_result.output).unwrap();
    assert!(parsed["proxy"]["http_proxy"].is_null());
    assert!(parsed["runtime_proxy"]["http_proxy"].is_null());
}
