#[test]
fn task_703_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("gaps_tests.rs"));
}
