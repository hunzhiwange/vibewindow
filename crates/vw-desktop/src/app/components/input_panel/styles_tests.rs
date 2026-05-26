#[test]
fn task_738_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("styles_tests.rs"));
}
