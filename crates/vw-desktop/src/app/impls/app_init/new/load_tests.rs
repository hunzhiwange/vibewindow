#[test]
fn load_tests_module_is_wired() {
    assert!(module_path!().ends_with("load_tests"));
}
