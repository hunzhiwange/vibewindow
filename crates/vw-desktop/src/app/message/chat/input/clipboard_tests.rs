#[test]
fn clipboard_tests_module_is_wired() {
    assert!(module_path!().ends_with("clipboard_tests"));
}
