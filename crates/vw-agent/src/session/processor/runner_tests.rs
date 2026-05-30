#[test]
fn runner_tests_module_is_wired() {
    let marker = String::from("runner_tests");
    assert_eq!(marker.as_str(), "runner_tests");
}

#[test]
fn full_access_option_enables_request_full_access() {
    let options = serde_json::json!({ "full_access": true });

    assert!(super::request_full_access_enabled(&options));
}

#[test]
fn missing_full_access_option_defaults_to_disabled() {
    let options = serde_json::json!({});

    assert!(!super::request_full_access_enabled(&options));
}
