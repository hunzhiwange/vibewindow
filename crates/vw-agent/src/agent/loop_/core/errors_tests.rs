use super::*;

#[test]
fn detects_cancelled_and_iteration_limit_errors() {
    let cancelled = anyhow::Error::new(ToolLoopCancelled);
    assert!(is_tool_loop_cancelled(&cancelled));

    let limit = anyhow::anyhow!("Agent exceeded maximum tool iterations");
    assert!(is_tool_iteration_limit_error(&limit));

    let other = anyhow::anyhow!("different failure");
    assert!(!is_tool_loop_cancelled(&other));
    assert!(!is_tool_iteration_limit_error(&other));
}
