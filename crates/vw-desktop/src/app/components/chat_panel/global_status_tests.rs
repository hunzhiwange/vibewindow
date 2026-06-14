use super::global_status::chat_global_status;
use crate::app::models::{ChatMessage, ChatRole};

fn assistant(content: &str) -> ChatMessage {
    ChatMessage { role: ChatRole::Assistant, content: content.to_string(), think_timing: vec![] }
}

#[test]
fn running_read_tool_becomes_latest_status() {
    let chat = vec![assistant(
        "tool read\n{\"status\":\"running\",\"input\":\"{\\\"filePath\\\":\\\"src/main.rs\\\"}\"}",
    )];

    let status = chat_global_status(&chat, true);

    let status = status.expect("running status should render");
    assert_eq!(status.action, "正在读取");
    assert_eq!(status.detail, "src/main.rs");
}

#[test]
fn latest_tool_replaces_previous_status() {
    let chat = vec![assistant(
        "tool read\n{\"status\":\"completed\",\"input\":\"{\\\"filePath\\\":\\\"old.rs\\\"}\"}\n\
         tool shell\n{\"status\":\"running\",\"input\":\"{\\\"command\\\":\\\"cargo clippy\\\"}\"}",
    )];

    let status = chat_global_status(&chat, true);

    let status = status.expect("latest running status should render");
    assert_eq!(status.action, "正在运行");
    assert_eq!(status.detail, "cargo clippy");
}

#[test]
fn open_think_without_tool_reports_thinking() {
    let chat = vec![assistant("<think>第一行\n第二行\n第三行")];

    let status = chat_global_status(&chat, true);

    let status = status.expect("thinking status should render");
    assert_eq!(status.action, "正在思考");
    assert_eq!(status.detail, "第二行\n第三行");
}

#[test]
fn completed_tool_without_request_hides_status() {
    let chat = vec![assistant(
        "tool shell\n{\"status\":\"completed\",\"input\":\"{\\\"command\\\":\\\"cargo clippy\\\"}\"}",
    )];

    assert_eq!(chat_global_status(&chat, false), None);
}

#[test]
fn idle_chat_hides_status() {
    assert_eq!(chat_global_status(&[], false), None);
}

#[test]
fn closed_think_keeps_status_until_request_finishes() {
    let chat = vec![assistant("<think>分析现有模块</think>")];

    let status = chat_global_status(&chat, true).expect("requesting think status should render");
    assert_eq!(status.action, "正在思考");
    assert_eq!(status.detail, "分析现有模块");
    assert_eq!(chat_global_status(&chat, false), None);
}
