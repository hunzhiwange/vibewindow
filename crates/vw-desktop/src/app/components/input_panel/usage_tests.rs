#[test]
fn task_742_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("usage_tests.rs"));
}
