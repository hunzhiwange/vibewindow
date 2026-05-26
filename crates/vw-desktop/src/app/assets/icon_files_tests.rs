#[test]
fn task_638_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("icon_files_tests.rs"));
}
