#[test]
fn run_macos_tests_module_is_wired() {
    assert!(module_path!().ends_with("run_macos_tests"));
}
