#[test]
fn binary_entry_test_module_is_loaded() {
    let path = String::from(module_path!());
    assert!(path.contains("vibe_agent_tests"));
}
