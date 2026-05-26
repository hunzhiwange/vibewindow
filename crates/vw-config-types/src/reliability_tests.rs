#[test]
fn task_627_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("reliability_tests.rs"));
}
