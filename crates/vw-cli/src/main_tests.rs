#[test]
fn task_611_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("main_tests.rs"));
}
