#[test]
fn task_608_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("estop_tests.rs"));
}
