#[test]
fn context_tests_module_is_wired() {
    assert!(module_path!().ends_with("context_tests"));
}
