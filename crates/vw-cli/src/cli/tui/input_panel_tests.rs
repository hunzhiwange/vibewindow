#[test]
fn task_604_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("input_panel_tests.rs"));
}
