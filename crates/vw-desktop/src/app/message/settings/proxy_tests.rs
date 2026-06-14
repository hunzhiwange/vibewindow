use super::*;
use crate::app::App;
use vw_config_types::proxy::ProxyScope;

fn app() -> App {
    App::new().0
}

#[test]
fn updates_proxy_fields_scope_and_help() {
    let mut app = app();
    app.proxy_settings.save_error = Some("old".to_string());
    let _ = update(&mut app, SettingsMessage::ProxyEnabledToggled(true));
    assert!(app.proxy_settings.enabled);
    assert!(app.proxy_settings.save_error.is_none());

    let _ = update(&mut app, SettingsMessage::ProxyScopeTextChanged("env".to_string()));
    assert_eq!(app.proxy_settings.scope, ProxyScope::Environment);
    let _ = update(&mut app, SettingsMessage::ProxyScopeTextChanged("service".to_string()));
    assert_eq!(app.proxy_settings.scope, ProxyScope::Services);
    let _ = update(&mut app, SettingsMessage::ProxyScopeTextChanged("unknown".to_string()));
    assert_eq!(app.proxy_settings.scope, ProxyScope::Vibewindow);

    let _ = update(&mut app, SettingsMessage::ProxyHttpChanged(" http ".to_string()));
    let _ = update(&mut app, SettingsMessage::ProxyHttpsChanged(" https ".to_string()));
    let _ = update(&mut app, SettingsMessage::ProxyAllChanged(" all ".to_string()));
    let _ =
        update(&mut app, SettingsMessage::ProxyNoProxyChanged("localhost,127.0.0.1".to_string()));
    let _ = update(&mut app, SettingsMessage::ProxyServicesChanged("openai\ngithub".to_string()));
    assert_eq!(app.proxy_settings.http_proxy, " http ");
    assert_eq!(app.proxy_settings.services_input, "openai\ngithub");

    let _ = update(&mut app, SettingsMessage::ProxyHelpOpen);
    assert!(app.proxy_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::ProxyHelpClose);
    assert!(!app.proxy_settings.show_help_modal);
}
