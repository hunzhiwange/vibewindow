use super::*;
use async_trait::async_trait;
use serde_json::{Value, json};

struct FailingTool;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for FailingTool {
    fn name(&self) -> &str {
        "file_write"
    }
    fn description(&self) -> &str {
        "write"
    }
    fn parameters_schema(&self) -> Value {
        json!({"type": "object"})
    }
    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: false, output: "out".into(), error: Some("err".into()) })
    }
}

#[test]
fn builders_dto_and_result_helpers_cover_structured_paths() {
    let spec = ToolSpec::new("id", "desc", json!({"type": "object"}))
        .with_display_name("Display")
        .with_aliases(vec!["a"])
        .with_read_only(true)
        .with_destructive(true)
        .with_concurrency_safe(true)
        .with_requires_user_interaction(true)
        .with_strict(false);
    let dto = spec.to_dto();
    assert_eq!(dto.id.0, "id");
    assert_eq!(dto.display_name, "Display");
    assert_eq!(dto.aliases, vec!["a"]);
    assert!(
        dto.read_only && dto.destructive && dto.concurrency_safe && dto.requires_user_interaction
    );
    assert!(!dto.strict);

    let hint = ToolRenderHint::titled("Title");
    assert_eq!(hint.to_dto().title.as_deref(), Some("Title"));

    let ok = ToolCallResult::from_legacy_result(ToolResult {
        success: true,
        output: "ok".into(),
        error: None,
    });
    assert!(ok.is_success());
    assert_eq!(ok.model_text(), "ok");
    assert!(ok.error_text().is_none());

    let bad = ToolCallResult::from_legacy_result(ToolResult {
        success: false,
        output: "out".into(),
        error: Some("bad".into()),
    });
    assert!(!bad.is_success());
    assert_eq!(bad.default_model_result(), json!("bad"));
    assert_eq!(bad.error_text().as_deref(), Some("bad"));
}

#[tokio::test]
async fn default_trait_call_maps_legacy_failure_and_default_flags() {
    let tool = FailingTool;
    let result = tool.call(json!({})).await.unwrap();
    assert!(!result.is_success());
    assert_eq!(result.model_result, json!("err"));
    assert!(tool.spec().destructive);

    assert_eq!(default_tool_aliases("web_fetch"), vec!["webfetch".to_string()]);
    assert!(default_tool_read_only("web_search_tool"));
    assert!(default_tool_destructive("todowrite"));
    assert!(default_tool_concurrency_safe("file_read"));
    assert!(default_tool_requires_user_interaction("question"));
}
