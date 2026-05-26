#[test]
fn task_721_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("ops_tests.rs"));
}
