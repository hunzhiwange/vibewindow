use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn browser_parsers_normalize_inputs() {
    assert_eq!(parse_csv_lines("a, b\nc"), vec!["a", "b", "c"]);
    assert_eq!(normalize_browser_open("new-tab"), "new_tab");
    assert_eq!(normalize_browser_open("???"), "default");
    assert_eq!(normalize_backend("native"), "native");
    assert_eq!(normalize_backend("rust_native"), "native");
    assert_eq!(normalize_backend("unexpected"), "agent_browser");
    assert_eq!(parse_timeout_ms("").unwrap(), 15_000);
    assert!(parse_timeout_ms("0").is_err());
    assert_eq!(parse_optional_i64("42", "field").unwrap(), Some(42));
    assert!(parse_optional_i64("nope", "field").is_err());
}

#[test]
fn browser_update_sets_errors_and_normalized_values() {
    let mut app = app();
    app.browser_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::Browser(BrowserMessage::Refresh));
    assert!(app.browser_settings.save_error.is_none());

    let _ = update(
        &mut app,
        SettingsMessage::Browser(BrowserMessage::AllowedDomainsChanged(
            " example.com,\napi.example.com ".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Browser(BrowserMessage::BrowserOpenChanged("new-window".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Browser(BrowserMessage::BackendChanged("rust_native".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Browser(BrowserMessage::ComputerUseTimeoutMsChanged("0".to_string())),
    );

    assert_eq!(app.browser_settings.allowed_domains_input, " example.com,\napi.example.com ");
    assert_eq!(app.browser_settings.browser_open, "new_window");
    assert_eq!(app.browser_settings.backend, "native");
    assert!(app.browser_settings.save_error.as_deref().unwrap_or("").contains("timeout_ms"));

    let _ = update(
        &mut app,
        SettingsMessage::Browser(BrowserMessage::ComputerUseTimeoutMsChanged("5000".to_string())),
    );
    assert!(app.browser_settings.save_error.is_none());
}
