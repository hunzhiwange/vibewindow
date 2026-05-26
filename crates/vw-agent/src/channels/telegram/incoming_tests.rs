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
