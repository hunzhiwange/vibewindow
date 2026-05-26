#[test]
fn task_620_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("config_tests.rs"));
}
