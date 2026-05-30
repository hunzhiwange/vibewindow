use super::*;

#[test]
fn contains_path_is_false_without_context() {
    assert!(!contains_path("/tmp/outside"));
}
