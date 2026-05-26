//! 覆盖 JSON 差异工具的消息处理行为，保护格式化和差异状态更新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::*;

#[test]
fn prettify_json_text_formats_pretty_output() {
    let formatted = prettify_json_text(r#"{"name":"alice","roles":["admin","dev"]}"#)
        .expect("json should format");

    assert_eq!(
        formatted,
        "{\n  \"name\": \"alice\",\n  \"roles\": [\n    \"admin\",\n    \"dev\"\n  ]\n}"
    );
}

#[test]
fn compare_json_documents_marks_changed_and_missing_fields() {
    let diffs = compare_json_documents(
        r#"{"name":"","enabled":true,"profile":{"city":"Shanghai"}}"#,
        r#"{"enabled":false,"profile":{"street":"Huaihai Rd"}}"#,
    )
    .expect("json should compare");

    assert_eq!(
        diffs,
        vec![
            JsonDiffEntry {
                path: "enabled".to_string(),
                left: Some("true".to_string()),
                right: Some("false".to_string()),
            },
            JsonDiffEntry { path: "name".to_string(), left: Some(String::new()), right: None },
            JsonDiffEntry {
                path: "profile.city".to_string(),
                left: Some("Shanghai".to_string()),
                right: None,
            },
            JsonDiffEntry {
                path: "profile.street".to_string(),
                left: None,
                right: Some("Huaihai Rd".to_string()),
            },
        ]
    );
}

#[test]
fn compare_json_documents_reports_side_specific_parse_error() {
    let error = compare_json_documents("{", r#"{"name":"ok"}"#).expect_err("left side should fail");

    assert!(error.contains("左侧 JSON 解析失败"));
}
