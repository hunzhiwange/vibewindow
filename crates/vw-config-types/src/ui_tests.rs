use super::{
    AppSystemSettingsConfig, AppUiConfig, GatewayClientServerConfig,
    GatewayClientSystemSettingsConfig, ModelRoute, PreviewAutoSaveMode,
};

#[test]
fn preview_auto_save_modes_have_stable_labels_and_display_text() {
    let labels = PreviewAutoSaveMode::ALL.map(PreviewAutoSaveMode::label);

    assert_eq!(PreviewAutoSaveMode::default(), PreviewAutoSaveMode::OnFocusChange);
    assert_eq!(labels, ["关闭", "延迟保存", "编辑器失焦时保存", "窗口失焦时保存"]);
    assert_eq!(PreviewAutoSaveMode::AfterDelay.to_string(), "延迟保存");
}

#[test]
fn app_system_settings_default_matches_desktop_contract() {
    let settings = AppSystemSettingsConfig::default();

    assert_eq!(settings.gateway_client.host, "127.0.0.1");
    assert_eq!(settings.gateway_client.port, 42617);
    assert_eq!(settings.app_theme, "Light");
    assert_eq!(settings.terminal_shell, "zsh");
    assert_eq!(settings.terminal_theme, "system");
    assert_eq!(settings.terminal_font_family, "JetBrains Mono");
    assert_eq!(settings.terminal_font_size, 13.0);
    assert!(settings.editor_follow_system_theme);
    assert_eq!(settings.editor_theme, "Light");
    assert_eq!(settings.editor_font_size, 14.0);
    assert_eq!(settings.editor_line_height, 20.0);
    assert!(settings.editor_auto_line_height);
    assert!(settings.dialogue_flow_show_reasoning_summary);
    assert!(!settings.dialogue_flow_expand_shell_tool_section);
    assert!(!settings.dialogue_flow_expand_edit_tool_section);
    assert_eq!(settings.preview_auto_save, PreviewAutoSaveMode::OnFocusChange);
}

#[test]
fn serde_defaults_fill_nested_ui_config() {
    let config: AppUiConfig = serde_json::from_value(serde_json::json!({})).unwrap();
    let settings: AppSystemSettingsConfig = serde_json::from_value(serde_json::json!({
        "gateway_client": {
            "host": "10.0.0.8",
            "servers": [{
                "id": "",
                "name": "",
                "host": "gw.internal",
                "port": 8080
            }]
        },
        "preview_auto_save": "afterDelay",
        "model_routes": [{
            "pattern": "code/*",
            "provider": "openai",
            "model": "gpt-5.4",
            "priority": 9
        }]
    }))
    .unwrap();

    assert_eq!(config.system_settings, AppSystemSettingsConfig::default());
    assert_eq!(settings.gateway_client.host, "10.0.0.8");
    assert_eq!(settings.gateway_client.port, 42617);
    assert_eq!(settings.preview_auto_save, PreviewAutoSaveMode::AfterDelay);
    assert_eq!(
        settings.model_routes,
        vec![ModelRoute {
            pattern: "code/*".to_string(),
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            priority: 9,
        }]
    );
}

#[test]
fn normalized_servers_falls_back_repairs_blanks_and_selects_active() {
    let gateway = GatewayClientSystemSettingsConfig {
        host: "gateway.local".to_string(),
        port: 19999,
        bearer_token: "token".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        skey: "skey".to_string(),
        active_server_id: String::new(),
        servers: Vec::new(),
    };
    let servers = gateway.normalized_servers();
    assert_eq!(servers[0].id, "local");
    assert_eq!(servers[0].name, "本地网关");
    assert_eq!(servers[0].host, "gateway.local");
    assert_eq!(servers[0].port, 19999);
    assert_eq!(servers[0].bearer_token, "token");

    let gateway = GatewayClientSystemSettingsConfig {
        active_server_id: "remote".to_string(),
        servers: vec![
            GatewayClientServerConfig {
                id: " ".to_string(),
                name: " ".to_string(),
                host: "one".to_string(),
                ..GatewayClientServerConfig::default()
            },
            GatewayClientServerConfig {
                id: "remote".to_string(),
                name: "".to_string(),
                host: "two".to_string(),
                ..GatewayClientServerConfig::default()
            },
        ],
        ..GatewayClientSystemSettingsConfig::default()
    };
    let normalized = gateway.normalized_servers();
    assert_eq!(normalized[0].id, "local");
    assert_eq!(normalized[0].name, "本地网关");
    assert_eq!(normalized[1].name, "网关 2");
    assert_eq!(gateway.active_server().id, "remote");
}

#[test]
fn set_servers_syncs_active_connection_fields_and_falls_back() {
    let mut gateway = GatewayClientSystemSettingsConfig::default();
    let local = GatewayClientServerConfig {
        id: "local".to_string(),
        host: "127.0.0.1".to_string(),
        port: 42617,
        ..GatewayClientServerConfig::default()
    };
    let remote = GatewayClientServerConfig {
        id: "remote".to_string(),
        name: "Remote".to_string(),
        host: "remote.example".to_string(),
        port: 443,
        bearer_token: "token".to_string(),
        username: "u".to_string(),
        password: "p".to_string(),
        skey: "s".to_string(),
    };

    gateway.set_servers(vec![local.clone(), remote.clone()], "remote".to_string());

    assert_eq!(gateway.active_server_id, "remote");
    assert_eq!(gateway.host, "remote.example");
    assert_eq!(gateway.port, 443);
    assert_eq!(gateway.bearer_token, "token");
    assert_eq!(gateway.username, "u");
    assert_eq!(gateway.password, "p");
    assert_eq!(gateway.skey, "s");
    assert_eq!(gateway.servers, vec![local, remote]);

    gateway.set_servers(Vec::new(), "missing".to_string());
    assert_eq!(gateway.active_server_id, "local");
    assert_eq!(gateway.servers, vec![GatewayClientServerConfig::default()]);
}
