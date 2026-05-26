#[test]
fn task_605_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("layout_tests.rs"));
}
