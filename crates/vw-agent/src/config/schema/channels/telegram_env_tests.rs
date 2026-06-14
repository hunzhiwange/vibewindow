use super::{ChannelsConfig, TelegramConfig, resolve_telegram_allowed_users_env_refs};

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

#[test]
fn telegram_env_refs_expand_json_and_comma_lists() {
    unsafe {
        std::env::set_var("VW_TEST_ALLOWED_JSON", r#"["100", 200, " 300 "]"#);
        std::env::set_var("VW_TEST_ALLOWED_LIST", "400, 500,,");
    }
    let mut channels = ChannelsConfig {
        telegram: Some(TelegramConfig {
            bot_token: "token".to_string(),
            allowed_users: vec![
                "${env:VW_TEST_ALLOWED_JSON}".to_string(),
                "${env:VW_TEST_ALLOWED_LIST}".to_string(),
            ],
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

    unsafe {
        std::env::remove_var("VW_TEST_ALLOWED_JSON");
        std::env::remove_var("VW_TEST_ALLOWED_LIST");
    }
    assert_eq!(
        channels.telegram.unwrap().allowed_users,
        vec![
            "100".to_string(),
            "200".to_string(),
            "300".to_string(),
            "400".to_string(),
            "500".to_string()
        ]
    );
}

#[test]
fn telegram_env_refs_reject_invalid_env_names_and_empty_values() {
    let mut channels = ChannelsConfig {
        telegram: Some(TelegramConfig {
            bot_token: "token".to_string(),
            allowed_users: vec!["${env:BAD-NAME}".to_string()],
            stream_mode: Default::default(),
            draft_update_interval_ms: 1000,
            interrupt_on_new_message: false,
            mention_only: false,
            group_reply: None,
            base_url: None,
        }),
        ..Default::default()
    };

    assert!(
        resolve_telegram_allowed_users_env_refs(&mut channels)
            .unwrap_err()
            .to_string()
            .contains("invalid env var name")
    );

    unsafe {
        std::env::set_var("VW_TEST_ALLOWED_EMPTY", "   ");
    }
    channels.telegram.as_mut().unwrap().allowed_users =
        vec!["${env:VW_TEST_ALLOWED_EMPTY}".to_string()];
    let err = resolve_telegram_allowed_users_env_refs(&mut channels).unwrap_err().to_string();
    unsafe {
        std::env::remove_var("VW_TEST_ALLOWED_EMPTY");
    }

    assert!(err.contains("empty value"));
}
