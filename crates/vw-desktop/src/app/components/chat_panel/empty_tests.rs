#[test]
fn task_648_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("empty_tests.rs"));
}
