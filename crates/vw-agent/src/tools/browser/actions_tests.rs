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

#[test]
fn parse_browser_action_covers_optional_and_numeric_fields() {
    assert!(matches!(
        parse_browser_action(
            "snapshot",
            &json!({"interactive_only": false, "compact": false, "depth": u64::MAX})
        )
        .unwrap(),
        BrowserAction::Snapshot { interactive_only: false, compact: false, depth: Some(u32::MAX) }
    ));
    assert!(matches!(
        parse_browser_action("screenshot", &json!({"path": "shot.png", "full_page": true}))
            .unwrap(),
        BrowserAction::Screenshot { path: Some(path), full_page: true } if path == "shot.png"
    ));
    assert!(matches!(
        parse_browser_action("wait", &json!({"selector": "#ready", "ms": 7, "text": "ok"}))
            .unwrap(),
        BrowserAction::Wait { selector: Some(selector), ms: Some(7), text: Some(text) }
            if selector == "#ready" && text == "ok"
    ));
    assert!(matches!(
        parse_browser_action("scroll", &json!({"direction": "down", "pixels": u64::MAX}))
            .unwrap(),
        BrowserAction::Scroll { direction, pixels: Some(u32::MAX) } if direction == "down"
    ));
    assert!(matches!(
        parse_browser_action(
            "find",
            &json!({"by": "role", "value": "button", "find_action": "fill", "fill_value": "Ada"})
        )
        .unwrap(),
        BrowserAction::Find { by, value, action, fill_value: Some(fill) }
            if by == "role" && value == "button" && action == "fill" && fill == "Ada"
    ));
}

#[test]
fn parse_browser_action_covers_all_simple_variants() {
    assert!(matches!(
        parse_browser_action("open", &json!({"url": "https://example.com"})).unwrap(),
        BrowserAction::Open { url } if url == "https://example.com"
    ));
    assert!(matches!(
        parse_browser_action("click", &json!({"selector": "#go"})).unwrap(),
        BrowserAction::Click { selector } if selector == "#go"
    ));
    assert!(matches!(
        parse_browser_action("type", &json!({"selector": "#name", "text": "Ada"})).unwrap(),
        BrowserAction::Type { selector, text } if selector == "#name" && text == "Ada"
    ));
    assert!(matches!(
        parse_browser_action("get_text", &json!({"selector": "main"})).unwrap(),
        BrowserAction::GetText { selector } if selector == "main"
    ));
    assert!(matches!(
        parse_browser_action("get_title", &json!({})).unwrap(),
        BrowserAction::GetTitle
    ));
    assert!(matches!(parse_browser_action("get_url", &json!({})).unwrap(), BrowserAction::GetUrl));
    assert!(matches!(
        parse_browser_action("press", &json!({"key": "Enter"})).unwrap(),
        BrowserAction::Press { key } if key == "Enter"
    ));
    assert!(matches!(
        parse_browser_action("hover", &json!({"selector": ".menu"})).unwrap(),
        BrowserAction::Hover { selector } if selector == ".menu"
    ));
    assert!(matches!(
        parse_browser_action("is_visible", &json!({"selector": "#modal"})).unwrap(),
        BrowserAction::IsVisible { selector } if selector == "#modal"
    ));
    assert!(matches!(parse_browser_action("close", &json!({})).unwrap(), BrowserAction::Close));
}

#[test]
fn parse_browser_action_reports_each_required_field() {
    for (action, args, expected) in [
        ("click", json!({}), "Missing 'selector' for click"),
        ("fill", json!({"selector": "#x"}), "Missing 'value' for fill"),
        ("type", json!({"selector": "#x"}), "Missing 'text' for type"),
        ("get_text", json!({}), "Missing 'selector' for get_text"),
        ("press", json!({}), "Missing 'key' for press"),
        ("hover", json!({}), "Missing 'selector' for hover"),
        ("scroll", json!({}), "Missing 'direction' for scroll"),
        ("is_visible", json!({}), "Missing 'selector' for is_visible"),
        ("find", json!({}), "Missing 'by' for find"),
        ("find", json!({"by": "text"}), "Missing 'value' for find"),
        ("find", json!({"by": "text", "value": "Save"}), "Missing 'find_action' for find"),
        ("unknown", json!({}), "Unsupported browser action"),
    ] {
        let err = parse_browser_action(action, &args).expect_err("action should fail");
        assert!(err.to_string().contains(expected), "{action}: {err}");
    }
}
