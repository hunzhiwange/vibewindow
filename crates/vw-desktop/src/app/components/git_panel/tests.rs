#[test]
fn task_720_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("tests.rs"));
}
