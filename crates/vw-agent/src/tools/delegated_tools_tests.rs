use super::*;

struct NamedTool {
    name: &'static str,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for NamedTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "test tool"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        })
    }

    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: self.name.to_string(), error: None })
    }
}

#[test]
fn build_agentic_tools_blocks_batch_even_when_allowlisted() {
    let parent_tools: Vec<Arc<dyn Tool>> = vec![Arc::new(NamedTool { name: "batch" })];
    let tools = build_agentic_tools(&parent_tools, &["batch".to_string()], &[]);

    assert!(tools.is_empty());
}

#[test]
fn build_agentic_tools_matches_tool_aliases() {
    let parent_tools: Vec<Arc<dyn Tool>> = vec![Arc::new(NamedTool { name: "file_write" })];
    let tools = build_agentic_tools(&parent_tools, &["write".to_string()], &[]);

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name(), "file_write");
}

#[tokio::test]
async fn delegated_skill_tool_enforces_allowed_skills() {
    let parent_tools: Vec<Arc<dyn Tool>> = vec![Arc::new(NamedTool { name: "skill" })];
    let tools =
        build_agentic_tools(&parent_tools, &["skill".to_string()], &["translate".to_string()]);

    assert_eq!(tools.len(), 1);

    let denied = tools[0].execute(json!({ "name": "other" })).await.unwrap();
    assert!(!denied.success);
    assert!(denied.error.as_deref().unwrap_or("").contains("not allowed"));

    let allowed = tools[0].execute(json!({ "name": "translate" })).await.unwrap();
    assert!(allowed.success);
}

#[test]
fn skill_tool_without_allowed_skills_is_not_exposed() {
    let parent_tools: Vec<Arc<dyn Tool>> = vec![Arc::new(NamedTool { name: "skill" })];
    let tools = build_agentic_tools(&parent_tools, &["skill".to_string()], &[]);

    assert!(tools.is_empty());
}
