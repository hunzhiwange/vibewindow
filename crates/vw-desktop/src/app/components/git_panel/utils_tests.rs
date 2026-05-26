#[test]
fn task_727_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("utils_tests.rs"));
}
