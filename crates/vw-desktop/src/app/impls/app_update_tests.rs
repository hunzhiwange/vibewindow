use super::*;
use crate::app::state::{ExternalOpenApp, RuntimePlatform};
use serde_json::json;

fn test_app() -> App {
    App::new().0
}

#[test]
fn sort_acp_agent_names_keeps_codex_first_and_sorts_remaining_names() {
    let names = vec!["zeta".to_string(), "codex".to_string(), "alpha".to_string()];

    assert_eq!(sort_acp_agent_names(names), ["codex", "alpha", "zeta"]);
}

#[test]
fn parse_external_open_app_accepts_known_stable_ids() {
    assert_eq!(App::parse_external_open_app("finder"), Some(ExternalOpenApp::Finder));
    assert_eq!(App::parse_external_open_app("vscode"), Some(ExternalOpenApp::VSCode));
    assert_eq!(
        App::parse_external_open_app("android-studio"),
        Some(ExternalOpenApp::AndroidStudio)
    );
    assert_eq!(App::parse_external_open_app("powershell"), Some(ExternalOpenApp::PowerShell));
    assert_eq!(App::parse_external_open_app("unknown"), None);
}

#[test]
fn external_open_priority_is_platform_specific() {
    assert_eq!(App::external_open_priority(Some(RuntimePlatform::MacOs))[0], ExternalOpenApp::Trae);
    assert!(
        App::external_open_priority(Some(RuntimePlatform::MacOs))
            .contains(&ExternalOpenApp::Terminal)
    );
    assert!(
        App::external_open_priority(Some(RuntimePlatform::Windows))
            .contains(&ExternalOpenApp::PowerShell)
    );
    assert!(
        !App::external_open_priority(Some(RuntimePlatform::Linux))
            .contains(&ExternalOpenApp::PowerShell)
    );
}

#[test]
fn apply_external_apps_state_selects_available_priority_and_keeps_finder_available() {
    let mut app = test_app();
    app.open_external_app = ExternalOpenApp::VSCode;

    app.apply_external_apps_state(
        Some(RuntimePlatform::MacOs),
        vec![
            ("cursor".to_string(), false),
            ("zed".to_string(), true),
            ("unknown".to_string(), true),
        ],
    );

    assert_eq!(app.open_external_platform, Some(RuntimePlatform::MacOs));
    assert_eq!(app.open_external_app, ExternalOpenApp::Zed);
    assert_eq!(app.open_external_exists.get(&ExternalOpenApp::Finder), Some(&true));
    assert_eq!(app.open_external_exists.get(&ExternalOpenApp::Cursor), Some(&false));
}

#[test]
fn apply_external_apps_state_preserves_current_app_when_still_available() {
    let mut app = test_app();
    app.open_external_platform = Some(RuntimePlatform::Linux);
    app.open_external_app = ExternalOpenApp::Cursor;

    app.apply_external_apps_state(None, vec![("cursor".to_string(), true)]);

    assert_eq!(app.open_external_platform, Some(RuntimePlatform::Linux));
    assert_eq!(app.open_external_app, ExternalOpenApp::Cursor);
}

#[test]
fn apply_project_chat_preferences_updates_current_and_empty_runtime_state() {
    let mut app = test_app();

    app.apply_project_chat_preferences("gpt-test".to_string(), true, Some("codex".to_string()));

    assert_eq!(app.model, "gpt-test");
    assert!(app.auto_model);
    assert_eq!(app.acp_agent.as_deref(), Some("codex"));

    let runtime = app.session_runtime_states.get("__empty__").expect("empty runtime state");
    assert_eq!(runtime.model, "gpt-test");
    assert!(runtime.auto_model);
    assert_eq!(runtime.acp_agent.as_deref(), Some("codex"));
}

#[test]
fn bootstrap_app_config_updates_known_fields_and_clamps_file_manager_width() {
    let mut app = test_app();

    let _ = app.update(Message::BootstrapAppConfig(Ok(json!({
        "model": "gpt-next",
        "auto_model": true,
        "acp_agent": "codex",
        "show_settings": true,
        "show_file_manager": true,
        "file_manager_width": 9999.0,
        "open_external_app": "cursor",
        "file_tree_expanded": ["src", 7, "crates"]
    }))));

    assert_eq!(app.model, "gpt-next");
    assert!(app.auto_model);
    assert_eq!(app.acp_agent.as_deref(), Some("codex"));
    assert!(app.show_settings);
    assert!(app.show_file_manager);
    assert_eq!(app.file_manager_width, 600.0);
    assert_eq!(app.open_external_app, ExternalOpenApp::Cursor);
    assert_eq!(app.file_tree_expanded, ["src", "crates"]);
    assert!(app.file_tree_expanded_set.contains("src"));
}

#[test]
fn bootstrap_browser_config_normalizes_invalid_options() {
    let mut app = test_app();
    let mut browser = vw_config_types::tools::BrowserConfig::default();
    browser.enabled = true;
    browser.allowed_domains = vec!["example.com".to_string(), "docs.example.com".to_string()];
    browser.browser_open = "unsupported".to_string();
    browser.backend = "rust-native".to_string();
    browser.session_name = Some("desk".to_string());
    browser.native_chrome_path = Some("/Applications/Chrome".to_string());
    browser.computer_use.window_allowlist = vec!["Vibe".to_string(), "Terminal".to_string()];
    browser.computer_use.max_coordinate_x = Some(1440);
    browser.computer_use.max_coordinate_y = Some(900);

    let _ = app.update(Message::BootstrapBrowserConfig(Ok(browser)));

    assert!(app.browser_settings.enabled);
    assert_eq!(app.browser_settings.allowed_domains_input, "example.com\ndocs.example.com");
    assert_eq!(app.browser_settings.browser_open, "default");
    assert_eq!(app.browser_settings.backend, "native");
    assert_eq!(app.browser_settings.session_name_input, "desk");
    assert_eq!(app.browser_settings.native_chrome_path_input, "/Applications/Chrome");
    assert_eq!(app.browser_settings.computer_use_window_allowlist_input, "Vibe, Terminal");
    assert_eq!(app.browser_settings.computer_use_max_coordinate_x_input, "1440");
    assert_eq!(app.browser_settings.computer_use_max_coordinate_y_input, "900");
}

#[test]
fn update_loads_acp_agents_archived_sessions_and_preferences_conditionally() {
    let mut app = test_app();
    app.project_path = Some("/workspace/app".to_string());

    let _ = app.update(Message::BootstrapAcpAgentsLoaded(Ok(vec![
        "zeta".to_string(),
        "codex".to_string(),
        "alpha".to_string(),
    ])));
    assert_eq!(app.acp_agents, ["codex", "alpha", "zeta"]);

    let archived = std::collections::HashSet::from(["session-1".to_string()]);
    let _ = app.update(Message::BootstrapArchivedSessions(Ok(archived.clone())));
    assert_eq!(app.archived_session_ids, archived);

    let _ = app.update(Message::ProjectChatPreferencesLoaded(
        "/workspace/other".to_string(),
        Some(("ignored".to_string(), true, None)),
    ));
    assert_ne!(app.model, "ignored");

    let _ = app.update(Message::ProjectChatPreferencesLoaded(
        "/workspace/app".to_string(),
        Some(("matched".to_string(), false, Some("codex".to_string()))),
    ));
    assert_eq!(app.model, "matched");
    assert!(!app.auto_model);
    assert_eq!(app.acp_agent.as_deref(), Some("codex"));
}

#[test]
fn update_handles_external_apps_and_session_preview_errors() {
    let mut app = test_app();
    app.session_previews =
        std::collections::HashMap::from([("session".to_string(), "preview".to_string())]);

    let _ = app.update(Message::ExternalAppsLoaded(Ok(vw_gateway_client::ExternalAppsStateDto {
        platform: Some("windows".to_string()),
        apps: vec![("powershell".to_string(), true)],
    })));
    assert_eq!(app.open_external_platform, Some(RuntimePlatform::Windows));
    assert_eq!(app.open_external_app, ExternalOpenApp::PowerShell);

    let _ = app.update(Message::SessionPreviewsLoaded(Err("offline".to_string())));
    assert!(app.session_previews.is_empty());
    assert!(app.notifications.iter().any(|notification| {
        notification.message.contains("Failed to load session previews: offline")
    }));
}

#[test]
fn copy_feedback_expiration_only_clears_matching_hash_and_close_error_clears_message() {
    let mut app = test_app();
    app.last_copied_code_hash = Some(11);
    app.last_copy_time = Some(web_time::SystemTime::now());

    let _ = app.update(Message::CopyFeedbackExpired(12));
    assert_eq!(app.last_copied_code_hash, Some(11));
    assert!(app.last_copy_time.is_some());

    let _ = app.update(Message::CopyFeedbackExpired(11));
    assert_eq!(app.last_copied_code_hash, None);
    assert_eq!(app.last_copy_time, None);

    app.error_message = Some("boom".to_string());
    let _ = app.update(Message::CloseError);
    assert_eq!(app.error_message, None);
}
