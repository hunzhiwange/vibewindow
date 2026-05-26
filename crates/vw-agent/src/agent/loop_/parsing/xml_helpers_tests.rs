use serde_json::json;

use super::xml_helpers::{
    extract_xml_pairs, find_first_tag, is_xml_meta_tag, matching_tool_call_close_tag,
    parse_xml_tool_calls, tool_call_close_tags, tool_call_open_tags,
};

#[test]
fn xml_meta_tags_are_case_insensitive() {
    assert!(is_xml_meta_tag("Thinking"));
    assert!(is_xml_meta_tag("tool-call"));
    assert!(!is_xml_meta_tag("shell"));
}

#[test]
fn extract_xml_pairs_keeps_trimmed_inner_content() {
    let pairs = extract_xml_pairs("<shell> pwd </shell><note> ok </note>");

    assert_eq!(pairs, vec![("shell", "pwd"), ("note", "ok")]);
}

#[test]
fn parse_xml_tool_calls_supports_json_and_nested_args() {
    let calls = parse_xml_tool_calls(
        r#"<shell>{"command":"pwd"}</shell><memory_recall><query>abc</query></memory_recall>"#,
    )
    .expect("tool calls should parse");

    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments, json!({"command": "pwd"}));
    assert_eq!(calls[1].arguments["query"], "abc");
}

#[test]
fn tool_call_tags_have_matching_close_tags() {
    let (_, open) = find_first_tag("<invoke>{}</invoke>", tool_call_open_tags())
        .expect("open tag should be found");

    assert_eq!(matching_tool_call_close_tag(open), Some("</invoke>"));
    assert!(tool_call_close_tags().contains(&"</invoke>"));
}
