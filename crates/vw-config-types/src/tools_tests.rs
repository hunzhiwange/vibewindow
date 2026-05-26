#[test]
fn task_632_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("tools_tests.rs"));
}
