use super::diff_utils::{
    count_apply_patch_format_changes, count_unified_diff_changes, extract_diff_block,
    is_likely_file_path, looks_like_unified_diff, parse_apply_patch_line_changes,
    parse_apply_patch_summary, string_or_string_array,
};
use serde_json::json;

#[test]
fn unified_diff_detection_and_counts_ignore_headers() {
    let diff = "--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-old\n+new\n";

    assert!(looks_like_unified_diff(diff));
    assert_eq!(count_unified_diff_changes(diff), (1, 1));
}

#[test]
fn apply_patch_summary_and_counts_parse_file_lines() {
    let summary_output = "M src/lib.rs\n";
    let count_output = "M src/lib.rs +1 -1\n";

    assert_eq!(parse_apply_patch_summary(summary_output), vec![('M', "src/lib.rs".to_string())]);
    assert_eq!(parse_apply_patch_line_changes(count_output), (1, 1));
    assert_eq!(
        count_apply_patch_format_changes("*** Update File: src/lib.rs\n+new\n-old\n"),
        (1, 1)
    );
}

#[test]
fn diff_helpers_extract_code_blocks_and_paths() {
    assert_eq!(extract_diff_block("<diff>\n+new\n</diff>"), Some("+new".to_string()));
    assert!(is_likely_file_path("src/main.rs"));
    assert_eq!(string_or_string_array(&json!(["a", "b"])), Some("a\nb".to_string()));
}
