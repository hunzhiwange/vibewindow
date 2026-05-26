#[test]
fn settings_builders_tests_module_is_wired() {
    assert!(module_path!().ends_with("settings_builders_tests"));
}
