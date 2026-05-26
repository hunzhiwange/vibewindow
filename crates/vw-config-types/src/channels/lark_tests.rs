#[test]
fn task_617_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("lark_tests.rs"));
}
