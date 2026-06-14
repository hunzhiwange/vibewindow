use super::*;

#[test]
fn normalize_telegram_identity_trims_leading_at() {
    assert_eq!(normalize_telegram_identity("  @alice  "), "alice");
    assert_eq!(normalize_telegram_identity("bob"), "bob");
    assert_eq!(normalize_telegram_identity("  @@carol  "), "carol");
    assert_eq!(normalize_telegram_identity("   "), "");
}

#[test]
fn maybe_restart_managed_daemon_service_is_currently_noop() {
    assert!(!maybe_restart_managed_daemon_service().expect("restart check"));
}

#[tokio::test]
async fn bind_telegram_identity_rejects_empty_identity() {
    let config = Config::default();

    let err = bind_telegram_identity(&config, "  @  ").await.expect_err("empty identity fails");

    assert!(err.to_string().contains("cannot be empty"));
}

#[tokio::test]
async fn bind_telegram_identity_requires_configured_channel() {
    let config = Config::default();

    let err = bind_telegram_identity(&config, "@alice").await.expect_err("missing telegram config");

    assert!(err.to_string().contains("not configured"));
}

#[tokio::test]
async fn bind_telegram_identity_is_noop_when_identity_already_allowed() {
    let mut config = Config::default();
    config.channels_config.telegram = Some(crate::app::agent::config::TelegramConfig {
        bot_token: "token".to_string(),
        allowed_users: vec!["@alice".to_string()],
        stream_mode: crate::app::agent::config::StreamMode::Off,
        draft_update_interval_ms: 1000,
        interrupt_on_new_message: false,
        mention_only: false,
        group_reply: None,
        base_url: None,
    });

    bind_telegram_identity(&config, " alice ").await.expect("existing identity should succeed");
}
