//! Composio 工具的单元测试模块
//!
//! 本模块包含对 Composio 工具（`ComposioTool`）的全面测试用例，涵盖以下方面：
//! - 工具基本属性验证（名称、描述、参数模式）
//! - 执行动作的各种场景（缺失参数、未知动作、安全策略限制等）
//! - API 响应数据结构的反序列化
//! - 工具 slug 和缓存键的规范化处理
//! - 已连接账户的可用性判断和选择逻辑
//! - API 请求构建和响应解析
//!
//! Composio 是一个集成平台，提供 1000+ 外部工具的统一访问接口。
//! 本测试套件确保 Composio 工具在 VibeWindow 代理运行时中正确运作。

use super::super::*;
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use crate::app::agent::tools::composio::api;
use crate::app::agent::tools::composio::types::{
    ComposioAction, ComposioAuthConfig, ComposioConnectedAccount, ComposioToolsResponse,
};
use crate::app::agent::tools::composio::util;
use crate::app::agent::tools::composio::util::{
    build_connected_account_hint, build_tool_slug_candidates, extract_api_error_message,
    extract_connected_account_id, extract_redirect_url, infer_app_slug_from_action_name,
    normalize_action_cache_key, normalize_app_slug, normalize_entity_id, normalize_tool_slug,
};
use serde_json::json;
use std::collections::HashMap;

/// 创建用于测试的默认安全策略
///
/// 返回一个使用默认配置的安全策略实例，用于大多数测试场景。
/// 默认策略允许所有操作，适用于正常功能测试。
///
/// # 返回值
///
/// 返回包装在 `Arc` 中的默认安全策略实例
fn test_security() -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy::default())
}

/// 验证 Composio 工具具有正确的名称
///
/// 测试工具的 `name()` 方法返回预期的标识符 "composio"。
/// 这是工具在系统中注册和调用的唯一标识。
#[test]
fn composio_tool_has_correct_name() {
    let tool = ComposioTool::new("test-key", None, test_security());
    assert_eq!(tool.name(), "composio");
}

/// 验证 Composio 工具具有有效的描述信息
///
/// 测试工具的 `description()` 方法返回非空描述，
/// 且描述中包含 "1000+" 字样，表明工具提供大量集成选项。
#[test]
fn composio_tool_has_description() {
    let _tool = ComposioTool::new("test-key", None, test_security());
    assert!(!ComposioTool::new("test-key", None, test_security()).description().is_empty());
    assert!(ComposioTool::new("test-key", None, test_security()).description().contains("1000+"));
}

/// 验证 Composio 工具参数模式包含所有必需字段
///
/// 测试工具的 `parameters_schema()` 返回的模式结构正确，
/// 包含所有必要的属性字段，且 "action" 被标记为必需字段。
/// 同时验证 action 枚举包含预期的动作类型（如 "list_accounts"）。
#[test]
fn composio_tool_schema_has_required_fields() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let schema = tool.parameters_schema();

    // 验证所有属性字段都是对象类型
    assert!(schema["properties"]["action"].is_object());
    assert!(schema["properties"]["action_name"].is_object());
    assert!(schema["properties"]["tool_slug"].is_object());
    assert!(schema["properties"]["params"].is_object());
    assert!(schema["properties"]["app"].is_object());
    assert!(schema["properties"]["auth_config_id"].is_object());
    assert!(schema["properties"]["connected_account_id"].is_object());

    // 验证 "action" 是必需字段
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("action")));

    // 验证 action 枚举包含 list_accounts
    let enum_values = schema["properties"]["action"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    assert!(enum_values.contains(&"list_accounts"));
}

/// 验证工具规格可以正确序列化和反序列化
///
/// 测试 `spec()` 方法返回的工具规格结构正确，
/// 名称和参数字段符合预期。
#[test]
fn composio_tool_spec_roundtrip() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let spec = tool.spec();
    assert_eq!(spec.name, "composio");
    assert!(spec.parameters.is_object());
}

/// 验证执行时缺少 action 参数会返回错误
///
/// 当调用 `execute()` 时未提供必需的 "action" 字段，
/// 应返回错误而非成功执行。
#[tokio::test]
async fn execute_missing_action_returns_error() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let result = tool.execute(json!({})).await;
    assert!(result.is_err());
}

/// 验证执行未知动作会返回错误响应
///
/// 当提供的 action 值不在支持的枚举列表中时，
/// 工具应返回失败结果并包含 "Unknown action" 错误信息。
#[tokio::test]
async fn execute_unknown_action_returns_error() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let result = tool.execute(json!({"action": "unknown"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Unknown action"));
}

/// 验证执行动作时缺少 action_name 参数会返回错误
///
/// 当 action 设置为 "execute" 但未提供 "action_name" 字段时，
/// 工具应返回错误，因为 execute 需要指定具体动作名称。
#[tokio::test]
async fn execute_without_action_name_returns_error() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let result = tool.execute(json!({"action": "execute"})).await;
    assert!(result.is_err());
}

/// 验证连接操作时缺少目标参数会返回错误
///
/// 当 action 设置为 "connect" 但未提供目标应用信息时，
/// 工具应返回错误，因为连接操作需要指定要连接的应用。
#[tokio::test]
async fn connect_without_target_returns_error() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let result = tool.execute(json!({"action": "connect"})).await;
    assert!(result.is_err());
}

/// 验证只读模式下执行动作会被阻止
///
/// 当安全策略的自主级别设置为 `ReadOnly` 时，
/// 任何执行动作都应被阻止，并返回包含 "read-only mode" 的错误信息。
#[tokio::test]
async fn execute_blocked_in_readonly_mode() {
    // 创建只读模式的安全策略
    let readonly =
        Arc::new(SecurityPolicy { autonomy: AutonomyLevel::ReadOnly, ..SecurityPolicy::default() });
    let tool = ComposioTool::new("test-key", None, readonly);

    // 尝试执行动作
    let result = tool
        .execute(json!({
            "action": "execute",
            "action_name": "GITHUB_LIST_REPOS"
        }))
        .await
        .unwrap();

    // 验证执行被阻止
    assert!(!result.success);
    assert!(result.error.as_deref().unwrap_or("").contains("read-only mode"));
}

/// 验证速率限制生效时会阻止执行
///
/// 当安全策略设置每小时最大操作数为 0 时（即完全禁用），
/// 执行动作应被阻止，并返回包含 "Rate limit exceeded" 的错误信息。
#[tokio::test]
async fn execute_blocked_when_rate_limited() {
    // 创建速率限制的安全策略（每小时 0 次操作）
    let limited = Arc::new(SecurityPolicy { max_actions_per_hour: 0, ..SecurityPolicy::default() });
    let tool = ComposioTool::new("test-key", None, limited);

    // 尝试执行动作
    let result = tool
        .execute(json!({
            "action": "execute",
            "action_name": "GITHUB_LIST_REPOS"
        }))
        .await
        .unwrap();

    // 验证执行被速率限制阻止
    assert!(!result.success);
    assert!(result.error.as_deref().unwrap_or("").contains("Rate limit exceeded"));
}

/// 验证 ComposioAction 结构可以正确反序列化
///
/// 测试从 JSON 字符串解析 ComposioAction 时，
/// 各字段（name、appName、description、enabled）能正确映射。
#[test]
fn composio_action_deserializes() {
    let json_str = r#"{"name": "GMAIL_FETCH_EMAILS", "appName": "gmail", "description": "Fetch emails", "enabled": true}"#;
    let action: ComposioAction = serde_json::from_str(json_str).unwrap();
    assert_eq!(action.name, "GMAIL_FETCH_EMAILS");
    assert_eq!(action.app_name.as_deref(), Some("gmail"));
    assert!(action.enabled);
}

/// 验证 ComposioToolsResponse 可以正确反序列化包含动作列表的响应
///
/// 测试解析包含单个动作项的 V3 API 响应时，
/// items 数组正确解析，slug 字段映射正确。
#[test]
fn composio_tools_response_deserializes() {
    let json_str = r#"{"items": [{"slug": "test-action", "name": "TEST_ACTION", "appName": "test", "description": "A test"}]}"#;
    let resp: ComposioToolsResponse = serde_json::from_str(json_str).unwrap();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].slug.as_deref(), Some("test-action"));
}

/// 验证空的动作列表可以正确反序列化
///
/// 测试当 API 返回空 items 数组时，
/// 响应结构正确解析且 items 为空。
#[test]
fn composio_tools_response_empty() {
    let json_str = r#"{"items": []}"#;
    let resp: ComposioToolsResponse = serde_json::from_str(json_str).unwrap();
    assert!(resp.items.is_empty());
}

/// 验证缺少 items 字段时响应默认为空列表
///
/// 测试当 API 响应中不包含 items 字段时，
/// 反序列化结果默认为空列表而非错误。
#[test]
fn composio_tools_response_missing_items_defaults() {
    let json_str = r"{}";
    let resp: ComposioToolsResponse = serde_json::from_str(json_str).unwrap();
    assert!(resp.items.is_empty());
}

/// 验证 V3 API 工具响应可以正确映射为动作列表
///
/// 测试 V3 API 响应格式（使用 toolkit 嵌套结构）
/// 能通过 `map_v3_tools_to_actions` 函数正确转换为标准动作格式。
/// 验证 slug、name、app_name 和 description 字段的映射。
#[test]
fn composio_v3_tools_response_maps_to_actions() {
    // V3 API 响应格式：使用 toolkit 嵌套结构
    let json_str = r#"{
            "items": [
                {
                    "slug": "gmail-fetch-emails",
                    "name": "Gmail Fetch Emails",
                    "description": "Fetch inbox emails",
                    "toolkit": { "slug": "gmail", "name": "Gmail" }
                }
            ]
        }"#;
    let resp: ComposioToolsResponse = serde_json::from_str(json_str).unwrap();

    // 将 V3 响应映射为动作列表
    let actions = util::map_v3_tools_to_actions(resp.items);

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "gmail-fetch-emails");
    assert_eq!(actions[0].app_name.as_deref(), Some("gmail"));
    assert_eq!(actions[0].description.as_deref(), Some("Fetch inbox emails"));
}

/// 验证 normalize_entity_id 对空白输入返回默认值
///
/// 测试当 entity_id 为空白字符串时，
/// 函数返回 "default" 作为回退值；非空白值保持原样。
#[test]
fn normalize_entity_id_falls_back_to_default_when_blank() {
    assert_eq!(normalize_entity_id("   "), "default");
    assert_eq!(normalize_entity_id("workspace-user"), "workspace-user");
}

/// 验证 normalize_tool_slug 支持遗留的动作名称格式
///
/// 测试函数能将大写下划线格式（GMAIL_FETCH_EMAILS）
/// 转换为小写连字符格式（gmail-fetch-emails），
/// 并能正确处理首尾空格。
#[test]
fn normalize_tool_slug_supports_legacy_action_name() {
    assert_eq!(normalize_tool_slug("GMAIL_FETCH_EMAILS"), "gmail-fetch-emails");
    assert_eq!(normalize_tool_slug(" github-list-repos "), "github-list-repos");
}

/// 验证 build_tool_slug_candidates 生成常见变体
///
/// 测试函数为给定的动作名称生成所有可能的 slug 变体，
/// 包括原始格式、小写连字符格式、小写下划线格式。
/// 这用于在缓存查找时匹配不同命名约定。
#[test]
fn build_tool_slug_candidates_cover_common_variants() {
    let candidates = build_tool_slug_candidates("GMAIL_FETCH_EMAILS");

    // 验证原始格式排在首位
    assert_eq!(candidates.first().map(String::as_str), Some("GMAIL_FETCH_EMAILS"));
    // 验证包含小写连字符格式
    assert!(candidates.contains(&"gmail-fetch-emails".to_string()));
    // 验证包含小写下划线格式
    assert!(candidates.contains(&"gmail_fetch_emails".to_string()));
    // 验证包含原始格式
    assert!(candidates.contains(&"GMAIL_FETCH_EMAILS".to_string()));

    // 测试连字符格式的变体生成
    let hyphen = build_tool_slug_candidates("github-list-repos");
    assert_eq!(hyphen.first().map(String::as_str), Some("github-list-repos"));
    assert!(hyphen.contains(&"github_list_repos".to_string()));
}

/// 验证 normalize_action_cache_key 合并下划线和连字符变体
///
/// 测试函数将不同格式的动作名称统一为小写连字符格式，
/// 用于缓存键的规范化。空白输入应返回 None。
#[test]
fn normalize_action_cache_key_merges_underscore_and_hyphen_variants() {
    // 大写下划线格式转换为小写连字符
    assert_eq!(
        normalize_action_cache_key(" GMAIL_FETCH_EMAILS ").as_deref(),
        Some("gmail-fetch-emails")
    );
    // 已是连字符格式保持不变
    assert_eq!(
        normalize_action_cache_key("gmail-fetch-emails").as_deref(),
        Some("gmail-fetch-emails")
    );
    // 空白输入返回 None
    assert_eq!(normalize_action_cache_key("  ").as_deref(), None);
}

/// 验证 normalize_app_slug 移除空格并规范化大小写
///
/// 测试函数将应用名称转换为小写格式，
/// 并将内部下划线替换为连字符。
#[test]
fn normalize_app_slug_removes_spaces_and_normalizes_case() {
    assert_eq!(normalize_app_slug(" Gmail "), "gmail");
    assert_eq!(normalize_app_slug("GITHUB_APP"), "github-app");
}

/// 验证 infer_app_slug_from_action_name 处理 V2 和 V3 格式
///
/// 测试函数能从 V3 格式（gmail-fetch-emails）和
/// V2 格式（GMAIL_FETCH_EMAILS）的动作名称中推断应用 slug。
/// 无法推断时返回 None。
#[test]
fn infer_app_slug_from_action_name_handles_v2_and_v3_formats() {
    // V3 格式：小写连字符
    assert_eq!(infer_app_slug_from_action_name("gmail-fetch-emails").as_deref(), Some("gmail"));
    // V2 格式：大写下划线
    assert_eq!(infer_app_slug_from_action_name("GMAIL_FETCH_EMAILS").as_deref(), Some("gmail"));
    // 无法推断的通用动作名
    assert!(infer_app_slug_from_action_name("execute").is_none());
}

/// 验证 connected_account_cache_key 生成稳定的缓存键
///
/// 测试函数为给定的应用名称和 entity ID 生成一致的缓存键，
/// 格式为 "entity_id:app_slug"（均为小写）。
#[test]
fn connected_account_cache_key_is_stable() {
    assert_eq!(util::connected_account_cache_key("GMAIL", " default "), "default:gmail");
}

/// 验证 build_connected_account_hint 在缺少账户引用时返回指导信息
///
/// 测试当 connected_account_id 未提供时，
/// 函数返回包含 list_accounts 动作指导、
/// 应用名称和 entity_id 的提示信息。
#[test]
fn build_connected_account_hint_returns_guidance_when_missing_ref() {
    let hint = build_connected_account_hint(Some("gmail"), Some("default"), None);
    assert!(hint.contains("list_accounts"));
    assert!(hint.contains("gmail"));
    assert!(hint.contains("default"));
}

/// 验证 build_connected_account_hint 在无应用名称时仍可执行
///
/// 测试当 app 参数为 None 时，
/// 提示信息仍包含 list_accounts 指导和 entity_id，
/// 但不包含应用特定信息。
#[test]
fn build_connected_account_hint_without_app_is_still_actionable() {
    let hint = build_connected_account_hint(None, Some("default"), None);
    assert!(hint.contains("list_accounts"));
    assert!(hint.contains("entity_id='default'"));
    assert!(!hint.contains("app='"));
}

/// 验证 ComposioConnectedAccount 的可用状态判断
///
/// 测试 `is_usable()` 方法对 INITIALIZING、ACTIVE 和 INITIATED
/// 状态返回 true，这些状态表示账户可用或正在激活中。
#[test]
fn connected_account_is_usable_for_initializing_active_and_initiated() {
    for status in ["INITIALIZING", "ACTIVE", "INITIATED"] {
        let account = ComposioConnectedAccount {
            id: "ca_1".to_string(),
            status: status.to_string(),
            toolkit: None,
        };
        assert!(account.is_usable(), "status {status} should be usable");
    }
}

/// 验证 extract_connected_account_id 支持常见的 JSON 结构
///
/// 测试函数能从不同格式的 JSON 响应中提取 connected_account_id：
/// - 根级别 snake_case（connected_account_id）
/// - 根级别 camelCase（connectedAccountId）
/// - 嵌套在 data 对象中
#[test]
fn extract_connected_account_id_supports_common_shapes() {
    // 根级别 snake_case 格式
    let root = json!({"connected_account_id": "ca_root"});
    // 根级别 camelCase 格式
    let camel = json!({"connectedAccountId": "ca_camel"});
    // 嵌套在 data 对象中
    let nested = json!({"data": {"connected_account_id": "ca_nested"}});

    assert_eq!(extract_connected_account_id(&root).as_deref(), Some("ca_root"));
    assert_eq!(extract_connected_account_id(&camel).as_deref(), Some("ca_camel"));
    assert_eq!(extract_connected_account_id(&nested).as_deref(), Some("ca_nested"));
}

/// 验证 extract_redirect_url 支持 V2 和 V3 的响应结构
///
/// 测试函数能从不同版本的 API 响应中提取重定向 URL：
/// - V2 格式：camelCase（redirectUrl）
/// - V3 格式：snake_case（redirect_url）
/// - 嵌套在 data 对象中
#[test]
fn extract_redirect_url_supports_v2_and_v3_shapes() {
    // V2 格式：camelCase
    let v2 = json!({"redirectUrl": "https://app.composio.dev/connect-v2"});
    // V3 格式：snake_case
    let v3 = json!({"redirect_url": "https://app.composio.dev/connect-v3"});
    // 嵌套格式
    let nested = json!({"data": {"redirect_url": "https://app.composio.dev/connect-nested"}});

    assert_eq!(extract_redirect_url(&v2).as_deref(), Some("https://app.composio.dev/connect-v2"));
    assert_eq!(extract_redirect_url(&v3).as_deref(), Some("https://app.composio.dev/connect-v3"));
    assert_eq!(
        extract_redirect_url(&nested).as_deref(),
        Some("https://app.composio.dev/connect-nested")
    );
}

/// 验证 ComposioAuthConfig 优先使用 status 字段判断启用状态
///
/// 测试 `is_enabled()` 方法的判断逻辑：
/// - 优先检查 status 字段是否为 "ENABLED"
/// - status 不存在或非 ENABLED 时，回退到 enabled 布尔字段
#[test]
fn auth_config_prefers_enabled_status() {
    // 使用 status 字段判断为启用
    let enabled =
        ComposioAuthConfig { id: "cfg_1".into(), status: Some("ENABLED".into()), enabled: None };
    // status 为 DISABLED，enabled 为 false，判断为禁用
    let disabled = ComposioAuthConfig {
        id: "cfg_2".into(),
        status: Some("DISABLED".into()),
        enabled: Some(false),
    };

    assert!(enabled.is_enabled());
    assert!(!disabled.is_enabled());
}

/// 验证 extract_api_error_message 从常见 JSON 结构提取错误信息
///
/// 测试函数能从不同格式的错误响应中提取消息：
/// - 嵌套格式：{"error": {"message": "..."}}
/// - 扁平格式：{"message": "..."}
/// - 非 JSON 字符串返回 None
#[test]
fn extract_api_error_message_from_common_shapes() {
    // 嵌套格式
    let nested = r#"{"error":{"message":"tool not found"}}"#;
    // 扁平格式
    let flat = r#"{"message":"invalid api key"}"#;

    assert_eq!(extract_api_error_message(nested).as_deref(), Some("tool not found"));
    assert_eq!(extract_api_error_message(flat).as_deref(), Some("invalid api key"));
    // 非 JSON 返回 None
    assert_eq!(extract_api_error_message("not-json"), None);
}

/// 验证 ComposioAction 可以正确处理 null 字段
///
/// 测试当 JSON 中某些字段为 null 时，
/// 反序列化能正确处理，可选字段映射为 None。
#[test]
fn composio_action_with_null_fields() {
    let json_str =
        r#"{"name": "TEST_ACTION", "appName": null, "description": null, "enabled": false}"#;
    let action: ComposioAction = serde_json::from_str(json_str).unwrap();
    assert_eq!(action.name, "TEST_ACTION");
    assert!(action.app_name.is_none());
    assert!(action.description.is_none());
    assert!(!action.enabled);
}

/// 验证 ComposioAction 可以正确处理特殊字符
///
/// 测试描述字段包含特殊字符（&、<、>、'、"）
/// 时，反序列化能正确保留这些字符。
#[test]
fn composio_action_with_special_characters() {
    let json_str = r#"{"name": "GMAIL_SEND_EMAIL_WITH_ATTACHMENT", "appName": "gmail", "description": "Send email with attachment & special chars: <>'\"\"", "enabled": true}"#;
    let action: ComposioAction = serde_json::from_str(json_str).unwrap();
    assert_eq!(action.name, "GMAIL_SEND_EMAIL_WITH_ATTACHMENT");
    assert!(action.description.as_ref().unwrap().contains('&'));
    assert!(action.description.as_ref().unwrap().contains('<'));
}

/// 验证 ComposioAction 可以正确处理 Unicode 字符
///
/// 测试描述字段包含 emoji 和 Unicode 字符时，
/// 反序列化能正确保留这些字符。
#[test]
fn composio_action_with_unicode() {
    let json_str = r#"{"name": "SLACK_SEND_MESSAGE", "appName": "slack", "description": "Send message with emoji 🎉 and unicode Ω", "enabled": true}"#;
    let action: ComposioAction = serde_json::from_str(json_str).unwrap();
    assert!(action.description.as_ref().unwrap().contains("🎉"));
    assert!(action.description.as_ref().unwrap().contains("Ω"));
}

/// 验证格式错误的 JSON 返回反序列化错误
///
/// 测试当 JSON 格式不正确（如末尾多余逗号）时，
/// 反序列化返回错误而非 panic。
#[test]
fn composio_malformed_json_returns_error() {
    let json_str = r#"{"name": "TEST_ACTION", "appName": "gmail", }"#;
    let result: Result<ComposioAction, _> = serde_json::from_str(json_str);
    assert!(result.is_err());
}

/// 验证空 JSON 字符串返回反序列化错误
///
/// 测试当 JSON 内容为空或仅包含空白时，
/// 反序列化返回错误。
#[test]
fn composio_empty_json_string_returns_error() {
    let json_str = r#" ""#;
    let result: Result<ComposioAction, _> = serde_json::from_str(json_str);
    assert!(result.is_err());
}

/// 验证可以处理大型动作列表
///
/// 测试解析包含 100 个动作项的响应时，
/// 反序列化能正确处理且性能可接受。
#[test]
fn composio_large_actions_list() {
    // 构建包含 100 个动作的列表
    let mut items = Vec::new();
    for i in 0..100 {
        items.push(json!({
            "slug": format!("action-{i}"),
            "name": format!("ACTION_{i}"),
            "app_name": "test",
            "description": "Test action"
        }));
    }
    let json_str = json!({"items": items}).to_string();
    let resp: ComposioToolsResponse = serde_json::from_str(&json_str).unwrap();
    assert_eq!(resp.items.len(), 100);
}

/// 验证 Composio API 基础 URL 为 V3 版本
///
/// 测试 `composio_api_base_v3()` 返回正确的 V3 API 端点。
#[test]
fn composio_api_base_url_is_v3() {
    assert_eq!(api::composio_api_base_v3(), "https://backend.composio.dev/api/v3");
}

/// 验证 build_execute_action_v3_request 使用正确的端点和请求体
///
/// 测试函数构建的 V3 执行请求：
/// - URL 使用 /tools/execute/{slug} 端点
/// - 请求体包含 arguments、version、user_id、connected_account_id
/// - 空白可选字段被正确处理
#[test]
fn build_execute_action_v3_request_uses_fixed_endpoint_and_body_account_id() {
    let (url, body) = ComposioTool::build_execute_action_v3_request(
        "gmail-send-email",
        json!({"to": "test@example.com"}),
        None,
        Some("workspace-user"),
        Some("account-42"),
    );

    // 验证 URL 格式
    assert_eq!(url, "https://backend.composio.dev/api/v3/tools/execute/gmail-send-email");
    // 验证请求体字段
    assert_eq!(body["arguments"]["to"], json!("test@example.com"));
    assert_eq!(body["version"], json!(api::composio_tool_version_latest()));
    assert_eq!(body["user_id"], json!("workspace-user"));
    assert_eq!(body["connected_account_id"], json!("account-42"));
}

/// 验证 build_list_actions_v3_query 请求最新版本
///
/// 测试无应用过滤时，查询参数包含：
/// - toolkit_versions 设置为最新版本
/// - limit 设置为 200
/// - 不包含 toolkits 或 toolkit_slug 过滤参数
#[test]
fn build_list_actions_v3_query_requests_latest_versions() {
    let query = ComposioTool::build_list_actions_v3_query(None)
        .into_iter()
        .collect::<HashMap<String, String>>();

    // 验证版本参数
    assert_eq!(
        query.get("toolkit_versions"),
        Some(&api::composio_tool_version_latest().to_string())
    );
    // 验证限制参数
    assert_eq!(query.get("limit"), Some(&"200".to_string()));
    // 验证无过滤参数
    assert!(!query.contains_key("toolkits"));
    assert!(!query.contains_key("toolkit_slug"));
}

/// 验证 build_list_actions_v3_query 在有应用过滤时添加过滤参数
///
/// 测试当提供应用名称时，查询参数额外包含：
/// - toolkits 参数（规范化后的小写格式）
/// - toolkit_slug 参数（与 toolkits 相同）
#[test]
fn build_list_actions_v3_query_adds_app_filters_when_present() {
    let query = ComposioTool::build_list_actions_v3_query(Some(" github "))
        .into_iter()
        .collect::<HashMap<String, String>>();

    // 验证版本参数
    assert_eq!(
        query.get("toolkit_versions"),
        Some(&api::composio_tool_version_latest().to_string())
    );
    // 验证应用过滤参数
    assert_eq!(query.get("toolkits"), Some(&"github".to_string()));
    assert_eq!(query.get("toolkit_slug"), Some(&"github".to_string()));
}

/// 验证多个可用账户存在时选择第一个
///
/// 测试当有多个 ACTIVE 状态的账户时，
/// 解析逻辑选择列表中的第一个可用账户。
#[test]
fn resolve_picks_first_usable_when_multiple_accounts_exist() {
    let accounts = vec![
        ComposioConnectedAccount {
            id: "ca_old".to_string(),
            status: "ACTIVE".to_string(),
            toolkit: None,
        },
        ComposioConnectedAccount {
            id: "ca_new".to_string(),
            status: "ACTIVE".to_string(),
            toolkit: None,
        },
    ];
    let resolved = accounts.into_iter().find(|a| a.is_usable()).map(|a| a.id);
    assert_eq!(resolved.as_deref(), Some("ca_old"));
}

/// 验证跳过不可用的头部账户选择后续可用账户
///
/// 测试当列表开头是不可用账户（如 DISCONNECTED）时，
/// 解析逻辑跳过它并选择第一个可用账户。
#[test]
fn resolve_picks_first_usable_skipping_unusable_head() {
    let accounts = vec![
        // 不可用的账户
        ComposioConnectedAccount {
            id: "ca_dead".to_string(),
            status: "DISCONNECTED".to_string(),
            toolkit: None,
        },
        // 可用的账户
        ComposioConnectedAccount {
            id: "ca_live".to_string(),
            status: "ACTIVE".to_string(),
            toolkit: None,
        },
    ];
    let resolved = accounts.into_iter().find(|a| a.is_usable()).map(|a| a.id);
    assert_eq!(resolved.as_deref(), Some("ca_live"));
}

/// 验证无可用账户时返回 None
///
/// 测试当所有账户都是不可用状态（如 DISCONNECTED）时，
/// 解析逻辑返回 None。
#[test]
fn resolve_returns_none_when_no_usable_accounts() {
    let accounts = vec![ComposioConnectedAccount {
        id: "ca_dead".to_string(),
        status: "DISCONNECTED".to_string(),
        toolkit: None,
    }];
    let resolved = accounts.into_iter().find(|a| a.is_usable()).map(|a| a.id);
    assert!(resolved.is_none());
}

/// 验证空账户列表时返回 None
///
/// 测试当账户列表为空时，解析逻辑返回 None。
#[test]
fn resolve_returns_none_for_empty_accounts() {
    let accounts: Vec<ComposioConnectedAccount> = vec![];
    let resolved = accounts.into_iter().find(|a| a.is_usable()).map(|a| a.id);
    assert!(resolved.is_none());
}

/// 验证 connected_accounts 别名与 list_accounts 行为一致
///
/// 测试 "connected_accounts" 作为 "list_accounts" 的别名，
/// 两者执行结果应相同（都会因缺少 API 密钥而失败，
/// 但错误信息不应是 "Unknown action"）。
#[tokio::test]
async fn connected_accounts_alias_dispatches_same_as_list_accounts() {
    let tool = ComposioTool::new("test-key", None, test_security());

    // 执行两个别名
    let r1 = tool.execute(json!({"action": "list_accounts"})).await.unwrap();
    let r2 = tool.execute(json!({"action": "connected_accounts"})).await.unwrap();

    // 两者都应失败（无真实 API 密钥），但不是未知动作错误
    assert!(!r1.success);
    assert!(!r2.success);
    let e1 = r1.error.unwrap_or_default();
    let e2 = r2.error.unwrap_or_default();
    assert!(!e1.contains("Unknown action"), "list_accounts: {e1}");
    assert!(!e2.contains("Unknown action"), "connected_accounts: {e2}");
}

/// 验证参数模式的 action 枚举包含 connected_accounts 别名
///
/// 测试工具的参数模式中，action 字段的枚举值
/// 同时包含 "connected_accounts" 和 "list_accounts"。
#[test]
fn schema_enum_includes_connected_accounts_alias() {
    let tool = ComposioTool::new("test-key", None, test_security());
    let schema = tool.parameters_schema();
    let values: Vec<&str> = schema["properties"]["action"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(values.contains(&"connected_accounts"));
    assert!(values.contains(&"list_accounts"));
}

/// 验证工具描述提及 connected_accounts 别名
///
/// 测试工具的 description() 返回值包含 "connected_accounts"，
/// 帮助用户了解此别名选项。
#[test]
fn description_mentions_connected_accounts() {
    let tool = ComposioTool::new("test-key", None, test_security());
    assert!(tool.description().contains("connected_accounts"));
}

/// 验证 build_execute_action_v3_request 省略空白的可选字段
///
/// 测试当可选参数为 None 或空白字符串时，
/// 构建的请求体不包含这些字段，保持请求简洁。
#[test]
fn build_execute_action_v3_request_drops_blank_optional_fields() {
    let (url, body) = ComposioTool::build_execute_action_v3_request(
        "github-list-repos",
        json!({}),
        None,
        None,
        Some("   "), // 空白的 connected_account_id
    );

    // 验证 URL
    assert_eq!(url, "https://backend.composio.dev/api/v3/tools/execute/github-list-repos");
    // 验证必需字段
    assert_eq!(body["arguments"], json!({}));
    assert_eq!(body["version"], json!(api::composio_tool_version_latest()));
    // 验证空白可选字段被省略
    assert!(body.get("connected_account_id").is_none());
    assert!(body.get("user_id").is_none());
}
