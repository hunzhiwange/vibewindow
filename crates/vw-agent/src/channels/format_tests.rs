use super::*;

#[test]
fn strip_tool_call_tags_removes_closed_tool_blocks() {
    let message = "before <tool>{\"name\":\"secret\"}</tool> after";

    assert_eq!(strip_tool_call_tags(message), "before  after");
}

#[test]
fn split_internal_progress_delta_detects_known_prefix() {
    let delta = format!("{}working", crate::app::agent::agent::loop_::DRAFT_PROGRESS_SENTINEL);
    let (is_progress, content) = split_internal_progress_delta(&delta);

    assert!(is_progress);
    assert_eq!(content, "working");
}

#[test]
fn channel_delivery_instructions_are_known_for_chat_channels() {
    assert!(channel_delivery_instructions("telegram").is_some());
    assert!(channel_delivery_instructions("unknown").is_none());
}
