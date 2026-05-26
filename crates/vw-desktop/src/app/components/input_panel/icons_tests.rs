#[test]
fn task_732_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("icons_tests.rs"));
}
