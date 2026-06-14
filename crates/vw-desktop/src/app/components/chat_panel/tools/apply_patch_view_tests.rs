use super::apply_patch_view::{
    apply_patch_change_totals, apply_patch_header_summary, diff_utils_looks_like_unified_diff,
    tool_apply_patch_view,
};
use crate::app::App;

#[test]
fn apply_patch_header_summary_splits_first_path() {
    let files = vec![('M', "src/main.rs".to_string()), ('A', "README.md".to_string())];

    assert_eq!(
        apply_patch_header_summary(&files),
        ("main.rs".to_string(), "/src".to_string(), Some("等 2 个文件".to_string()))
    );
}

#[test]
fn apply_patch_header_summary_handles_empty_changes() {
    assert_eq!(apply_patch_header_summary(&[]), (String::new(), String::new(), None));
}

#[test]
fn apply_patch_helpers_accept_change_totals_and_unified_diff() {
    let _ = apply_patch_change_totals(2, 1);
    assert!(diff_utils_looks_like_unified_diff("--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@"));
    assert!(!diff_utils_looks_like_unified_diff("--- a/a.rs\n+++ b/a.rs\n"));
}

#[test]
fn apply_patch_view_rejects_bad_tool_and_invalid_json() {
    let app = App::new().0;

    assert!(tool_apply_patch_view(&app, 0, 0, "tool bash\n{}").is_none());
    assert!(tool_apply_patch_view(&app, 0, 0, "tool apply_patch\nnot-json").is_none());
}

#[test]
fn apply_patch_view_renders_running_error_and_file_summary_states() {
    let mut app = App::new().0;
    app.chat_tool_hovered_idx = Some((1_u64 << 32) | 1);
    app.chat_tool_file_expanded.insert("1:1:src/main.rs".to_string());

    assert!(
        tool_apply_patch_view(
            &app,
            1,
            1,
            r#"tool apply_patch
{"status":"running","input":"*** Begin Patch\n*** End Patch"}"#
        )
        .is_some()
    );
    assert!(
        tool_apply_patch_view(
            &app,
            1,
            1,
            r#"tool apply_patch
{"status":"error","error":"patch failed","input":"*** Begin Patch\n*** End Patch"}"#
        )
        .is_some()
    );
    assert!(
        tool_apply_patch_view(
            &app,
            1,
            1,
            concat!(
                "tool apply_patch\n",
                "{\"input\":\"*** Begin Patch\\n*** Update File: src/main.rs\\n-old\\n+new\\n*** End Patch\",",
                "\"output\":\"M src/main.rs +1 -1\\n\\n<changes>\\n",
                "{\\\"files\\\":[{\\\"path\\\":\\\"src/main.rs\\\",\\\"additions\\\":1,\\\"deletions\\\":1,\\\"before\\\":\\\"old\\\\n\\\",\\\"after\\\":\\\"new\\\\n\\\"}]}",
                "\\n</changes>\"}"
            )
        )
        .is_some()
    );
}
