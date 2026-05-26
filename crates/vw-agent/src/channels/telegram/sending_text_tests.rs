use super::message_utils::split_message_for_telegram;

#[test]
fn sending_text_uses_plain_single_chunk_for_empty_message() {
    assert_eq!(split_message_for_telegram(""), vec!["".to_string()]);
}
