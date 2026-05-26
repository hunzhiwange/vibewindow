#[test]
fn task_702_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("added_deleted_tests.rs"));
}
