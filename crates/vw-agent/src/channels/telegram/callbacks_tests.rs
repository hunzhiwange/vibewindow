use super::TelegramChannel;

#[test]
fn approval_callback_parser_rejects_unknown_prefix() {
    assert!(TelegramChannel::parse_approval_callback_command("other:abc").is_none());
}
