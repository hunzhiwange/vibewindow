const SOURCE: &str = include_str!("system_settings_acp.rs");

#[test]
fn system_settings_acp_tests_are_wired() {
    assert!(module_path!().contains("system_settings_acp_tests"));
}

#[test]
fn system_settings_acp_page_keeps_core_actions() {
    for needle in [
        "AcpMessage::Refresh",
        "AcpMessage::SetEnabled",
        "settings_success_banner",
        "\"openclaw\"",
        "GitHub Copilot",
    ] {
        assert!(SOURCE.contains(needle), "missing ACP page needle: {needle}");
    }
}
