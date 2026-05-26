#[test]
fn message_tests_module_is_wired() {
    let marker = String::from("message_tests");
    assert_eq!(marker.as_str(), "message_tests");
}
