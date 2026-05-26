#[test]
fn task_733_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("input_editor_tests.rs"));
}
