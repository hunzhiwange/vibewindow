#[test]
fn task_622_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("hooks_tests.rs"));
}
