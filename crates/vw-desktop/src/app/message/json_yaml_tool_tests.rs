#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("json_yaml_tool_tests"));
}
