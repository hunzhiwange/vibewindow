use super::*;
use crate::app::agent::runtime::{NativeRuntime, RuntimeAdapter};
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::context::ToolUseContext;
use crate::app::agent::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

struct EchoTool;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "echo tool"
    }
    fn parameters_schema(&self) -> Value {
        json!({"type": "object"})
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: args.to_string(), error: None })
    }
}

#[test]
fn runtime_context_and_scalar_parsing_cover_core_branches() {
    let ctx = ToolRuntimeContext::new("s", Some("/tmp/root".to_string()))
        .with_tool_use_context(ToolUseContext::new("inner", None));
    assert_eq!(ctx.session, "s");
    assert_eq!(ctx.tool_use_context().session(), "inner");
    assert_eq!(ctx.tool_use_context().root(), Some("/tmp/root"));

    assert_eq!(parse_tool_input("", "").unwrap(), json!({}));
    assert_eq!(parse_tool_input("file_read", "a.txt").unwrap(), json!({"path": "a.txt"}));
    assert_eq!(
        parse_tool_input("web_fetch", "https://e.test").unwrap(),
        json!({"url": "https://e.test"})
    );
    assert!(parse_tool_input("unknown", "plain").is_err());
    assert!(parse_tool_input("unknown", "{bad").unwrap_err().to_string().contains("invalid JSON"));
}

#[test]
fn classify_message_and_binary_detection_cover_errors_and_content() {
    assert!(matches!(classify_message("blocked".to_string()), ToolCallError::Denied { .. }));
    assert!(matches!(classify_message("failed".to_string()), ToolCallError::Failed(_)));

    let dir = tempfile::tempdir().unwrap();
    let text = dir.path().join("a.txt");
    let bin = dir.path().join("a.bin");
    let nul = dir.path().join("a.data");
    std::fs::write(&text, b"hello\n").unwrap();
    std::fs::write(&bin, b"text").unwrap();
    std::fs::write(&nul, b"a\0b").unwrap();
    assert!(!is_binary(&text));
    assert!(is_binary(&bin));
    assert!(is_binary(&nul));
    assert!(!is_binary(&dir.path().join("missing")));
}

#[tokio::test]
async fn arc_delegating_tool_forwards_methods() {
    let boxed = ArcDelegatingTool::boxed(Arc::new(EchoTool));
    assert_eq!(boxed.name(), "echo");
    assert_eq!(boxed.parameters_schema(), json!({"type": "object"}));
    assert_eq!(boxed.validate_input(json!({"x": 1})).unwrap(), json!({"x": 1}));
    assert!(boxed.execute(json!({"x": 1})).await.unwrap().success);
}

#[test]
fn default_tools_include_native_capability_tools() {
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(NativeRuntime::new());
    let tools = default_tools_with_runtime(Arc::new(SecurityPolicy::default()), runtime);
    let names = tools.iter().map(|tool| tool.name()).collect::<Vec<_>>();

    assert!(names.contains(&"file_read"));
    assert!(names.contains(&"batch"));
}
