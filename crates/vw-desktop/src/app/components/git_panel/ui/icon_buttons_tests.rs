#[test]
fn task_725_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("icon_buttons_tests.rs"));
}
