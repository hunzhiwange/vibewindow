#[test]
fn task_749_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("overlay_element_tests.rs"));
}
