#[test]
fn task_pool_api_module_is_available() {
    assert!(module_path!().ends_with("task_pool_api_tests"));
}
