#[test]
fn task_641_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("images_tests.rs"));
}
