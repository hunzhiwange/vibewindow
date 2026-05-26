#[test]
fn task_737_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("send_controls_tests.rs"));
}
