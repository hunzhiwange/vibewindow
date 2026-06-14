use super::*;
use crate::app::App;
use crate::app::message::gateway_health::server_health_key;
use crate::app::state::GatewayClientServerDraft;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_gateway_client_tests_are_wired() {
    assert!(module_path!().contains("system_settings_gateway_client_tests"));
}

#[test]
fn view_builds_empty_single_and_multi_server_states() {
    let mut empty = test_app();
    empty.gateway_client_settings.servers.clear();
    empty.gateway_client_settings.save_error = Some("client save failed".to_string());
    let _ = view(&empty);

    let app = test_app();
    let _ = view(&app);

    let mut multi = test_app();
    let extra = GatewayClientServerDraft {
        id: "remote".to_string(),
        name: "Remote".to_string(),
        host: "https://gateway.example.com".to_string(),
        port: 443,
        skey: "skey".to_string(),
    };
    if let Some(key) = server_health_key(&extra) {
        multi.gateway_client_settings.health.insert(key, true);
    }
    multi.gateway_client_settings.selected_server_id = extra.id.clone();
    multi.gateway_client_settings.name_input = extra.name.clone();
    multi.gateway_client_settings.host_input = extra.host.clone();
    multi.gateway_client_settings.port = extra.port;
    multi.gateway_client_settings.skey_input = extra.skey.clone();
    multi.gateway_client_settings.servers.push(extra);
    let _ = view(&multi);
}

#[test]
fn overlays_cover_remove_confirmation_missing_server_and_help_modal() {
    let mut app = test_app();
    app.gateway_client_settings.pending_remove_server_id =
        Some(app.gateway_client_settings.selected_server_id.clone());
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&app, base);

    app.gateway_client_settings.pending_remove_server_id = Some("missing".to_string());
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&app, base);

    app.gateway_client_settings.pending_remove_server_id = None;
    app.gateway_client_settings.show_help_modal = true;
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&app, base);
}
