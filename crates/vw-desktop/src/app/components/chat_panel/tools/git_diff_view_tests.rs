use super::git_diff_view::{
    append_preview_gap, append_preview_line, is_git_diff_tool, parse_git_diff_previews,
    structured_git_diff_output, tool_git_diff_view,
};
use crate::app::{App, Message};
use serde_json::json;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn append_preview_helpers_preserve_line_boundaries() {
    let mut buf = String::new();
    append_preview_line(&mut buf, "first");
    append_preview_gap(&mut buf);
    append_preview_line(&mut buf, "last");

    assert!(buf.contains("first"));
    assert!(buf.contains("last"));
}

#[test]
fn git_operations_diff_is_treated_as_git_diff_tool() {
    assert!(is_git_diff_tool("git_diff", ""));
    assert!(is_git_diff_tool("git_operations", r#"{"operation":"diff"}"#));
    assert!(!is_git_diff_tool("git_operations", r#"{"operation":"status"}"#));
}

#[test]
fn parse_git_diff_previews_returns_none_without_diff_data() {
    assert!(parse_git_diff_previews("", &json!({})).is_none());
}

#[test]
fn structured_git_diff_output_reads_data_or_json_content_blocks() {
    let direct = json!({"result":{"data":{"hunks":[]}}});
    assert_eq!(structured_git_diff_output(&direct), Some(json!({"hunks":[]})));

    let content = json!({
        "result": {
            "content": [
                {"type":"text","text":"ignored"},
                {"type":"json","value":{"hunks":[]}}
            ]
        }
    });
    assert_eq!(structured_git_diff_output(&content), Some(json!({"hunks":[]})));
}

#[test]
fn parse_git_diff_previews_groups_hunks_and_counts_changes() {
    let input = r#"{"cached":true}"#;
    let value = json!({
        "result": {
            "data": {
                "hunks": [
                    {
                        "file": "src/main.rs",
                        "header": "@@ -1 +1 @@",
                        "lines": [
                            {"text":"diff --git a/src/main.rs b/src/main.rs"},
                            {"text":"--- a/src/main.rs"},
                            {"text":"+++ b/src/main.rs"},
                            {"text":"-old"},
                            {"text":"+new"},
                            {"text":" context"}
                        ]
                    },
                    {
                        "file": "src/main.rs",
                        "lines": [{"text":"+again"}]
                    },
                    {
                        "file": "",
                        "lines": [{"text":"+skip"}]
                    }
                ]
            }
        }
    });

    let previews = parse_git_diff_previews(input, &value).expect("diff previews");

    assert_eq!(previews.len(), 1);
    assert_eq!(previews[0].path, "src/main.rs");
    assert_eq!(previews[0].additions, 2);
    assert_eq!(previews[0].deletions, 1);
    assert!(previews[0].cached);
    assert!(previews[0].before.contains("old"));
    assert!(previews[0].after.contains("again"));
}

#[test]
fn parse_git_diff_previews_accepts_output_json_string() {
    let value = json!({
        "output": "{\"hunks\":[{\"file\":\"src/lib.rs\",\"lines\":[{\"text\":\"+pub fn lib() {}\"}]}]}"
    });

    let previews = parse_git_diff_previews("{}", &value).expect("output json previews");

    assert_eq!(previews[0].path, "src/lib.rs");
    assert_eq!(previews[0].additions, 1);
}

#[test]
fn tool_git_diff_view_rejects_non_diff_or_empty_completed_output() {
    let app = test_app();

    assert!(tool_git_diff_view(&app, 0, 0, "tool read\n{}").is_none());
    assert!(
        tool_git_diff_view(
            &app,
            0,
            0,
            r#"tool git_diff
{"input":"{}","status":"completed"}"#
        )
        .is_none()
    );
}

#[test]
fn tool_git_diff_view_builds_success_running_and_error_cards() {
    let app = test_app();

    let success = tool_git_diff_view(&app, 1, 1, r#"tool git_diff
{"input":"{}","status":"completed","result":{"data":{"hunks":[{"file":"src/main.rs","lines":[{"text":"+new"}]}]}}}"#)
    .expect("success git diff view");
    keep_element(success);

    let running = tool_git_diff_view(
        &app,
        1,
        2,
        r#"tool git_diff
{"input":"{}","status":"running"}"#,
    )
    .expect("running git diff view");
    keep_element(running);

    let error = tool_git_diff_view(
        &app,
        1,
        3,
        r#"tool git_diff
{"input":"{}","status":"error","error":"diff failed"}"#,
    )
    .expect("error git diff view");
    keep_element(error);
}
