#[test]
fn todos_tests_module_is_wired() {
    assert!(module_path!().ends_with("todos_tests"));
}
