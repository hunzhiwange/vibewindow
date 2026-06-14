#![allow(unused_must_use)]
use super::*;
use crate::app::state::{RedisConnectionTab, RedisDetailTab, RedisKeyValueKind};

fn app() -> App {
    App::new().0
}

#[test]
fn update_routes_modal_and_navigation_messages() {
    let mut app = app();

    update(&mut app, RedisToolMessage::OpenSettingsModal);
    assert!(app.redis_tool.show_settings_modal);

    update(&mut app, RedisToolMessage::CloseSettingsModal);
    assert!(!app.redis_tool.show_settings_modal);

    update(&mut app, RedisToolMessage::SearchConnectionsChanged("prod".to_string()));
    assert_eq!(app.redis_tool.connection_search_query, "prod");

    update(&mut app, RedisToolMessage::KeyBrowserPatternChanged("app:*".to_string()));
    update(&mut app, RedisToolMessage::ToggleKeyTreePath("app".to_string()));
    update(&mut app, RedisToolMessage::InfoFilterChanged("memory".to_string()));

    assert_eq!(app.redis_tool.key_browser_pattern, "app:*");
    assert!(app.redis_tool.key_tree_expanded_paths.contains("app"));
    assert_eq!(app.redis_tool.info_filter, "memory");
}

#[test]
fn update_routes_draft_input_messages() {
    let mut app = app();

    update(&mut app, RedisToolMessage::DraftNameChanged("Primary".to_string()));
    update(&mut app, RedisToolMessage::DraftHostChanged("redis.local".to_string()));
    update(&mut app, RedisToolMessage::DraftPortChanged("6380".to_string()));
    update(&mut app, RedisToolMessage::DraftDbChanged("2".to_string()));
    update(&mut app, RedisToolMessage::DraftUsernameChanged("user".to_string()));
    update(&mut app, RedisToolMessage::DraftPasswordChanged("secret".to_string()));
    update(&mut app, RedisToolMessage::DraftTabChanged(RedisConnectionTab::Tls));
    update(&mut app, RedisToolMessage::DraftTlsToggled(true));
    update(
        &mut app,
        RedisToolMessage::DraftTlsPrivateKeyPathChanged("/tmp/client.key".to_string()),
    );
    update(&mut app, RedisToolMessage::DraftSshEnabledToggled(true));
    update(&mut app, RedisToolMessage::DraftSshHostChanged("bastion".to_string()));
    update(&mut app, RedisToolMessage::DraftSentinelEnabledToggled(true));
    update(&mut app, RedisToolMessage::DraftClusterToggled(true));
    update(&mut app, RedisToolMessage::DraftReadOnlyToggled(true));
    update(&mut app, RedisToolMessage::DraftKeyPatternChanged("app:*".to_string()));

    assert_eq!(app.redis_tool.draft.name, "Primary");
    assert_eq!(app.redis_tool.draft.host, "redis.local");
    assert_eq!(app.redis_tool.draft.port, "6380");
    assert_eq!(app.redis_tool.draft.db, "2");
    assert_eq!(app.redis_tool.draft.username, "user");
    assert_eq!(app.redis_tool.draft.password, "secret");
    assert_eq!(app.redis_tool.draft_tab, RedisConnectionTab::Tls);
    assert!(app.redis_tool.draft.use_tls);
    assert_eq!(app.redis_tool.draft.tls_cert.private_key_path, "/tmp/client.key");
    assert!(app.redis_tool.draft.ssh_tunnel.enabled);
    assert_eq!(app.redis_tool.draft.ssh_tunnel.host, "bastion");
    assert!(app.redis_tool.draft.sentinel.enabled);
    assert!(app.redis_tool.draft.use_cluster);
    assert!(app.redis_tool.draft.read_only);
    assert_eq!(app.redis_tool.draft.key_pattern, "app:*");
}

#[test]
fn update_routes_operation_input_messages() {
    let mut app = app();

    update(&mut app, RedisToolMessage::DefaultLoadCountChanged("25".to_string()));
    update(&mut app, RedisToolMessage::IncreaseDefaultLoadCount);
    update(&mut app, RedisToolMessage::DecreaseDefaultLoadCount);
    update(&mut app, RedisToolMessage::CreateKeyNameChanged("app:key".to_string()));
    update(&mut app, RedisToolMessage::CreateKeyTypeChanged(RedisKeyValueKind::List));
    update(&mut app, RedisToolMessage::CommandInputChanged("PING".to_string()));
    update(&mut app, RedisToolMessage::DetailTabChanged(RedisDetailTab::Command));
    update(&mut app, RedisToolMessage::ClearNotification);
    update(&mut app, RedisToolMessage::ClearGatewayError);

    assert_eq!(app.redis_tool.default_load_count_input, "25");
    assert_eq!(app.redis_tool.create_key_draft.name, "app:key");
    assert_eq!(app.redis_tool.create_key_draft.key_type, RedisKeyValueKind::List);
    assert_eq!(app.redis_tool.command_input, "PING");
    assert_eq!(app.redis_tool.detail_tab, RedisDetailTab::Command);
    assert_eq!(app.redis_tool.notification, None);
    assert_eq!(app.redis_tool.gateway_error, None);
}
