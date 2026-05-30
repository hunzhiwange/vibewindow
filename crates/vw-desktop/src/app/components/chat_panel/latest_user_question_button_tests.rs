//! 验证最新用户问题按钮行为。
//! 测试确保按钮只在合适状态出现并指向正确消息。

use super::{is_chat_message_idx_visible, user_question_indices};

#[test]
fn chat_message_idx_visible_returns_true_inside_window() {
    assert!(is_chat_message_idx_visible(5, 4, 7, 420.0));
}

#[test]
fn chat_message_idx_visible_returns_false_outside_window() {
    assert!(!is_chat_message_idx_visible(3, 4, 7, 420.0));
}

#[test]
fn chat_message_idx_visible_returns_false_without_viewport() {
    assert!(!is_chat_message_idx_visible(3, 0, 4, 0.0));
    assert!(!is_chat_message_idx_visible(3, 4, 4, 420.0));
}

#[test]
fn user_question_indices_preserve_question_order() {
    let chat = vec![
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::User,
            content: "问题一".to_string(),
            think_timing: Vec::new(),
        },
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::Assistant,
            content: "回答一".to_string(),
            think_timing: Vec::new(),
        },
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::User,
            content: "问题二".to_string(),
            think_timing: Vec::new(),
        },
    ];

    assert_eq!(user_question_indices(&chat), vec![0, 2]);
}
