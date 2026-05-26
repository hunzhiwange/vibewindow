#[test]
fn question_poller_tests_module_is_loaded() {
    let path = String::from(module_path!());
    assert!(path.contains("tests"));
}
