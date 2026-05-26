#[test]
fn app_subscription_tests_module_is_wired() {
    assert!(module_path!().ends_with("app_subscription_tests"));
}
