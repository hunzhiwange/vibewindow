#[test]
fn cleaner_scan_module_is_wired() {
    assert!(module_path!().ends_with("scan_tests"));
}
