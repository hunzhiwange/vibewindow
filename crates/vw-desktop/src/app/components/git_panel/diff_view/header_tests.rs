#[test]
fn task_711_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("header_tests.rs"));
}
