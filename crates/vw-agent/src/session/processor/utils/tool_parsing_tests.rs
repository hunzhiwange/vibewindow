use super::{is_valid_tool_name, parse_tool_at, query_has_any_tool_calls_with_allowed};
use std::collections::HashSet;

fn allowed(names: &[&str]) -> HashSet<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

#[test]
fn validates_tool_names_with_expected_character_set() {
    assert!(is_valid_tool_name("read_file"));
    assert!(is_valid_tool_name("Tool123"));
    assert!(!is_valid_tool_name(""));
    assert!(!is_valid_tool_name("bad-name"));
    assert!(!is_valid_tool_name("tool.name"));
}

#[test]
fn parses_inline_json_tool_call_only_when_allowed_and_valid() {
    let tools = allowed(&["read"]);
    let lines = ["/read {\"path\":\"Cargo.toml\"}"];

    assert_eq!(
        parse_tool_at(&lines, 0, &tools),
        Some(("read".to_string(), "{\"path\":\"Cargo.toml\"}".to_string(), 1))
    );
    assert!(!query_has_any_tool_calls_with_allowed("/write {}", &tools));
    assert_eq!(parse_tool_at(&["/read {not-json}"], 0, &tools), None);
}
