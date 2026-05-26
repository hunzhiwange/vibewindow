#[test]
fn task_741_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("usage_button_tests.rs"));
}
