use super::*;
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use crate::app::agent::tools::traits::Tool;
use serde_json::json;
use std::sync::Arc;

fn tool(provider: &str, key: Option<&str>) -> WebSearchTool {
    WebSearchTool::new(
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            ..SecurityPolicy::default()
        }),
        provider.to_string(),
        key.map(str::to_string),
        None,
        2,
        0,
        "ua".to_string(),
    )
}

#[test]
fn constructor_helpers_and_result_parsers_are_covered() {
    let tool = tool(" BRAVE ", Some("a, ,b"));
    assert_eq!(tool.provider, "brave");
    assert_eq!(tool.timeout_secs, 1);
    assert_eq!(tool.get_next_api_key().as_deref(), Some("a"));
    assert_eq!(tool.provider_label(), "Brave");
    assert_eq!(tool.resolve_serper_endpoint("bing"), "https://bing.serper.dev/search");

    let (provider, rows) = WebSearchTool::parse_formatted_results(
        "Search results for: q (via Brave)\n1. One\n   https://one.test\n   A\n   B",
    );
    assert_eq!(provider.as_deref(), Some("Brave"));
    assert_eq!(rows[0].snippet.as_deref(), Some("A B"));

    let ddg = tool.parse_duckduckgo_results(
        r#"<a class="result__a" href="https://duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com">Title</a><a class="result__snippet">Desc</a>"#,
        "q",
    ).unwrap();
    assert!(ddg.contains("https://example.com"));
    assert!(
        tool.parse_serper_results(&json!({"organic": []}), "q", "serper")
            .unwrap()
            .contains("No results")
    );
    assert!(tool.parse_serper_results(&json!({}), "q", "serper").is_err());
    assert_eq!(decode_ddg_redirect_url("x?uddg=https%3A%2F%2Fe.test&rut=1"), "https://e.test");
    assert_eq!(strip_tags("<b>x</b>"), "x");
}

#[test]
fn args_schema_and_spec_cover_claude_surface() {
    let args: Args =
        serde_json::from_value(json!({"query": "rust", "numResults": 3, "lang": "en"})).unwrap();
    assert_eq!(args.query, "rust");
    assert_eq!(args.num, Some(3));
    assert_eq!(args.lr.as_deref(), Some("en"));

    let tool = tool("duckduckgo", None);
    assert!(tool.parameters_schema()["properties"]["numResults"].is_object());
    assert!(tool.spec().read_only);
}

#[tokio::test]
async fn execute_rejects_readonly_empty_unknown_and_missing_key_without_network() {
    let readonly = WebSearchTool::new(
        Arc::new(SecurityPolicy { autonomy: AutonomyLevel::ReadOnly, ..SecurityPolicy::default() }),
        "duckduckgo".into(),
        None,
        None,
        2,
        1,
        "ua".into(),
    );
    assert!(
        readonly
            .execute(json!({"query": "rust"}))
            .await
            .unwrap()
            .error
            .unwrap()
            .contains("read-only")
    );
    assert!(tool("duckduckgo", None).execute(json!({"query": ""})).await.is_err());
    assert!(
        tool("unknown", None)
            .execute(json!({"query": "rust"}))
            .await
            .unwrap_err()
            .to_string()
            .contains("Unknown search provider")
    );
    assert!(
        tool("brave", None)
            .execute(json!({"query": "rust"}))
            .await
            .unwrap_err()
            .to_string()
            .contains("API key")
    );
}
