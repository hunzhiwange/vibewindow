#[test]
fn git_diff_tests_module_is_wired() {
    assert!(module_path!().ends_with("git_diff_tests"));
}
