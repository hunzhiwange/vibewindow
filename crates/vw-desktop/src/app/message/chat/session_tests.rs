#[test]
fn session_tests_module_is_wired() {
    assert!(module_path!().ends_with("session_tests"));
}
