use super::*;

#[test]
fn split_message_for_discord_keeps_short_messages_intact() {
    assert_eq!(split_message_for_discord("hello"), vec!["hello".to_string()]);
}

#[test]
fn split_message_for_discord_respects_unicode_character_limit() {
    let message = "界".repeat(DISCORD_MAX_MESSAGE_LENGTH + 1);
    let chunks = split_message_for_discord(&message);

    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().all(|chunk| chunk.chars().count() <= DISCORD_MAX_MESSAGE_LENGTH));
    assert_eq!(chunks.concat(), message);
}
