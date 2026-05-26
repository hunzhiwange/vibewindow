#[test]
fn task_723_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("disabled_buttons_tests.rs"));
}
