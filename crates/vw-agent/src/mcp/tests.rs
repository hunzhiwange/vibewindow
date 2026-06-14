use super::*;
use crate::tools::{self, ToolRuntimeContext};
use rmcp::handler::server::ServerHandler;
use rmcp::model::RawContent;
use serde_json::json;
use std::sync::Arc;

#[test]
fn server_can_be_constructed_with_runtime_context() {
    let _server = AgentToolServer::new(ToolRuntimeContext::for_specs());
}

#[test]
fn server_info_exposes_stable_identity() {
    let server = AgentToolServer::new(ToolRuntimeContext::for_specs());
    let info = ServerHandler::get_info(&server);

    assert_eq!(info.server_info.name, "vibe-window");
    assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(info.instructions.as_deref(), Some("Expose vibe-window agent tools over MCP."));
}

#[test]
fn cloned_server_shares_runtime_context() {
    let server = AgentToolServer::new(ToolRuntimeContext::for_specs());
    assert_eq!(Arc::strong_count(&server.ctx), 1);

    let cloned = server.clone();

    assert_eq!(Arc::strong_count(&server.ctx), 2);
    assert!(Arc::ptr_eq(&server.ctx, &cloned.ctx));
}

#[test]
fn registry_specs_are_mapped_to_mcp_tools() {
    let tools = mcp_tools_from_registry();

    assert!(!tools.is_empty());
    assert!(tools.iter().all(|tool| !tool.name.trim().is_empty()));
    assert!(tools.iter().all(|tool| tool.description.as_ref().is_some_and(|d| !d.is_empty())));
    assert!(tools.iter().all(|tool| !tool.input_schema.contains_key("$invalid")));
}

#[test]
fn serialize_tool_arguments_defaults_to_empty_object() {
    assert_eq!(serialize_tool_arguments(None), "{}");
    assert_eq!(serialize_tool_arguments(Some(Default::default())), "{}");
}

#[test]
fn serialize_tool_arguments_preserves_json_object_arguments() {
    let mut args = serde_json::Map::new();
    args.insert("path".to_string(), json!("/tmp/example.txt"));
    args.insert("limit".to_string(), json!(3));

    let encoded = serialize_tool_arguments(Some(args));
    let decoded: serde_json::Value = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded["path"], "/tmp/example.txt");
    assert_eq!(decoded["limit"], 3);
}

fn first_text(result: &CallToolResult) -> &str {
    match &result.content[0].raw {
        RawContent::Text(text) => &text.text,
        other => panic!("expected text content, got {other:?}"),
    }
}

#[test]
fn successful_tool_execution_maps_to_successful_mcp_result() {
    let tool_result = tools::ToolCallResult {
        model_result: json!("done"),
        telemetry: Some(tools::ToolCallTelemetry { success: true, ..Default::default() }),
        ..Default::default()
    };

    let result = mcp_call_tool_result(Ok(tool_result));

    assert_eq!(result.is_error, Some(false));
    assert_eq!(first_text(&result), "done");
}

#[test]
fn failed_tool_execution_maps_to_mcp_error_result() {
    let result = mcp_call_tool_result(Err(tools::ToolCallError::Failed("boom".into())));

    assert_eq!(result.is_error, Some(true));
    assert!(first_text(&result).contains("Failed(\"boom\")"));
}
