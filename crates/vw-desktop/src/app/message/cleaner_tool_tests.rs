#[test]
fn cleaner_tool_tests_module_is_wired() {
    assert!(module_path!().ends_with("cleaner_tool_tests"));
}
