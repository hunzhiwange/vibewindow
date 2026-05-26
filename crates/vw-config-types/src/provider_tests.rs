#[test]
fn task_625_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("provider_tests.rs"));
}
