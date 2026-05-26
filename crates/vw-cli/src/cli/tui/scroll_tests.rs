#[test]
fn task_607_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("scroll_tests.rs"));
}
