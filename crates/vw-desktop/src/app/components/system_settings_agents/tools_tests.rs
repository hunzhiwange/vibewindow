use super::tools::tool_in_preset;

#[test]
fn tool_in_preset_rejects_unknown_preset() {
    assert!(!tool_in_preset("shell", "missing"));
}
