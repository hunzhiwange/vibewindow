#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("agent_tests"));
}

#[test]
fn session_runtime_defaults_to_full_access() {
    let runtime = super::SessionRuntimeState::new();

    assert!(runtime.full_access_enabled);
}
