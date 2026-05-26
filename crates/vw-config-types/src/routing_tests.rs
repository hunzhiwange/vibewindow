#[test]
fn task_628_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("routing_tests.rs"));
}
