#[test]
fn expand_env_path_preserves_unknown_tokens() {
    let expanded = super::expand_env_path("$VIBE_WINDOW_UNKNOWN_TEST_TOKEN/cache");

    assert!(expanded.contains("$VIBE_WINDOW_UNKNOWN_TEST_TOKEN"));
}
