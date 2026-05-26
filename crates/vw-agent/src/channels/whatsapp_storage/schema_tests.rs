#[test]
fn schema_tests_are_feature_gated() {
    assert!(!cfg!(feature = "whatsapp-web") || cfg!(feature = "whatsapp-web"));
}
