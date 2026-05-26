#[test]
fn rename_tests_module_is_wired() {
    assert!(module_path!().ends_with("rename_tests"));
}
