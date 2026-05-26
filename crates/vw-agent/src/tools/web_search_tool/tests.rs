//! 验证 Web 搜索工具的参数、结果解析和 API key 轮换行为。
//! 测试保持在工具边界内，避免真实网络调用影响安全策略与可重复性。

use std::sync::Arc;

use super::super::traits::Tool;
use super::super::web_search_tool::{WebSearchTool, decode_ddg_redirect_url, strip_tags};
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use crate::app::agent::tools::{WEB_SEARCH_TOOL_ALIAS, WEB_SEARCH_TOOL_ID};
use serde_json::json;

fn new_tool(provider: &str, api_key: Option<&str>) -> WebSearchTool {
    WebSearchTool::new(
        test_security(),
        provider.to_string(),
        api_key.map(ToString::to_string),
        None,
        5,
        15,
        "test".to_string(),
    )
}

fn test_security() -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy { autonomy: AutonomyLevel::Supervised, ..SecurityPolicy::default() })
}

#[test]
fn test_tool_name() {
    let tool = new_tool("duckduckgo", None);
    assert_eq!(tool.name(), "web_search_tool");
}

#[test]
fn test_tool_description() {
    let tool = new_tool("duckduckgo", None);
    assert!(
        tool.description().contains("网络搜索") || tool.description().contains("Search the web")
    );
}

#[test]
fn test_parameters_schema() {
    let tool = new_tool("duckduckgo", None);
    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["query"].is_object());
    assert!(schema["properties"]["num"].is_object());
    assert!(schema["properties"]["numResults"].is_object());
    assert!(schema["properties"]["lr"].is_object());
}

#[test]
fn test_tool_spec_uses_claude_surface() {
    let tool = new_tool("duckduckgo", None);
    let spec = tool.spec();

    assert_eq!(spec.id, WEB_SEARCH_TOOL_ID);
    assert!(spec.aliases.iter().any(|alias| alias == WEB_SEARCH_TOOL_ALIAS));
}

#[test]
fn test_strip_tags() {
    let html = "<b>Hello</b> <i>World</i>";
    assert_eq!(strip_tags(html), "Hello World");
}

#[test]
fn test_parse_duckduckgo_results_empty() {
    let tool = new_tool("duckduckgo", None);
    let result = tool.parse_duckduckgo_results("<html>No results here</html>", "test").unwrap();
    assert!(result.contains("No results found"));
}

#[test]
fn test_parse_duckduckgo_results_with_data() {
    let tool = new_tool("duckduckgo", None);
    let html = r#"
            <a class="result__a" href="https://example.com">Example Title</a>
            <a class="result__snippet">This is a description</a>
        "#;
    let result = tool.parse_duckduckgo_results(html, "test").unwrap();
    assert!(result.contains("Example Title"));
    assert!(result.contains("https://example.com"));
}

#[test]
fn test_parse_duckduckgo_results_decodes_redirect_url() {
    let tool = new_tool("duckduckgo", None);
    let html = r#"
            <a class="result__a" href="https://duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpath%3Fa%3D1&amp;rut=test">Example Title</a>
            <a class="result__snippet">This is a description</a>
        "#;
    let result = tool.parse_duckduckgo_results(html, "test").unwrap();
    assert!(result.contains("https://example.com/path?a=1"));
    assert!(!result.contains("rut=test"));
}

#[test]
fn test_constructor_clamps_web_search_limits() {
    let tool = WebSearchTool::new(
        test_security(),
        "duckduckgo".to_string(),
        None,
        None,
        0,
        0,
        "test".to_string(),
    );
    let html = r#"
            <a class="result__a" href="https://example.com">Example Title</a>
            <a class="result__snippet">This is a description</a>
        "#;
    let result = tool.parse_duckduckgo_results(html, "test").unwrap();
    assert!(result.contains("Example Title"));
}

#[test]
fn test_args_accept_claude_compat_fields() {
    let args: super::super::web_search_tool::Args = serde_json::from_value(json!({
        "query": "rust",
        "num": 3,
        "lr": "lang_en"
    }))
    .unwrap();

    assert_eq!(args.query, "rust");
    assert_eq!(args.num, Some(3));
    assert_eq!(args.lr.as_deref(), Some("lang_en"));
}

#[tokio::test]
async fn test_execute_missing_query() {
    let tool = new_tool("duckduckgo", None);
    let result = tool.execute(json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_empty_query() {
    let tool = new_tool("duckduckgo", None);
    let result = tool.execute(json!({"query": ""})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_brave_without_api_key() {
    let tool = new_tool("brave", None);
    let result = tool.execute(json!({"query": "test"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key"));
}

#[tokio::test]
async fn test_execute_firecrawl_without_api_key() {
    let tool = new_tool("firecrawl", None);
    let result = tool.execute(json!({"query": "test"})).await;
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    if cfg!(feature = "firecrawl") {
        assert!(error.contains("api_key"));
    } else {
        assert!(error.contains("requires Cargo feature 'firecrawl'"));
    }
}

#[tokio::test]
async fn test_execute_blocked_in_read_only_mode() {
    let security =
        Arc::new(SecurityPolicy { autonomy: AutonomyLevel::ReadOnly, ..SecurityPolicy::default() });
    let tool = WebSearchTool::new(
        security,
        "duckduckgo".to_string(),
        None,
        None,
        5,
        15,
        "test".to_string(),
    );
    let result = tool.execute(json!({"query": "rust"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("read-only"));
}

#[tokio::test]
async fn test_execute_tavily_without_api_key() {
    let tool = new_tool("tavily", None);
    let result = tool.execute(json!({"query": "test"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("api_key"));
}

#[test]
fn test_parse_serper_results_with_data() {
    let tool = new_tool("serper", Some("key"));
    let parsed = json!({
        "organic": [
            {
                "title": "Example Title",
                "link": "https://example.com",
                "snippet": "Example snippet"
            }
        ]
    });

    let result = tool.parse_serper_results(&parsed, "test", "serper").unwrap();
    assert!(result.contains("via Serper"));
    assert!(result.contains("Example Title"));
    assert!(result.contains("https://example.com"));
    assert!(result.contains("Example snippet"));
}

#[test]
fn test_parse_serper_results_supports_bing_label() {
    let tool = new_tool("bing", Some("key"));
    let parsed = json!({
        "organic": [
            {
                "title": "Bing Title",
                "link": "https://bing.example.com",
                "snippet": "Bing snippet"
            }
        ]
    });

    let result = tool.parse_serper_results(&parsed, "test", "bing").unwrap();
    assert!(result.contains("via Bing"));
}

#[tokio::test]
async fn test_execute_serper_without_api_key() {
    let tool = new_tool("serper", None);
    let result = tool.execute(json!({"query": "test"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("api_key"));
}

#[tokio::test]
async fn test_execute_google_without_api_key() {
    let tool = new_tool("google", None);
    let result = tool.execute(json!({"query": "test"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("api_key"));
}

#[tokio::test]
async fn test_execute_bing_without_api_key() {
    let tool = new_tool("bing", None);
    let result = tool.execute(json!({"query": "test"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("api_key"));
}

#[test]
fn test_multiple_api_keys_parsing() {
    let tool = new_tool("tavily", Some("key1,key2,key3"));
    assert_eq!(tool.api_keys.len(), 3);
    assert_eq!(tool.api_keys[0], "key1");
    assert_eq!(tool.api_keys[1], "key2");
    assert_eq!(tool.api_keys[2], "key3");
}

#[test]
fn test_multiple_api_keys_with_spaces() {
    let tool = new_tool("tavily", Some("key1, key2 , key3"));
    assert_eq!(tool.api_keys.len(), 3);
    assert_eq!(tool.api_keys[0], "key1");
    assert_eq!(tool.api_keys[1], "key2");
    assert_eq!(tool.api_keys[2], "key3");
}

#[test]
fn test_round_robin_api_key_selection() {
    let tool = new_tool("tavily", Some("key1,key2,key3"));

    assert_eq!(tool.get_next_api_key().unwrap(), "key1");
    assert_eq!(tool.get_next_api_key().unwrap(), "key2");
    assert_eq!(tool.get_next_api_key().unwrap(), "key3");
    assert_eq!(tool.get_next_api_key().unwrap(), "key1");
}

#[test]
fn test_empty_api_key_returns_none() {
    let tool = new_tool("tavily", None);
    assert!(tool.get_next_api_key().is_none());
}

#[test]
fn test_single_api_key_works() {
    let tool = new_tool("tavily", Some("single-key"));
    assert_eq!(tool.api_keys.len(), 1);
    assert_eq!(tool.get_next_api_key().unwrap(), "single-key");
}
