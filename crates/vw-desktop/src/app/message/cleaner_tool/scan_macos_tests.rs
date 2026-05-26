#[test]
fn scan_macos_tests_module_is_wired() {
    assert!(module_path!().ends_with("scan_macos_tests"));
}
