#[test]
fn task_728_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("input_mention_highlighter_tests.rs"));
}
