#[test]
fn structured_stream_tests_module_is_loaded() {
    let path = String::from(module_path!());
    assert!(path.contains("tests"));
}
