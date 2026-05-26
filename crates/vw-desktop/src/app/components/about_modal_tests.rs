#[test]
fn task_646_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("about_modal_tests.rs"));
}
