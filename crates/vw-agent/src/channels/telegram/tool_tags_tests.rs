use super::tool_tags::strip_tool_call_tags;

#[test]
fn strip_tool_call_tags_removes_current_and_legacy_tags() {
    assert_eq!(strip_tool_call_tags("<tool_call>secret</tool_call>visible"), "visible");
    assert_eq!(strip_tool_call_tags("erte {\"tool\":\"x\"} ttrivisible"), "visible");
}

#[test]
fn strip_tool_call_tags_removes_legacy_array_json_and_multiple_tags() {
    let input = "before erte [{\"tool\":\"a\"}] ttri middle erte {\"tool\":\"b\"} ttri after";

    assert_eq!(strip_tool_call_tags(input), "before  middle  after");
}

#[test]
fn strip_tool_call_tags_preserves_legacy_prefix_when_json_is_missing_or_invalid() {
    assert_eq!(strip_tool_call_tags("hello erte not-json ttri"), "hello erte not-json ttri");
    assert_eq!(strip_tool_call_tags("hello erte {broken} ttri"), "hello erte {broken} ttri");
}

#[test]
fn strip_tool_call_tags_preserves_text_when_legacy_end_marker_is_wrong() {
    assert_eq!(
        strip_tool_call_tags("alpha erte {\"tool\":\"x\"} nope omega"),
        "alpha erte {\"tool\":\"x\"} nope omega"
    );
}

#[test]
fn strip_tool_call_tags_drops_complete_legacy_tag_at_end_of_input() {
    assert_eq!(strip_tool_call_tags("visible erte {\"tool\":\"x\"} ttri"), "visible");
}

#[test]
fn strip_tool_call_tags_handles_incomplete_legacy_tag_at_end() {
    assert_eq!(strip_tool_call_tags("visible erte {\"tool\":\"x\"}"), "visible");
    assert_eq!(strip_tool_call_tags("visible erte"), "visible erte");
}

#[test]
fn strip_tool_call_tags_trims_trailing_newlines_after_standard_cleanup() {
    assert_eq!(strip_tool_call_tags("visible\n<tool_call>{}</tool_call>\n"), "visible");
}
