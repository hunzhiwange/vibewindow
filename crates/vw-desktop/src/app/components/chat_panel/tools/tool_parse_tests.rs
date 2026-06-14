use super::tool_parse::{
    explore_item_dedupe_key, explore_tool_kind, is_explore_tool, should_hide_tool_block,
    tool_call_id_from_raw, tool_change_file_summaries, tool_error_text, tool_identity_from_raw,
    tool_input, tool_input_path, tool_name_from_raw, tool_output_path, tool_output_text,
    tool_status, tool_status_from_raw, tool_structured_diff_text, tool_summary_text,
};
use crate::app::App;
use crate::app::components::chat_panel::tools::types::ChangeFileSummary;

#[test]
fn tool_name_and_call_id_are_read_from_raw_tool_block() {
    let raw = r#"tool bash
{"call_id":"call-1","status":"completed","input":"echo ok"}"#;

    assert_eq!(tool_name_from_raw(raw), Some("bash".to_string()));
    assert_eq!(tool_call_id_from_raw(raw), Some("call-1".to_string()));
    assert_eq!(tool_status_from_raw(raw), Some("completed".to_string()));
}

#[test]
fn explore_tool_kind_classifies_read_like_tools() {
    assert!(is_explore_tool("read"));
    assert!(explore_tool_kind("read").is_some());
    assert!(
        explore_item_dedupe_key(
            r#"tool read
{"input":"{\"filePath\":\"src/main.rs\"}"}"#
        )
        .is_some()
    );
}

#[test]
fn tool_input_accepts_raw_input_alias() {
    let value = serde_json::json!({
        "rawInput": "{\"command\":\"date\"}"
    });

    assert_eq!(tool_input(&value), "{\"command\":\"date\"}");
}

#[test]
fn tool_input_path_accepts_claude_file_path_alias() {
    assert_eq!(
        tool_input_path(r#"{"file_path":"docs/demo.md"}"#),
        Some("docs/demo.md".to_string())
    );
}

#[test]
fn tool_name_prefers_canonical_id_from_render_hint() {
    let raw = r#"tool Read
{"renderHint":{"metadata":{"canonical_tool_id":"file_read"}}}"#;

    assert_eq!(tool_name_from_raw(raw), Some("read".to_string()));
}

#[test]
fn tool_call_id_and_summary_are_found_in_nested_locations() {
    let raw = r#"tool read
{"result":{"metadata":{"toolUseId":"call-2"}},"render_hint":{"summary":"peek file"}}"#;

    assert_eq!(tool_call_id_from_raw(raw), Some("call-2".to_string()));
    assert_eq!(tool_status_from_raw(raw), Some("".to_string()));

    let value = serde_json::json!({
        "render_hint": {"summary": "peek file"},
        "result": {"success": true}
    });
    assert_eq!(tool_summary_text(&value).as_deref(), Some("peek file"));
    assert_eq!(tool_status(&value), "completed");
}

#[test]
fn tool_output_helpers_fall_back_to_structured_result_content() {
    let value = serde_json::json!({
        "result": {
            "content": [
                {
                    "type": "structured_patch",
                    "hunks": [
                        {
                            "path": "src/main.rs",
                            "header": "@@ -1 +1 @@",
                            "lines": ["-old", "+new"]
                        }
                    ]
                }
            ]
        }
    });

    let diff = tool_structured_diff_text(&value).expect("diff text");
    assert!(diff.contains("--- a/src/main.rs"));
    assert!(tool_output_text(&value).expect("output").contains("+new"));
    assert_eq!(
        tool_change_file_summaries(&value),
        vec![ChangeFileSummary {
            kind: 'M',
            path: "src/main.rs".to_string(),
            additions: 1,
            deletions: 1,
        }]
    );
}

#[test]
fn tool_identity_and_hide_rules_cover_special_cases() {
    let bash_raw = r#"tool bash
{"input":"{\"command\":\"cargo test\"}"}"#;
    let grep_raw = r#"tool grep
{"input":"{\"pattern\":\"main\",\"include\":\"*.rs\",\"path\":\"src\"}"}"#;
    let todo_raw = r#"tool todowrite
{"status":"completed"}"#;

    assert_eq!(tool_identity_from_raw(bash_raw).as_deref(), Some("bash:cargo test"));
    assert_eq!(tool_identity_from_raw(grep_raw).as_deref(), Some("grep:main|*.rs|src"));
    assert!(should_hide_tool_block(todo_raw));
}

#[test]
fn output_path_and_error_text_handle_aliases() {
    let value = serde_json::json!({
        "status": "error",
        "output": "permission denied",
        "renderHint": { "metadata": { "outputPath": "[main](/tmp/out.rs#L7)" } }
    });

    assert_eq!(tool_output_path(&value).as_deref(), Some("/tmp/out.rs"));
    assert_eq!(tool_error_text(&value).as_deref(), Some("permission denied"));
}

#[test]
fn resolve_output_path_joins_project_root_for_relative_values() {
    let mut app = App::new().0;
    app.project_path = Some("/tmp/demo".to_string());

    assert_eq!(super::tool_parse::resolve_output_path(&app, "file:///tmp/out.rs"), "tmp/out.rs");
    assert_eq!(
        super::tool_parse::resolve_output_path(&app, "logs/out.txt"),
        "/tmp/demo/logs/out.txt"
    );
}
