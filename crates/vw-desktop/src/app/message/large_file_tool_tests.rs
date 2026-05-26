#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("large_file_tool_tests"));
}
