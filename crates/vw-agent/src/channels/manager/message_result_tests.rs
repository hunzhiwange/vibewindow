use super::*;

#[test]
fn reaction_done_emoji_marks_success_only_for_completed_ok_result() {
    assert_eq!(reaction_done_emoji(&LlmExecutionResult::Completed(Ok(Ok("done".to_string())))), "\u{2705}");
    assert_eq!(reaction_done_emoji(&LlmExecutionResult::Completed(Ok(Err(anyhow::anyhow!("failed"))))), "\u{26A0}\u{FE0F}");
    assert_eq!(reaction_done_emoji(&LlmExecutionResult::Cancelled), "\u{26A0}\u{FE0F}");
}
