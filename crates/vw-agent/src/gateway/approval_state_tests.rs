use super::*;

#[test]
fn clear_approval_manager_is_idempotent_for_unknown_directory() {
    clear_approval_manager_for_directory("/tmp/not-present");
    clear_approval_manager_for_directory("/tmp/not-present");
}
