use super::{
    compute_visible_message_window_for_chat, is_chat_message_idx_visible,
    live_message_meta_fallback, rough_message_heights, user_question_indices,
};
use crate::app::models::{ChatMessage, ChatRole};

fn message(role: ChatRole, content: &str) -> ChatMessage {
    ChatMessage { role, content: content.to_string(), think_timing: Vec::new() }
}

#[test]
fn visible_message_window_clamps_to_available_heights() {
    let heights = vec![40.0, 50.0, 60.0];

    let (start, end) = compute_visible_message_window_for_chat(10, &heights, 0.0, 70.0);

    assert_eq!(start, 0);
    assert!(end <= heights.len());
    assert!(end > start);
}

#[test]
fn visible_message_window_returns_empty_without_messages_or_heights() {
    assert_eq!(compute_visible_message_window_for_chat(0, &[40.0], 0.0, 100.0), (0, 0));
    assert_eq!(compute_visible_message_window_for_chat(2, &[], 0.0, 100.0), (0, 0));
}

#[test]
fn visible_message_idx_requires_positive_non_empty_window() {
    assert!(is_chat_message_idx_visible(1, 0, 2, 200.0));
    assert!(!is_chat_message_idx_visible(2, 0, 2, 200.0));
    assert!(!is_chat_message_idx_visible(1, 0, 2, 0.0));
    assert!(!is_chat_message_idx_visible(1, 2, 2, 200.0));
}

#[test]
fn user_question_indices_only_returns_user_roles() {
    let chat = vec![
        message(ChatRole::System, "system"),
        message(ChatRole::User, "first"),
        message(ChatRole::Assistant, "answer"),
        message(ChatRole::User, "second"),
    ];

    assert_eq!(user_question_indices(&chat), vec![1, 3]);
}

#[test]
fn rough_message_heights_grow_for_rich_messages() {
    let chat = vec![
        message(ChatRole::User, "short"),
        message(ChatRole::Assistant, "<think>x</think>\ntool bash\n{}\nanswer"),
    ];

    let heights = rough_message_heights(&chat);

    assert_eq!(heights.len(), 2);
    assert!(heights[1] > heights[0]);
}

#[test]
fn live_message_meta_fallback_applies_to_chat_roles_only() {
    assert_eq!(
        live_message_meta_fallback(ChatRole::Assistant, false, false, "gpt"),
        Some("gpt · 刚刚".to_string())
    );
    assert_eq!(
        live_message_meta_fallback(ChatRole::User, false, false, "gpt"),
        Some("gpt · 刚刚".to_string())
    );
    assert_eq!(live_message_meta_fallback(ChatRole::System, true, true, "gpt"), None);
}
