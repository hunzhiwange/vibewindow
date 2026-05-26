#[test]
fn task_722_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("ui_tests.rs"));
}
