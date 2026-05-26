use super::*;

#[test]
fn script_embeds_boolean_flags_and_depth() {
    let script = snapshot_script(true, false, Some(3));
    assert!(script.contains("const interactiveOnly = true;"));
    assert!(script.contains("const compact = false;"));
    assert!(script.contains("const maxDepth = 3;"));
}

#[test]
fn script_uses_null_depth_when_unbounded() {
    let script = snapshot_script(false, true, None);
    assert!(script.contains("const interactiveOnly = false;"));
    assert!(script.contains("const compact = true;"));
    assert!(script.contains("const maxDepth = null;"));
}
