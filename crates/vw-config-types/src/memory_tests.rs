#[test]
fn task_623_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("memory_tests.rs"));
}
