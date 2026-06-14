use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn gateway_client_helpers_normalize_and_select() {
    let mut app = app();

    assert_eq!(normalize_host(""), "127.0.0.1");
    assert_eq!(normalize_name(" ", 1), "网关 2");
    assert!(selected_index(&app).is_some());

    app.gateway_client_settings.name_input = " primary ".to_string();
    app.gateway_client_settings.host_input = "".to_string();
    sync_selected_server_from_inputs(&mut app);
    assert_eq!(app.gateway_client_settings.servers[0].name, "primary");
    assert_eq!(app.gateway_client_settings.servers[0].host, "127.0.0.1");

    app.gateway_client_settings.servers[0].host = "0.0.0.0".to_string();
    load_selected_server_into_inputs(&mut app);
    assert_eq!(app.gateway_client_settings.host_input, "0.0.0.0");
}

#[test]
fn gateway_client_update_adds_selects_and_removes_servers() {
    let mut app = app();
    let initial_len = app.gateway_client_settings.servers.len();
    let original = app.gateway_client_settings.selected_server_id.clone();

    let _ = update(&mut app, SettingsMessage::GatewayClient(GatewayClientMessage::AddServer));
    assert_eq!(app.gateway_client_settings.servers.len(), initial_len + 1);
    assert_ne!(app.gateway_client_settings.selected_server_id, original);

    let new_id = app.gateway_client_settings.selected_server_id.clone();
    let _ = update(
        &mut app,
        SettingsMessage::GatewayClient(GatewayClientMessage::RemoveServerRequested(new_id.clone())),
    );
    assert_eq!(
        app.gateway_client_settings.pending_remove_server_id.as_deref(),
        Some(new_id.as_str())
    );
    let _ = update(
        &mut app,
        SettingsMessage::GatewayClient(GatewayClientMessage::RemoveServerCanceled),
    );
    assert!(app.gateway_client_settings.pending_remove_server_id.is_none());

    let _ = update(
        &mut app,
        SettingsMessage::GatewayClient(GatewayClientMessage::RemoveServerRequested(new_id.clone())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::GatewayClient(GatewayClientMessage::RemoveServerConfirmed(new_id)),
    );
    assert_eq!(app.gateway_client_settings.servers.len(), initial_len);

    let _ = update(&mut app, SettingsMessage::GatewayClient(GatewayClientMessage::HelpOpen));
    assert!(app.gateway_client_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::GatewayClient(GatewayClientMessage::HelpClose));
    assert!(!app.gateway_client_settings.show_help_modal);
}
