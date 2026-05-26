#[test]
fn run_windows_tests_module_is_wired() {
    assert!(module_path!().ends_with("run_windows_tests"));
}
