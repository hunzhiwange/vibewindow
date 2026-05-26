#[test]
fn tool_detail_tests_module_is_wired() {
    assert!(module_path!().ends_with("tool_detail_tests"));
}
