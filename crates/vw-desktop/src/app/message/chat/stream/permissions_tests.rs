#[test]
fn permissions_tests_module_is_wired() {
    assert!(module_path!().ends_with("permissions_tests"));
}
