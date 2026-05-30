#[test]
fn tools_exec_tests_module_is_wired() {
    let marker = String::from("tools_exec_tests");
    assert_eq!(marker.as_str(), "tools_exec_tests");
}

#[test]
fn completed_tool_payload_for_ui_keeps_structured_patch_content() {
    use crate::app::agent::tools::{ToolCallResult, ToolCallTelemetry};
    use serde_json::Value;
    use vw_api_types::tools::{StructuredPatchHunkDto, ToolResultContentDto};

    let result = ToolCallResult {
        content_blocks: vec![ToolResultContentDto::StructuredPatch {
            hunks: vec![StructuredPatchHunkDto {
                header: "@@ -1 +1 @@".to_string(),
                path: Some("docs/tailwind/align-self.mdx".to_string()),
                old_start: Some(1),
                old_lines: Some(1),
                new_start: Some(1),
                new_lines: Some(1),
                lines: vec!["+full replacement line".to_string()],
            }],
        }],
        telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
        ..ToolCallResult::default()
    };

    let payload = super::completed_tool_payload_for_ui(
        "file_edit",
        r#"{"file_path":"docs/tailwind/align-self.mdx"}"#,
        &result,
        "Updated docs/tailwind/align-self.mdx.",
    );

    let content = payload
        .get("result")
        .and_then(|value| value.get("content"))
        .and_then(Value::as_array)
        .expect("structured content should be exposed to UI");

    assert_eq!(content[0].get("type").and_then(Value::as_str), Some("structured_patch"));
    assert_eq!(
        content[0]
            .get("hunks")
            .and_then(Value::as_array)
            .and_then(|hunks| hunks[0].get("lines"))
            .and_then(Value::as_array)
            .and_then(|lines| lines[0].as_str()),
        Some("+full replacement line")
    );
}
