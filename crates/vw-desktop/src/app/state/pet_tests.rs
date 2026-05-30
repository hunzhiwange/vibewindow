use super::{current_open_think_body, newest_open_thinking_text, task_pet_title, truncate_for_pet};
use crate::app::models::{ChatMessage, ChatRole};

#[test]
fn task_pet_title_uses_first_non_empty_line() {
    let title = task_pet_title("\n\n  修复网关重启配置丢失\n后续内容");

    assert_eq!(title, "修复网关重启配置丢失");
}

#[test]
fn truncate_for_pet_marks_long_text() {
    let value = truncate_for_pet("abcdefghijklmnopqrstuvwxyz", 5);

    assert_eq!(value, "abcde...");
}

#[test]
fn current_open_think_body_uses_new_empty_think() {
    let body = current_open_think_body("<think>旧思考</think>\n普通回复\n<think>");

    assert_eq!(body, Some(String::new()));
}

#[test]
fn newest_open_thinking_text_ignores_visible_answer() {
    let messages = vec![ChatMessage {
        role: ChatRole::Assistant,
        content: "<think>分析实现路径".to_string(),
        think_timing: Vec::new(),
    }];

    let detail = newest_open_thinking_text(messages.iter());

    assert_eq!(detail, Some("分析实现路径".to_string()));
}

#[test]
fn newest_open_thinking_text_returns_none_without_open_think() {
    let messages = vec![ChatMessage {
        role: ChatRole::Assistant,
        content: "<think>分析完成</think>\n最终回答".to_string(),
        think_timing: Vec::new(),
    }];

    let detail = newest_open_thinking_text(messages.iter());

    assert_eq!(detail, None);
}
