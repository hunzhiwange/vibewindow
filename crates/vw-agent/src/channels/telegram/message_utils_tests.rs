use super::message_utils::{
    TELEGRAM_ACK_REACTIONS, TELEGRAM_MAX_MESSAGE_LENGTH, build_telegram_ack_reaction_request,
    random_telegram_ack_reaction, split_message_for_telegram,
};

#[test]
fn split_message_keeps_short_message_single_chunk() {
    assert_eq!(split_message_for_telegram("short"), vec!["short".to_string()]);
}

#[test]
fn split_message_respects_telegram_limit() {
    let message = "a".repeat(TELEGRAM_MAX_MESSAGE_LENGTH + 10);
    let chunks = split_message_for_telegram(&message);

    assert!(chunks.len() > 1);
    assert!(chunks.iter().all(|chunk| chunk.chars().count() <= TELEGRAM_MAX_MESSAGE_LENGTH));
}

#[test]
fn ack_reaction_request_preserves_fields() {
    let request = build_telegram_ack_reaction_request("chat", 42, "👍");

    assert_eq!(request["chat_id"], "chat");
    assert_eq!(request["message_id"], 42);
    assert!(TELEGRAM_ACK_REACTIONS.contains(&random_telegram_ack_reaction()));
}
