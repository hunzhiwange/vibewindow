//! 维护聊天会话的加载、运行态和 UI 分块派生逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;

#[test]
fn prioritize_chat_ui_chunk_starts_keeps_base_first_on_initial_plan() {
    let (chunk_starts, anchor_chunk_start) =
        prioritize_chat_ui_chunk_starts(160, 64, 96, true, None, 32);

    assert_eq!(anchor_chunk_start, Some(64));
    assert_eq!(chunk_starts, vec![32, 64, 96]);
}

#[test]
fn prioritize_chat_ui_chunk_starts_prefers_forward_chunks_when_scrolling_down() {
    let (chunk_starts, anchor_chunk_start) =
        prioritize_chat_ui_chunk_starts(192, 72, 104, true, Some(32), 32);

    assert_eq!(anchor_chunk_start, Some(64));
    assert_eq!(chunk_starts, vec![96, 64, 128, 32]);
}

#[test]
fn prioritize_chat_ui_chunk_starts_prefers_backward_chunks_when_scrolling_up() {
    let (chunk_starts, anchor_chunk_start) =
        prioritize_chat_ui_chunk_starts(192, 40, 72, true, Some(96), 32);

    assert_eq!(anchor_chunk_start, Some(32));
    assert_eq!(chunk_starts, vec![32, 64, 0, 96]);
}

#[test]
fn apply_prepared_chat_ui_phase_preserves_explore_summary_animation_history() {
    let (mut app, _task) = App::new();
    let first = concat!(
        "tool read\n",
        "{\"status\":\"completed\",\"input\":\"{\\\"filePath\\\":\\\"/tmp/a.rs\\\",\\\"offset\\\":0,\\\"limit\\\":10}\",\"output\":\"ok\"}\n"
    );
    let second = concat!(
        "tool read\n",
        "{\"status\":\"completed\",\"input\":\"{\\\"filePath\\\":\\\"/tmp/a.rs\\\",\\\"offset\\\":0,\\\"limit\\\":10}\",\"output\":\"ok\"}\n",
        "tool read\n",
        "{\"status\":\"completed\",\"input\":\"{\\\"filePath\\\":\\\"/tmp/b.rs\\\",\\\"offset\\\":0,\\\"limit\\\":10}\",\"output\":\"ok\"}\n"
    );

    app.chat = vec![ChatMessage {
        role: ChatRole::Assistant,
        content: first.to_string(),
        think_timing: Vec::new(),
    }];
    app.chat_message_ids = vec![None];
    app.apply_prepared_chat_ui_phase(prepare_chat_ui_chunk_phase(&app.chat, 0, true));

    let key = explore_summary_animation_key(0, 0);
    let initial_state = app
        .chat_explore_summary_animations
        .get(&key)
        .expect("initial explore summary animation state should exist");
    assert_eq!(initial_state.previous_summary_text, "1 次读取");
    assert_eq!(initial_state.current_summary_text, "1 次读取");
    assert_eq!(initial_state.changed_at_ms, None);

    app.chat[0].content = second.to_string();
    app.apply_prepared_chat_ui_phase(prepare_chat_ui_chunk_phase(&app.chat, 0, true));

    let updated_state = app
        .chat_explore_summary_animations
        .get(&key)
        .expect("updated explore summary animation state should exist");
    assert_eq!(updated_state.previous_summary_text, "1 次读取");
    assert_eq!(updated_state.current_summary_text, "2 次读取");
    assert!(updated_state.changed_at_ms.is_some());
}

#[test]
fn prepare_chat_ui_chunk_phase_uses_display_text_for_tool_editor_cache() {
    let chat = vec![ChatMessage {
        role: ChatRole::Tool,
        content: concat!(
            "tool grep\n",
            "{\"status\":\"completed\",\"input\":\"selectors\",\"output\":\"2 matches\"}\n"
        )
        .to_string(),
        think_timing: Vec::new(),
    }];

    let PreparedChatUiPhase::Base(chunk) = prepare_chat_ui_chunk_phase(&chat, 0, true) else {
        panic!("base phase should be returned");
    };

    assert_eq!(chunk.message_editor_texts[0].as_deref(), Some("2 matches"));
    assert_eq!(chunk.visible_texts[0].as_deref(), Some(chat[0].content.as_str()));
}

#[test]
fn rebuild_active_session_message_meta_keeps_tool_outside_step_indexing() {
    let (mut app, _task) = App::new();
    app.active_session_view_state.updated_ms = 3_000;
    app.chat = vec![
        ChatMessage {
            role: ChatRole::Assistant,
            content: "先给出结论".to_string(),
            think_timing: Vec::new(),
        },
        ChatMessage {
            role: ChatRole::Tool,
            content: "tool grep\n{\"status\":\"completed\",\"output\":\"2 matches\"}\n"
                .to_string(),
            think_timing: Vec::new(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "继续".to_string(),
            think_timing: Vec::new(),
        },
    ];

    app.upsert_active_session_step(ChatSessionStep {
        index: 1,
        started_ms: 1_000,
        finished_ms: Some(1_400),
        start_snapshot_path: None,
        finish_snapshot_path: None,
        usage: TokenUsage::default(),
        cost_usd: None,
        finish_reason: Some("stop".to_string()),
        model: Some("model-a".to_string()),
    });
    app.upsert_active_session_step(ChatSessionStep {
        index: 2,
        started_ms: 2_000,
        finished_ms: Some(2_200),
        start_snapshot_path: None,
        finish_snapshot_path: None,
        usage: TokenUsage::default(),
        cost_usd: None,
        finish_reason: Some("stop".to_string()),
        model: Some("model-b".to_string()),
    });

    app.rebuild_active_session_message_meta();

    assert_eq!(app.active_session_view_state.message_meta_texts.len(), 3);
    assert!(app.active_session_view_state.message_meta_texts[0]
        .as_deref()
        .is_some_and(|text| text.contains("model-a")));
    assert_eq!(app.active_session_view_state.message_meta_texts[1], None);
    assert!(app.active_session_view_state.message_meta_texts[2]
        .as_deref()
        .is_some_and(|text| text.contains("model-b")));
}
