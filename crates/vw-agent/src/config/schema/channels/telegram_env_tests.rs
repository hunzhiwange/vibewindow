use super::{resolve_telegram_allowed_users_env_refs, ChannelsConfig, TelegramConfig};

#[test]
fn telegram_env_refs_trim_plain_entries_without_env_lookup() {
    let mut channels = ChannelsConfig {
        telegram: Some(TelegramConfig {
            bot_token: "token".to_string(),
            allowed_users: vec![" alice ".to_string(), "".to_string(), "123".to_string()],
            stream_mode: Default::default(),
            draft_update_interval_ms: 1000,
            interrupt_on_new_message: false,
            mention_only: false,
            group_reply: None,
            base_url: None,
        }),
        ..Default::default()
    };

    resolve_telegram_allowed_users_env_refs(&mut channels).unwrap();

    assert_eq!(
        channels.telegram.unwrap().allowed_users,
        vec!["alice".to_string(), "123".to_string()]
    );
}
