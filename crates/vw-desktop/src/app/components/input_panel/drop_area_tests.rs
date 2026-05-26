#[test]
fn task_729_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("drop_area_tests.rs"));
}
