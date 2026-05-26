#[test]
fn task_630_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("security_tests.rs"));
}
