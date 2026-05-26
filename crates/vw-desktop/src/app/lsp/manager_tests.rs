#[test]
fn manager_tests_module_is_wired() {
    assert!(module_path!().ends_with("manager_tests"));
}
