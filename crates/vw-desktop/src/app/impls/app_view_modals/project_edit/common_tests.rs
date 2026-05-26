#[test]
fn common_tests_module_is_wired() {
    assert!(module_path!().ends_with("common_tests"));
}
