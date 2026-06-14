#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("tests"));
}

#[test]
fn update_delegates_provider_messages() {
    let (mut app, _) = crate::app::App::new();

    let _ = super::update(&mut app, super::SettingsMessage::ProviderConnectClose);

    assert!(app.provider_settings.connect_modal.is_none());
}
