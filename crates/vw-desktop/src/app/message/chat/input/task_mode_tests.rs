#[test]
fn task_mode_tests_module_is_wired() {
    assert!(module_path!().ends_with("task_mode_tests"));
}
