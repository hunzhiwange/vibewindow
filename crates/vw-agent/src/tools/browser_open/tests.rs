//! BrowserOpenTool 测试模块
//!
//! 本模块包含对 `BrowserOpenTool` 工具的全面测试，覆盖以下方面：
//! - 域名标准化处理（去除协议、路径、大小写转换等）
//! - 域名白名单去重
//! - URL 验证逻辑（域名匹配、子域名、通配符等）
//! - 安全策略限制（只读模式、速率限制）
//! - 拒绝私有地址和本地地址（localhost、私有 IP 等）
//!
//! ## 测试分类
//! - **域名标准化测试**：`normalize_domain_*` 系列测试
//! - **URL 验证通过测试**：`validate_accepts_*` 系列测试
//! - **URL 验证拒绝测试**：`validate_rejects_*` 系列测试
//! - **执行阶段限制测试**：`execute_*` 系列测试

use super::super::*;
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use crate::app::agent::tools::url_validation::normalize_allowed_domains;
use crate::app::agent::tools::url_validation::normalize_domain;
use serde_json::json;

/// 创建用于测试的 BrowserOpenTool 实例
///
/// # 参数
/// - `allowed_domains`: 允许访问的域名列表，用于 URL 验证
///
/// # 返回值
/// 返回一个配置好的 `BrowserOpenTool` 实例，使用默认安全策略（Supervised 级别）
///
/// # 示例
/// ```ignore
/// let tool = test_tool(vec!["example.com", "api.example.com"]);
/// ```
fn test_tool(allowed_domains: Vec<&str>) -> BrowserOpenTool {
    let security = Arc::new(SecurityPolicy {
        autonomy: AutonomyLevel::Supervised,
        ..SecurityPolicy::default()
    });
    BrowserOpenTool::new(
        security,
        allowed_domains.into_iter().map(String::from).collect(),
        "default".into(),
    )
}

#[test]
fn browser_open_spec_uses_claude_surface() {
    let tool = test_tool(vec!["example.com"]);
    let spec = tool.spec();

    assert_eq!(spec.id, BROWSER_OPEN_TOOL_ID);
    assert!(spec.aliases.iter().any(|alias| alias == BROWSER_OPEN_TOOL_ALIAS));
}

/// 测试 normalize_domain 函数能够正确标准化域名
///
/// 验证以下处理逻辑：
/// - 去除前后空白字符
/// - 去除协议前缀（如 HTTPS://）
/// - 去除路径部分（如 /path）
/// - 转换为小写
#[test]
fn normalize_domain_strips_scheme_path_and_case() {
    let got = normalize_domain("  HTTPS://Docs.Example.com/path ").unwrap();
    assert_eq!(got, "docs.example.com");
}

/// 测试 normalize_allowed_domains 函数能够正确去重
///
/// 验证以下场景的去重处理：
/// - 不同大小写的同一域名（example.com vs EXAMPLE.COM）
/// - 带协议和不带协议的同一域名
/// - 最终只保留一个标准化后的域名
#[test]
fn normalize_allowed_domains_deduplicates() {
    let got = normalize_allowed_domains(vec![
        "example.com".into(),
        "EXAMPLE.COM".into(),
        "https://example.com/".into(),
    ]);
    assert_eq!(got, vec!["example.com".to_string()]);
}

/// 测试验证器能够接受精确匹配的域名
///
/// 当白名单包含 "example.com" 时，访问 "example.com" 应该被允许
#[test]
fn validate_accepts_exact_domain() {
    let tool = test_tool(vec!["example.com"]);
    let got = tool.validate_url("https://example.com/docs").unwrap();
    assert_eq!(got, "https://example.com/docs");
}

/// 测试验证器能够接受子域名
///
/// 当白名单包含 "example.com" 时，访问其子域名（如 api.example.com）应该被允许
#[test]
fn validate_accepts_subdomain() {
    let tool = test_tool(vec!["example.com"]);
    assert!(tool.validate_url("https://api.example.com/v1").is_ok());
}

/// 测试通配符白名单（"*"）能够接受公共主机
///
/// 当白名单为 ["*"] 时，公共互联网地址应该被允许访问
#[test]
fn validate_accepts_wildcard_allowlist_for_public_host() {
    let tool = test_tool(vec!["*"]);
    assert!(tool.validate_url("https://www.rust-lang.org").is_ok());
}

/// 测试通配符白名单仍然拒绝私有主机
///
/// 即使白名单为 ["*"]，本地/私有地址（如 localhost）仍应被拒绝，
/// 以防止 SSRF 攻击
#[test]
fn validate_wildcard_allowlist_still_rejects_private_host() {
    let tool = test_tool(vec!["*"]);
    let err = tool.validate_url("https://localhost:8443").unwrap_err().to_string();
    assert!(err.contains("local/private"));
}

/// 测试通配符子域名模式（"*.example.com"）
///
/// "*.example.com" 模式应该匹配：
/// - 根域名 example.com
/// - 任意子域名（如 sub.example.com）
/// - 不应匹配其他域名（如 other.com）
#[test]
fn validate_accepts_wildcard_subdomain_pattern() {
    let tool = test_tool(vec!["*.example.com"]);
    assert!(tool.validate_url("https://example.com").is_ok());
    assert!(tool.validate_url("https://sub.example.com").is_ok());
    assert!(tool.validate_url("https://other.com").is_err());
}

/// 测试验证器拒绝 HTTP 协议
///
/// 出于安全考虑，只允许 HTTPS 协议，HTTP 应该被拒绝
#[test]
fn validate_rejects_http() {
    let tool = test_tool(vec!["example.com"]);
    let err = tool.validate_url("http://example.com").unwrap_err().to_string();
    assert!(err.contains("https://"));
}

/// 测试验证器拒绝 localhost
///
/// 即使白名单包含 "localhost"，也应拒绝访问，防止访问本地服务
#[test]
fn validate_rejects_localhost() {
    let tool = test_tool(vec!["localhost"]);
    let err = tool.validate_url("https://localhost:8080").unwrap_err().to_string();
    assert!(err.contains("local/private"));
}

/// 测试验证器拒绝私有 IPv4 地址
///
/// 私有 IP 地址（如 192.168.x.x）应该被拒绝，防止内网探测
#[test]
fn validate_rejects_private_ipv4() {
    let tool = test_tool(vec!["192.168.1.5"]);
    let err = tool.validate_url("https://192.168.1.5").unwrap_err().to_string();
    assert!(err.contains("local/private"));
}

/// 测试验证器拒绝不在白名单中的域名
///
/// 当访问的域名不在白名单中时，应该被拒绝
#[test]
fn validate_rejects_allowlist_miss() {
    let tool = test_tool(vec!["example.com"]);
    let err = tool.validate_url("https://google.com").unwrap_err().to_string();
    assert!(err.contains("allowed_domains"));
}

/// 测试验证器拒绝包含空白字符的 URL
///
/// URL 中不应包含空白字符，这是防止 URL 解析漏洞的措施
#[test]
fn validate_rejects_whitespace() {
    let tool = test_tool(vec!["example.com"]);
    let err = tool.validate_url("https://example.com/hello world").unwrap_err().to_string();
    assert!(err.contains("whitespace"));
}

/// 测试验证器拒绝包含用户信息的 URL
///
/// URL 中不应包含用户信息部分（如 user@example.com），
/// 以防止凭证泄露和 URL 注入攻击
#[test]
fn validate_rejects_userinfo() {
    let tool = test_tool(vec!["example.com"]);
    let err = tool.validate_url("https://user@example.com").unwrap_err().to_string();
    assert!(err.contains("userinfo"));
}

/// 测试白名单为空时的行为
///
/// 当白名单为空时，所有 URL 都应该被拒绝
#[test]
fn validate_requires_allowlist() {
    let security = Arc::new(SecurityPolicy::default());
    let tool = BrowserOpenTool::new(security, vec![], "default".into());
    let err = tool.validate_url("https://example.com").unwrap_err().to_string();
    assert!(err.contains("allowed_domains"));
}

/// 测试只读模式下阻止执行
///
/// 当安全策略的 autonomy 级别为 ReadOnly 时，
/// execute 方法应该拒绝执行并返回错误
#[tokio::test]
async fn execute_blocks_readonly_mode() {
    // 配置只读模式的安全策略
    let security =
        Arc::new(SecurityPolicy { autonomy: AutonomyLevel::ReadOnly, ..SecurityPolicy::default() });
    let tool = BrowserOpenTool::new(security, vec!["example.com".into()], "default".into());

    // 执行应该失败
    let result = tool.execute(json!({"url": "https://example.com"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("read-only"));
}

/// 测试速率限制生效时阻止执行
///
/// 当安全策略的 max_actions_per_hour 为 0 时，
/// execute 方法应该拒绝执行并返回速率限制错误
#[tokio::test]
async fn execute_blocks_when_rate_limited() {
    // 配置速率限制为 0 的安全策略
    let security =
        Arc::new(SecurityPolicy { max_actions_per_hour: 0, ..SecurityPolicy::default() });
    let tool = BrowserOpenTool::new(security, vec!["example.com".into()], "default".into());

    // 执行应该因速率限制而失败
    let result = tool.execute(json!({"url": "https://example.com"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("rate limit"));
}
