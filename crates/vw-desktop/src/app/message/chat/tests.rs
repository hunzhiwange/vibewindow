#[test]
fn tests_module_is_wired() {
    assert!(module_path!().ends_with("tests"));
}

#[test]
fn think_block_key_packs_message_and_think_indexes() {
    assert_eq!(super::think_block_key(0, 7), 7);
    assert_eq!(super::think_block_key(2, 3), (2_u64 << 32) | 3);
}

#[test]
fn fork_session_target_and_actions_keep_expected_equality() {
    assert_eq!(super::ForkSessionTarget::Local, super::ForkSessionTarget::Local);
    assert_ne!(super::ForkSessionTarget::Local, super::ForkSessionTarget::NewWorktree);

    assert_eq!(
        super::MessageIndexedSessionAction::Reset { revert_code: true },
        super::MessageIndexedSessionAction::Reset { revert_code: true }
    );
    assert_ne!(
        super::MessageIndexedSessionAction::Reset { revert_code: true },
        super::MessageIndexedSessionAction::Reset { revert_code: false }
    );
}

#[test]
fn clipboard_payload_variants_are_constructible() {
    let payloads = [
        super::ClipboardPastePayload::Text("text".to_string()),
        super::ClipboardPastePayload::AttachmentPath("/tmp/a.png".to_string()),
        super::ClipboardPastePayload::Empty,
        super::ClipboardPastePayload::Error("boom".to_string()),
    ];

    assert_eq!(payloads.len(), 4);
}

#[test]
fn arm_autoscroll_hold_enables_follow_mode_and_extends_deadline() {
    let (mut app, _task) = crate::app::App::new();
    app.chat_auto_scroll = false;
    app.chat_autoscroll_hold_until_ms = 0;

    super::arm_autoscroll_hold(&mut app);

    assert!(app.chat_auto_scroll);
    assert!(app.chat_autoscroll_hold_until_ms >= super::AUTOSCROLL_HOLD_MS);
}

#[test]
fn throttled_stream_autoscroll_respects_follow_mode_and_window() {
    let (mut app, _task) = crate::app::App::new();
    app.chat_auto_scroll = false;
    assert!(super::throttled_stream_autoscroll_task(&mut app).is_none());

    app.chat_auto_scroll = true;
    app.chat_stream_autoscroll_last_ms = super::now_ms();
    assert!(super::throttled_stream_autoscroll_task(&mut app).is_none());

    app.chat_stream_autoscroll_last_ms = 0;
    assert!(super::throttled_stream_autoscroll_task(&mut app).is_some());
}
