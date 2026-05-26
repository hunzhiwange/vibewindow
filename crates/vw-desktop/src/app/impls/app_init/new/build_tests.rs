#[test]
fn build_tests_module_is_wired() {
    assert!(module_path!().ends_with("build_tests"));
}
