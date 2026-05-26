#[test]
fn app_update_tests_module_is_wired() {
    assert!(module_path!().ends_with("app_update_tests"));
}
