#[test]
fn task_710_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("start_end_tests.rs"));
}
