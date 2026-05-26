use super::apply_patch_view::{apply_patch_change_totals, apply_patch_header_summary, diff_utils_looks_like_unified_diff};

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
}
