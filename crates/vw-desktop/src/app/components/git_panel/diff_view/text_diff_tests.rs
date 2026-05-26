#[test]
fn task_716_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("text_diff_tests.rs"));
}
