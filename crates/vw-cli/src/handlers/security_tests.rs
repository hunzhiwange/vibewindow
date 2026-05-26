#[test]
fn task_609_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("security_tests.rs"));
}
