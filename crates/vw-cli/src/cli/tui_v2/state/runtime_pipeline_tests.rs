#[test]
fn task_601_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("runtime_pipeline_tests.rs"));
}
