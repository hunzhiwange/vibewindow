use super::*;

#[test]
fn completion_detection_requires_completion_action_and_object() {
    assert!(looks_like_unverified_action_completion_without_tool_call("Done, I updated the file."));
    assert!(!looks_like_unverified_action_completion_without_tool_call("Done."));
    assert!(!looks_like_unverified_action_completion_without_tool_call("I will update the file."));
    assert!(!looks_like_unverified_action_completion_without_tool_call(""));
}
