use super::*;

#[test]
fn message_includes_goal_and_instruction_lines() {
    let message = enter_plan_mode_message(false);
    assert!(message.contains("Entered plan mode"));
    assert!(enter_plan_mode_instruction_lines().iter().any(|line| line.contains("codebase")));
}

#[test]
fn result_text_is_stable_for_empty_goal() {
    let text = enter_plan_mode_result_text(true);
    assert!(text.contains("already active"));
    assert!(text.contains("DO NOT write"));
}
