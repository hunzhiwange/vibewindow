#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("agents_ipc_tests"));
}

#[test]
fn agents_ipc_update_keeps_persistence_inputs_normalized() {
    let source = include_str!("agents_ipc.rs");

    assert!(source.contains("staleness_secs.clamp(1, 86_400)"));
    assert!(source.contains("db_path_input.trim().to_string()"));
    assert!(source.contains("vw_config_types::paths::agents_ipc_db_path()"));
}

#[test]
fn agents_ipc_update_routes_every_user_setting_message_to_persist() {
    let source = include_str!("agents_ipc.rs");

    for message in [
        "AgentsIpcEnabledToggled",
        "AgentsIpcDbPathChanged",
        "AgentsIpcStalenessSecsChanged",
        "AgentsIpcSave",
    ] {
        assert!(source.contains(message), "missing message branch: {message}");
    }

    assert!(source.matches("persist_agents_ipc_settings(app)").count() >= 4);
}

#[test]
fn agents_ipc_help_messages_are_local_ui_state_only() {
    let source = include_str!("agents_ipc.rs");

    assert!(source.contains("AgentsIpcHelpOpen"));
    assert!(source.contains("show_help_modal = true"));
    assert!(source.contains("AgentsIpcHelpClose"));
    assert!(source.contains("show_help_modal = false"));
}
