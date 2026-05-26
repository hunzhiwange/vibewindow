#[test]
fn startup_tests_module_is_wired() {
    assert!(module_path!().ends_with("startup_tests"));
}
