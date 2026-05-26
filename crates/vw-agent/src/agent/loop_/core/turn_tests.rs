use super::*;

#[test]
fn reply_target_task_local_defaults_to_unset_outside_scope() {
    assert!(TOOL_LOOP_REPLY_TARGET.try_with(|target| target.clone()).is_err());
}
