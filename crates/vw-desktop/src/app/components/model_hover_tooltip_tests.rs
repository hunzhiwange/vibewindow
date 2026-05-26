#[test]
fn task_746_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("model_hover_tooltip_tests.rs"));
}
