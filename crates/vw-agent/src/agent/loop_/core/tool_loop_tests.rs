use super::*;

#[test]
fn completion_detection_is_reexported_for_tool_loop() {
    assert!(looks_like_unverified_action_completion_without_tool_call(
        "I have created the file successfully"
    ));
}
