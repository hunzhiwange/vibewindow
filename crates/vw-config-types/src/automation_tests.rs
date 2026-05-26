#[test]
fn task_613_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("automation_tests.rs"));
}
