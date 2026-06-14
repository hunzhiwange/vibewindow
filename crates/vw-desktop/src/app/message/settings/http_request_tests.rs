use super::*;
use crate::app::App;
use vw_config_types::tools::HttpRequestConfig;

fn app() -> App {
    App::new().0
}

#[test]
fn http_request_normalizes_domains_and_handles_duplicates() {
    assert_eq!(normalize_domain(" Example.COM "), "example.com");

    let mut app = app();
    let _ = update(
        &mut app,
        SettingsMessage::HttpRequest(HttpRequestMessage::NewAllowedDomainChanged(
            "Example.com,\nAPI.EXAMPLE.COM".to_string(),
        )),
    );
    let _ = update(&mut app, SettingsMessage::HttpRequest(HttpRequestMessage::AddAllowedDomain));
    assert_eq!(
        app.http_request_settings.allowed_domains,
        vec!["example.com".to_string(), "api.example.com".to_string()]
    );
    assert!(app.http_request_settings.new_allowed_domain_input.is_empty());

    let _ = update(
        &mut app,
        SettingsMessage::HttpRequest(HttpRequestMessage::NewAllowedDomainChanged(
            "example.com".to_string(),
        )),
    );
    let _ = update(&mut app, SettingsMessage::HttpRequest(HttpRequestMessage::AddAllowedDomain));
    assert!(app.http_request_settings.save_error.as_deref().unwrap_or("").contains("已存在"));
}

#[test]
fn http_request_update_resets_user_agent_and_removes_domains() {
    let mut app = app();
    app.http_request_settings.allowed_domains = vec!["example.com".to_string()];
    app.http_request_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::HttpRequest(HttpRequestMessage::Refresh));
    assert!(app.http_request_settings.save_error.is_none());

    let _ = update(
        &mut app,
        SettingsMessage::HttpRequest(HttpRequestMessage::UserAgentChanged("   ".to_string())),
    );
    assert_eq!(app.http_request_settings.user_agent, HttpRequestConfig::default().user_agent);

    let _ =
        update(&mut app, SettingsMessage::HttpRequest(HttpRequestMessage::RemoveAllowedDomain(0)));
    assert!(app.http_request_settings.allowed_domains.is_empty());
}
