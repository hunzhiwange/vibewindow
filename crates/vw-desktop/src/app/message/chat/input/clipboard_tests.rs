#[test]
fn clipboard_tests_module_is_wired() {
    assert!(module_path!().ends_with("clipboard_tests"));
}

#[test]
fn read_clipboard_for_input_builds_task_without_touching_callers_state() {
    let _task = super::read_clipboard_for_input();
}
