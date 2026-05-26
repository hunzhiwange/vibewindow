use super::tool_tags::strip_tool_call_tags;

#[test]
fn strip_tool_call_tags_removes_current_and_legacy_tags() {
    assert_eq!(strip_tool_call_tags("<tool_call>secret</tool_call>visible"), "visible");
    assert_eq!(strip_tool_call_tags("erte {\"tool\":\"x\"} ttrivisible"), "visible");
}
