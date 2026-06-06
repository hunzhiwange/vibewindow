const SOURCE: &str = include_str!("acp.rs");

#[test]
fn acp_settings_message_tests_are_wired() {
    assert!(module_path!().contains("acp_tests"));
}

#[test]
fn acp_settings_update_keeps_gateway_paths() {
    for needle in
        ["load_acp_settings_snapshot_async", "set_global_acp_agent_enabled_async", "apply_snapshot"]
    {
        assert!(SOURCE.contains(needle), "missing ACP settings update needle: {needle}");
    }
}
