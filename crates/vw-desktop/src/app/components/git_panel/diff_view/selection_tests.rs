#[test]
fn task_714_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("selection_tests.rs"));
}
