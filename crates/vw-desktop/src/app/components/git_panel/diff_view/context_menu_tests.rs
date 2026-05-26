#[test]
fn task_701_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("context_menu_tests.rs"));
}
