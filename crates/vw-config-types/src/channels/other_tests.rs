#[test]
fn task_618_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("other_tests.rs"));
}
