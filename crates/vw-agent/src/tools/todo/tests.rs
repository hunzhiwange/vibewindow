use super::*;
use crate::app::agent::tools::traits::Tool;

#[test]
fn read_tool_spec_is_read_only_and_strict() {
    let tool = TodoReadTool::new("session".to_string());
    let spec = tool.spec();

    assert_eq!(tool.name(), "todoread");
    assert!(spec.read_only);
    assert!(spec.strict);
}

#[test]
fn write_rejects_empty_input() {
    let ctx = crate::app::agent::tools::ToolRuntimeContext::new("todo-empty", None);

    assert!(write("   ", &ctx).is_err());
}
