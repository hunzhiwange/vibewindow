use super::*;
use crate::types::PermissionMode;

#[test]
fn permission_mode_satisfies_uses_rank_order() {
    assert!(permission_mode_satisfies(PermissionMode::ApproveAll, PermissionMode::DenyAll));
    assert!(permission_mode_satisfies(PermissionMode::ApproveReads, PermissionMode::ApproveReads));
    assert!(!permission_mode_satisfies(PermissionMode::ApproveReads, PermissionMode::ApproveAll));
}

#[test]
fn infer_tool_kind_from_title_uses_command_head() {
    assert_eq!(infer_tool_kind_from_title("Read: file"), Some("read"));
    assert_eq!(infer_tool_kind_from_title("grep: files"), Some("search"));
    assert_eq!(infer_tool_kind_from_title("edit: file"), Some("edit"));
    assert_eq!(infer_tool_kind_from_title("unknown: thing"), Some("other"));
    assert_eq!(infer_tool_kind_from_title("   "), None);
}

#[test]
fn read_and_search_kinds_are_auto_approved() {
    assert!(is_auto_approved_read_kind(Some("read")));
    assert!(is_auto_approved_read_kind(Some("search")));
    assert!(!is_auto_approved_read_kind(Some("edit")));
    assert!(!is_auto_approved_read_kind(None));
}
