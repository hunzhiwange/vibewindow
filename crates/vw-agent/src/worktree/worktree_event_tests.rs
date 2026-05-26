use super::event::{FAILED, READY};

#[test]
fn event_types_are_stable() {
    assert_eq!(READY.r#type, "worktree.ready");
    assert_eq!(FAILED.r#type, "worktree.failed");
}
