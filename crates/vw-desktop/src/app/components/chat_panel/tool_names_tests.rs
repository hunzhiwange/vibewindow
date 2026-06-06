use super::tool_names::{canonical_tool_name, is_compact_tool_call_trace, is_known_tool_name};

#[test]
fn canonical_tool_name_normalizes_common_aliases() {
    assert_eq!(canonical_tool_name(" shell "), "bash");
    assert_eq!(canonical_tool_name("execute"), "bash");
    assert_eq!(canonical_tool_name("Terminal"), "bash");
    assert_eq!(canonical_tool_name("ask_user_question"), "question");
    assert_eq!(canonical_tool_name("plan_enter"), "enter_plan_mode");
}

#[test]
fn known_tool_name_uses_canonical_name() {
    assert!(is_known_tool_name("shell"));
    assert!(is_known_tool_name("execute"));
    assert!(is_known_tool_name("todo_write"));
    assert!(!is_known_tool_name("unknown_tool"));
}

#[test]
fn compact_tool_call_trace_requires_known_function_shape() {
    assert!(is_compact_tool_call_trace("tool bash(command=\"ls\")"));
    assert!(!is_compact_tool_call_trace("tool unknown_tool()"));
    assert!(!is_compact_tool_call_trace("tool bash"));
}
