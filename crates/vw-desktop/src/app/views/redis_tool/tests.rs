#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("tests"));
}

#[test]
fn redis_tool_width_keeps_desktop_workspace_available() {
    assert!(super::REDIS_TOOL_MAX_WIDTH >= 1180.0);
}
