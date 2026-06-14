use super::*;
use crate::app::agent::tools::{ToolResult, ToolSpec};

struct TestTool {
    id: &'static str,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Tool for TestTool {
    fn name(&self) -> &str {
        self.id
    }

    fn description(&self) -> &str {
        "test tool"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object"})
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: String::new(), error: None })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.id, self.description(), self.parameters_schema())
    }
}

#[test]
fn filtered_tool_specs_for_runtime_returns_all_specs_when_no_exclusions() {
    let tools: Vec<Box<dyn Tool>> =
        vec![Box::new(TestTool { id: "shell" }), Box::new(TestTool { id: "file_read" })];

    let specs = filtered_tool_specs_for_runtime(&tools, &[]);

    assert_eq!(
        specs.iter().map(|spec| spec.id.as_str()).collect::<Vec<_>>(),
        ["shell", "file_read"]
    );
}

#[test]
fn filtered_tool_specs_for_runtime_excludes_exact_id_matches_only() {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(TestTool { id: "shell" }),
        Box::new(TestTool { id: "shell_extra" }),
        Box::new(TestTool { id: "memory_recall" }),
    ];
    let excluded = vec!["shell".to_string(), "missing".to_string()];

    let specs = filtered_tool_specs_for_runtime(&tools, &excluded);

    assert_eq!(
        specs.iter().map(|spec| spec.id.as_str()).collect::<Vec<_>>(),
        ["shell_extra", "memory_recall"]
    );
}

#[tokio::test]
async fn create_resilient_provider_nonblocking_returns_error_for_unknown_provider() {
    let err = match create_resilient_provider_nonblocking(
        "__definitely_missing_provider__",
        None,
        None,
        crate::app::agent::config::ReliabilityConfig::default(),
        crate::app::agent::providers::ProviderRuntimeOptions::default(),
    )
    .await
    {
        Ok(_) => panic!("unknown provider should fail"),
        Err(error) => error,
    };

    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn create_routed_provider_nonblocking_returns_error_for_unknown_provider() {
    let err = match create_routed_provider_nonblocking(
        "__definitely_missing_provider__",
        None,
        None,
        crate::app::agent::config::ReliabilityConfig::default(),
        Vec::new(),
        "model".to_string(),
        crate::app::agent::providers::ProviderRuntimeOptions::default(),
    )
    .await
    {
        Ok(_) => panic!("unknown routed provider should fail"),
        Err(error) => error,
    };

    assert!(!err.to_string().is_empty());
}
