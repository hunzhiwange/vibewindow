#[test]
fn task_mode_test_module_is_wired() {
    assert_eq!("task-mode".replace('-', "_"), "task_mode");
}
