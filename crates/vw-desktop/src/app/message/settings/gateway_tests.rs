use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn gateway_helpers_normalize_and_clamp() {
    assert_eq!(normalize_host(""), "127.0.0.1");
    assert_eq!(normalize_host(" 0.0.0.0 "), "0.0.0.0");
    assert_eq!(clamp_token_limit(0), 1);
    assert_eq!(clamp_token_limit(200_000), 100_000);
    assert_eq!(
        mask_skey_for_display("sk-4234324234324abcdefghijklmnop111111111"),
        "sk-4234324234324***************111111111"
    );
}

#[test]
fn gateway_update_generates_skeys_and_handles_help() {
    let mut app = app();
    app.gateway_settings.skeys.clear();
    app.gateway_settings.save_error = Some("old".to_string());

    assert_eq!(app.gateway_settings.active_tab, crate::app::state::GatewaySettingsTab::Config);
    let _ = update(
        &mut app,
        SettingsMessage::Gateway(GatewayMessage::TabSelected(
            crate::app::state::GatewaySettingsTab::Skeys,
        )),
    );
    assert_eq!(app.gateway_settings.active_tab, crate::app::state::GatewaySettingsTab::Skeys);

    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::Refresh));
    assert!(app.gateway_settings.save_error.is_none());

    let _ = update(
        &mut app,
        SettingsMessage::Gateway(GatewayMessage::NewSkeyNameChanged(" laptop ".to_string())),
    );
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::NewSkeyCalendarToggled));
    assert!(app.gateway_settings.new_skey_calendar_open);
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::NewSkeyCalendarClosed));
    assert!(!app.gateway_settings.new_skey_calendar_open);
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::NewSkeyCalendarToggled));
    assert!(app.gateway_settings.new_skey_calendar_open);
    let _ = update(
        &mut app,
        SettingsMessage::Gateway(GatewayMessage::NewSkeyExpiresDateSelected(
            "2026-12-31".to_string(),
        )),
    );
    assert!(!app.gateway_settings.new_skey_calendar_open);
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::AddSkey));
    assert_eq!(app.gateway_settings.skeys.len(), 1);
    assert!(app.gateway_settings.skeys[0].enabled);
    assert_eq!(app.gateway_settings.skeys[0].name, "laptop");
    assert_eq!(app.gateway_settings.skeys[0].expires_at.as_deref(), Some("2026-12-31T23:59:59Z"));
    assert!(app.gateway_settings.skeys[0].skey.is_none());
    assert!(app.gateway_settings.new_skey_name_input.is_empty());
    let generated = app.gateway_settings.last_created_skey.clone().unwrap();
    assert!(generated.starts_with("sk-"));
    assert_eq!(app.gateway_settings.skeys[0].skey_hash, hash_skey(&generated));
    assert_eq!(app.gateway_settings.skeys[0].masked_skey, mask_skey_for_display(&generated));

    let _ =
        update(&mut app, SettingsMessage::Gateway(GatewayMessage::SkeyEnabledToggled(0, false)));
    assert!(!app.gateway_settings.skeys[0].enabled);

    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::CopyLastCreatedSkey));
    assert!(app.gateway_settings.last_created_skey_copied);
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::ClearLastCreatedSkeyCopied));
    assert!(!app.gateway_settings.last_created_skey_copied);

    let _ = update(
        &mut app,
        SettingsMessage::Gateway(GatewayMessage::NewSkeyNameChanged(" ".to_string())),
    );
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::AddSkey));
    assert!(app.gateway_settings.save_error.as_deref().unwrap_or("").contains("不能为空"));

    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::HostChanged("".to_string())));
    let _ =
        update(&mut app, SettingsMessage::Gateway(GatewayMessage::IdempotencyTtlSecsChanged(0)));
    assert_eq!(app.gateway_settings.host_input, "127.0.0.1");
    assert_eq!(app.gateway_settings.idempotency_ttl_secs, 1);

    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::HelpOpen));
    assert!(app.gateway_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::Gateway(GatewayMessage::HelpClose));
    assert!(!app.gateway_settings.show_help_modal);
}

#[test]
fn gateway_service_command_completion_updates_status_fields() {
    let mut app = app();
    app.gateway_settings.service_action_running = Some("status".to_string());

    let _ = update(
        &mut app,
        SettingsMessage::Gateway(GatewayMessage::ServiceCommandCompleted(
            "status".to_string(),
            Ok("Service: running".to_string()),
        )),
    );
    assert!(app.gateway_settings.service_action_running.is_none());
    assert_eq!(app.gateway_settings.service_action_output.as_deref(), Some("Service: running"));
    assert!(app.gateway_settings.save_error.is_none());

    app.gateway_settings.service_action_running = Some("start".to_string());
    let _ = update(
        &mut app,
        SettingsMessage::Gateway(GatewayMessage::ServiceCommandCompleted(
            "start".to_string(),
            Err("not installed".to_string()),
        )),
    );
    assert!(app.gateway_settings.service_action_running.is_none());
    assert!(app.gateway_settings.service_action_output.is_none());
    assert!(
        app.gateway_settings.save_error.as_deref().unwrap_or_default().contains("not installed")
    );
}
