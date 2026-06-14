#[test]
fn stream_events_tests_module_is_wired() {
    assert!(module_path!().ends_with("stream_events_tests"));
}

use crate::app::QueueItem;
use crate::app::models::{ChatMessage, ChatRole, ThinkTiming};
use crate::app::state::{AcpHistoryReplayMode, ChatSendBehavior};

fn message(role: ChatRole, content: &str) -> ChatMessage {
    ChatMessage { role, content: content.to_string(), think_timing: Vec::new() }
}

fn queue_item(send_behavior: ChatSendBehavior) -> QueueItem {
    QueueItem {
        created_ms: 1,
        query: "query".to_string(),
        attachments: Vec::new(),
        root: None,
        model: None,
        acp_test: false,
        acp_agent: None,
        acp_allowed_tools: None,
        agent: None,
        allowed_tools: None,
        acp_force_new_session: false,
        acp_history_mode: AcpHistoryReplayMode::Discard,
        acp_recent_count: 0,
        full_access_enabled: false,
        send_behavior,
        request_history_override: None,
        resume_history_only: false,
        workflow_mode_enabled: false,
    }
}

#[test]
fn update_think_timing_ignores_empty_delta() {
    let mut message = message(ChatRole::Assistant, "<think>open");
    message.think_timing.push(ThinkTiming { start_ms: 1, end_ms: None, last_update_ms: 1 });

    super::update_think_timing_from_delta(&mut message, "", 99);

    assert_eq!(message.think_timing[0].last_update_ms, 1);
    assert_eq!(message.think_timing[0].end_ms, None);
}

#[test]
fn update_think_timing_opens_and_closes_think_block() {
    let mut message = message(ChatRole::Assistant, "");

    super::update_think_timing_from_delta(&mut message, "<think>working", 10);
    message.content.push_str("<think>working");
    super::update_think_timing_from_delta(&mut message, "</think>done", 25);

    assert_eq!(message.think_timing.len(), 1);
    assert_eq!(message.think_timing[0].start_ms, 10);
    assert_eq!(message.think_timing[0].end_ms, Some(25));
    assert_eq!(message.think_timing[0].last_update_ms, 25);
}

#[test]
fn stream_done_message_ids_resize_and_assign_last_assistant_and_parent_user() {
    let chat = vec![
        message(ChatRole::User, "first"),
        message(ChatRole::Assistant, "old"),
        message(ChatRole::User, "latest"),
        message(ChatRole::Assistant, "answer"),
    ];
    let mut ids = vec![Some("first-id".to_string())];

    super::apply_stream_done_message_ids(
        &chat,
        &mut ids,
        Some("assistant-id".to_string()),
        Some("parent-id".to_string()),
    );

    assert_eq!(ids.len(), 4);
    assert_eq!(ids[0].as_deref(), Some("first-id"));
    assert_eq!(ids[2].as_deref(), Some("parent-id"));
    assert_eq!(ids[3].as_deref(), Some("assistant-id"));
}

#[test]
fn message_ids_tail_returns_at_most_four_entries() {
    let ids = vec![Some("1".to_string()), None, Some("3".to_string()), None, Some("5".to_string())];

    assert_eq!(
        super::message_ids_tail(&ids),
        vec![None, Some("3".to_string()), None, Some("5".to_string())]
    );
}

#[test]
fn parse_tool_block_len_requires_tool_prefix_and_complete_json() {
    let block = "tool shell\n{\"tool_call_id\":\"call-1\",\"status\":\"ok\"}\ntrailing";

    assert_eq!(super::parse_tool_block_len(block), Some(block.find("trailing").unwrap()));
    assert_eq!(super::parse_tool_block_len("prefix tool shell\n{}"), None);
    assert_eq!(super::parse_tool_block_len("tool shell\n{"), None);
}

#[test]
fn tool_block_start_allowed_accepts_line_start_and_colon_prefix_only() {
    assert!(super::tool_block_start_allowed("tool shell\n{}", 0));
    let ascii_colon = "工具:\ntool shell\n{}";
    let full_width_colon = "工具：\ntool shell\n{}";
    assert!(super::tool_block_start_allowed(ascii_colon, ascii_colon.find("tool").unwrap()));
    assert!(super::tool_block_start_allowed(
        full_width_colon,
        full_width_colon.find("tool").unwrap()
    ));
    assert!(!super::tool_block_start_allowed("inline tool shell\n{}", 7));
}

#[test]
fn workflow_tool_block_upsert_replaces_matching_call_id() {
    let running = r#"tool workflow_node
{"tool_call_id":"workflow-node-2-llm","status":"running","summary":"Run Task · llm"}
"#;
    let finished = r#"tool workflow_node
{"tool_call_id":"workflow-node-2-llm","status":"completed","summary":"Run Task · llm","output":"ok"}
"#;
    let mut content = String::new();

    super::upsert_tool_block_by_call_id(&mut content, running);
    super::upsert_tool_block_by_call_id(&mut content, finished);

    assert_eq!(content.matches("tool workflow_node").count(), 1);
    assert!(content.contains("\"status\":\"completed\""));
    assert!(!content.contains("\"status\":\"running\""));
}

#[test]
fn workflow_tool_block_upsert_appends_when_call_id_is_missing_or_different() {
    let mut content = "prefix".to_string();

    super::upsert_tool_block_by_call_id(&mut content, "tool shell\n{\"status\":\"ok\"}\n");
    super::upsert_tool_block_by_call_id(
        &mut content,
        "tool shell\n{\"tool_call_id\":\"call-2\",\"status\":\"ok\"}\n",
    );

    assert_eq!(content.matches("tool shell").count(), 2);
    assert!(content.ends_with('\n'));
}

#[test]
fn leading_guide_count_only_counts_initial_guide_items() {
    let guide = queue_item(ChatSendBehavior::Guide);
    let normal = queue_item(ChatSendBehavior::Queue);
    let late_guide = queue_item(ChatSendBehavior::Guide);

    assert_eq!(super::leading_guide_count(&[guide, normal, late_guide]), 1);
}

#[test]
fn continuation_label_from_history_uses_last_user_message() {
    let history = vec![
        message(ChatRole::User, "first"),
        message(ChatRole::Assistant, "answer"),
        message(ChatRole::User, "second"),
    ];

    assert_eq!(super::continuation_label_from_history(&history), "second");
    assert_eq!(super::continuation_label_from_history(&[]), "");
}
