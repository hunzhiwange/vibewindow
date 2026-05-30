use super::TelegramChannel;

#[test]
fn api_url_uses_configured_base_and_token() {
    let channel = TelegramChannel::new("123:ABC".to_string(), vec![], false)
        .with_api_base("https://tapi.example.test".to_string());

    assert_eq!(channel.api_url("sendMessage"), "https://tapi.example.test/bot123:ABC/sendMessage");
}

#[test]
fn sanitize_telegram_error_redacts_bot_token() {
    let sanitized = TelegramChannel::sanitize_telegram_error(
        "failed https://api.telegram.org/bot123:SECRET/getMe",
    );

    assert!(!sanitized.contains("SECRET"));
    assert!(sanitized.contains("[redacted]"));
}
