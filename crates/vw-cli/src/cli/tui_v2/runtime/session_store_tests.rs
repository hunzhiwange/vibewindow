#[test]
fn session_store_tests_module_is_loaded() {
    let path = String::from(module_path!());
    assert!(path.contains("tests"));
}
