use super::*;

#[test]
fn exported_handlers_are_available() {
    let _ = handle_api_integrations;
    let _ = handle_api_integrations_settings;
    let _ = handle_api_integration_credentials_put;
}
