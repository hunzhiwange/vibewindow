use super::TelegramChannel;

#[test]
fn group_message_detects_negative_chat_id() {
    let message = serde_json::json!({"chat": {"id": -100123, "type": "supergroup"}});

    assert!(TelegramChannel::is_group_message(&message));
}

#[test]
fn reply_target_prefers_message_thread_id_when_present() {
    let update = serde_json::json!({
        "message": {
            "chat": {"id": -100123},
            "message_id": 55,
            "message_thread_id": 7
        }
    });

    let target = TelegramChannel::extract_update_message_target(&update).unwrap();

    assert_eq!(target, ("-100123".to_string(), 55));
}

#[test]
fn reply_target_round_trips_plain_and_threaded_targets() {
    assert_eq!(TelegramChannel::parse_reply_target("12345"), ("12345".to_string(), None));
    assert_eq!(
        TelegramChannel::parse_reply_target("-100:77"),
        ("-100".to_string(), Some("77".to_string()))
    );
}

#[test]
fn extract_update_message_target_requires_message_chat_and_id() {
    assert_eq!(
        TelegramChannel::extract_update_message_target(&serde_json::json!({
            "callback_query": {"message": {"message_id": 1}}
        })),
        None
    );
    assert_eq!(
        TelegramChannel::extract_update_message_target(&serde_json::json!({
            "message": {"chat": {"id": 123}}
        })),
        None
    );
    assert_eq!(
        TelegramChannel::extract_update_message_target(&serde_json::json!({
            "message": {"message_id": 9, "chat": {"id": "123"}}
        })),
        None
    );
}

#[test]
fn group_message_only_accepts_group_types() {
    assert!(TelegramChannel::is_group_message(&serde_json::json!({"chat": {"type": "group"}})));
    assert!(TelegramChannel::is_group_message(
        &serde_json::json!({"chat": {"type": "supergroup"}})
    ));
    assert!(!TelegramChannel::is_group_message(&serde_json::json!({"chat": {"type": "private"}})));
    assert!(!TelegramChannel::is_group_message(&serde_json::json!({})));
}

#[test]
fn extract_sender_info_falls_back_to_id_then_unknown() {
    let with_id = serde_json::json!({"from": {"id": 42}});
    assert_eq!(
        TelegramChannel::extract_sender_info(&with_id),
        ("unknown".to_string(), Some("42".to_string()), "42".to_string())
    );

    let missing = serde_json::json!({});
    assert_eq!(
        TelegramChannel::extract_sender_info(&missing),
        ("unknown".to_string(), None, "unknown".to_string())
    );
}

#[test]
fn extract_reply_context_formats_media_and_multiline_text() {
    let ch = TelegramChannel::new("token".into(), vec!["*".into()], false);

    let text_reply = serde_json::json!({
        "reply_to_message": {
            "from": {"first_name": "Alice"},
            "text": "line one\nline two"
        }
    });
    assert_eq!(ch.extract_reply_context(&text_reply).unwrap(), "> @Alice:\n> line one\n> line two");

    for (field, expected) in [
        ("photo", "[Photo]"),
        ("document", "[Document]"),
        ("video", "[Video]"),
        ("sticker", "[Sticker]"),
    ] {
        let message = serde_json::json!({
            "reply_to_message": {
                "from": {"username": "bot"},
                field: {}
            }
        });
        assert_eq!(ch.extract_reply_context(&message).unwrap(), format!("> @bot:\n> {expected}"));
    }
}

#[test]
fn extract_reply_context_uses_cached_voice_transcription_when_available() {
    let ch = TelegramChannel::new("token".into(), vec!["*".into()], false);
    ch.voice_transcriptions.lock().insert("100:55".to_string(), "cached words".to_string());

    let message = serde_json::json!({
        "chat": {"id": 100},
        "reply_to_message": {
            "message_id": 55,
            "from": {"username": "alice"},
            "voice": {"file_id": "voice"}
        }
    });

    assert_eq!(ch.extract_reply_context(&message).unwrap(), "> @alice:\n> [Voice] cached words");
}

#[test]
fn parse_update_message_filters_missing_text_and_unauthorized_senders() {
    let ch = TelegramChannel::new("token".into(), vec!["alice".into()], false);

    assert!(ch.parse_update_message(&serde_json::json!({"message": {"photo": []}})).is_none());

    let update = serde_json::json!({
        "message": {
            "message_id": 1,
            "text": "hello",
            "from": {"id": 2, "username": "bob"},
            "chat": {"id": 123, "type": "private"}
        }
    });
    assert!(ch.parse_update_message(&update).is_none());
}

#[test]
fn parse_update_message_builds_private_and_threaded_channel_messages() {
    let ch = TelegramChannel::new("token".into(), vec!["alice".into()], false);
    let update = serde_json::json!({
        "message": {
            "message_id": 44,
            "message_thread_id": 9,
            "text": "hello",
            "from": {"id": 2, "username": "alice"},
            "chat": {"id": -100, "type": "supergroup"}
        }
    });

    let parsed = ch.parse_update_message(&update).unwrap();

    assert_eq!(parsed.id, "telegram_-100_44");
    assert_eq!(parsed.sender, "alice");
    assert_eq!(parsed.reply_target, "-100:9");
    assert_eq!(parsed.thread_ts.as_deref(), Some("9"));
    assert_eq!(parsed.channel, "telegram");
    assert_eq!(parsed.content, "hello");
}

#[test]
fn parse_update_message_honors_mention_only_and_allowed_group_sender_bypass() {
    let ch = TelegramChannel::new("token".into(), vec!["alice".into()], true);
    *ch.bot_username.lock() = Some("vibebot".to_string());

    let unmentioned = serde_json::json!({
        "message": {
            "message_id": 1,
            "text": "hello room",
            "from": {"id": 2, "username": "alice"},
            "chat": {"id": -100, "type": "group"}
        }
    });
    assert!(ch.parse_update_message(&unmentioned).is_none());

    let mentioned = serde_json::json!({
        "message": {
            "message_id": 2,
            "text": "@vibebot please help",
            "from": {"id": 2, "username": "alice"},
            "chat": {"id": -100, "type": "group"}
        }
    });
    assert_eq!(ch.parse_update_message(&mentioned).unwrap().content, "please help");

    let bypass = TelegramChannel::new("token".into(), vec!["alice".into()], true)
        .with_group_reply_allowed_senders(vec!["2".into()]);
    let parsed = bypass.parse_update_message(&unmentioned).unwrap();
    assert_eq!(parsed.content, "hello room");
}
