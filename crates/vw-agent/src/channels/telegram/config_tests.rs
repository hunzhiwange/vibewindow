use super::TelegramChannel;

#[test]
fn with_api_base_trims_trailing_slashes() {
    let channel = TelegramChannel::new("123:ABC".to_string(), vec![], false)
        .with_api_base("https://api.example.test///".to_string());

    assert_eq!(channel.api_url("getMe"), "https://api.example.test/bot123:ABC/getMe");
}

#[test]
fn normalize_identity_accepts_usernames_and_numeric_ids() {
    assert_eq!(TelegramChannel::normalize_identity("@Alice"), "Alice");
    assert_eq!(TelegramChannel::normalize_identity(" 12345 "), "12345");
    assert_eq!(TelegramChannel::normalize_identity("@"), "");
}
