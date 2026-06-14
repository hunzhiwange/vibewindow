use super::*;
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_gateway_tests_are_wired() {
    assert!(module_path!().contains("system_settings_gateway_tests"));
}

#[test]
fn paired_token_list_height_handles_empty_single_and_capped_rows() {
    assert_eq!(paired_token_list_max_height(0), 0.0);
    assert_eq!(paired_token_list_max_height(1), GATEWAY_PAIRED_TOKEN_ROW_HEIGHT);

    let expected = 10.0 * GATEWAY_PAIRED_TOKEN_ROW_HEIGHT + 9.0 * GATEWAY_PAIRED_TOKEN_ROW_SPACING;
    assert_eq!(paired_token_list_max_height(10), expected);
    assert_eq!(paired_token_list_max_height(11), expected);
}

#[test]
fn paired_token_scrollbar_width_is_four_pixels() {
    assert_eq!(GATEWAY_PAIRED_TOKEN_SCROLLBAR_WIDTH, 4);
}

#[test]
fn view_builds_empty_and_populated_gateway_states() {
    let app = test_app();
    assert_eq!(app.gateway_settings.active_tab, GatewaySettingsTab::Config);
    let _ = view(&app);

    let mut populated = test_app();
    populated.gateway_settings.active_tab = GatewaySettingsTab::Skeys;
    populated.gateway_settings.host_input = "0.0.0.0".to_string();
    populated.gateway_settings.auth_enabled = true;
    populated.gateway_settings.allow_public_bind = true;
    populated.gateway_settings.trust_forwarded_headers = true;
    populated.gateway_settings.skeys = vec![vw_config_types::gateway::GatewaySkey {
        enabled: true,
        skey: None,
        skey_hash: "a".repeat(64),
        masked_skey: "sk-aaaaaaaaaaaaa***************aaaaaaaaa".to_string(),
        name: "abcd***wxyz".to_string(),
        expires_at: None,
    }];
    populated.gateway_settings.new_skey_name_input = "pending-skey".to_string();
    populated.gateway_settings.new_skey_expires_at_input = "2026-12-31".to_string();
    populated.gateway_settings.new_skey_calendar_month = "2026-12".to_string();
    populated.gateway_settings.new_skey_calendar_open = true;
    populated.gateway_settings.last_created_skey = Some("sk-test".to_string());
    populated.gateway_settings.last_created_skey_copied = true;
    populated.gateway_settings.node_control_enabled = true;
    populated.gateway_settings.node_control_auth_token_input = "node-secret".to_string();
    populated.gateway_settings.node_control_allowed_node_ids_input = "node-a\nnode-b".to_string();
    populated.gateway_settings.save_error = Some("gateway save failed".to_string());
    let _ = view(&populated);
}

#[test]
fn overlays_return_base_or_help_modal_for_gateway() {
    let app = test_app();
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&app, base);

    let mut with_help = test_app();
    with_help.gateway_settings.show_help_modal = true;
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&with_help, base);
}
