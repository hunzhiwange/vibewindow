use super::*;

#[test]
fn default_workflow_state_is_inactive_and_unbound() {
    let state = WorkflowState::default();
    assert!(!state.plan_mode.active);
    assert!(state.plan_mode.goal.is_none());
    assert!(state.worktree.directory.is_none());
}

#[test]
fn context_preserves_session_and_root() {
    let context = ToolUseContext::new("session-a", Some("/tmp/work".into()));
    assert_eq!(context.session(), "session-a");
    assert_eq!(context.root(), Some("/tmp/work"));
    assert_eq!(context.root_path().as_deref(), Some(std::path::Path::new("/tmp/work")));
}
