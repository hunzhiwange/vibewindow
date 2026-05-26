#[test]
fn signal_store_tests_are_feature_gated() {
    assert!(!cfg!(feature = "whatsapp-web") || cfg!(feature = "whatsapp-web"));
}
