#[test]
fn task_602_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("selectors_tests.rs"));
}
