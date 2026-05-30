use super::*;
use serde_json::json;

#[test]
fn parse_browser_action_preserves_required_fields() {
    let action = parse_browser_action("fill", &json!({"selector": "#name", "value": "Ada"}))
        .expect("fill action should parse");

    match action {
        BrowserAction::Fill { selector, value } => {
            assert_eq!(selector, "#name");
            assert_eq!(value, "Ada");
        }
        other => panic!("unexpected action: {other:?}"),
    }
}

#[test]
fn parse_browser_action_rejects_missing_required_field() {
    let err = parse_browser_action("open", &json!({})).expect_err("url is required");
    assert!(err.to_string().contains("Missing 'url'"));
}

#[test]
fn action_support_lists_include_computer_use_only_actions() {
    assert!(is_supported_browser_action("open"));
    assert!(is_supported_browser_action("mouse_click"));
    assert!(is_computer_use_only_action("mouse_click"));
    assert!(!is_computer_use_only_action("open"));
    assert!(!is_supported_browser_action("launch_missiles"));
}
