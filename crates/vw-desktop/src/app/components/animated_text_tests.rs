#[test]
fn task_647_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("animated_text_tests.rs"));
}
