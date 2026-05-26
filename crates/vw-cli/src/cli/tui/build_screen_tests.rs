#[test]
fn task_603_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("build_screen_tests.rs"));
}
