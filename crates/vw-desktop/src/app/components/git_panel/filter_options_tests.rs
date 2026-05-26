#[test]
fn task_717_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("filter_options_tests.rs"));
}
