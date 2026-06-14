use super::*;

#[test]
fn invalid_webdriver_endpoint_is_not_available() {
    assert!(!NativeBrowserState::is_available(true, "not a url", None));
}

#[tokio::test]
async fn close_action_succeeds_without_existing_session() {
    let mut state = NativeBrowserState::default();

    let value = state
        .execute_action(BrowserAction::Close, true, "http://127.0.0.1:9", None)
        .await
        .expect("close should be idempotent");

    assert_eq!(value["backend"], "rust_native");
    assert_eq!(value["action"], "close");
    assert_eq!(value["closed"], true);
}

#[tokio::test]
async fn actions_requiring_session_report_missing_open_hint() {
    let mut state = NativeBrowserState::default();

    for action in [
        BrowserAction::GetTitle,
        BrowserAction::GetUrl,
        BrowserAction::Snapshot { interactive_only: true, compact: true, depth: None },
        BrowserAction::Click { selector: "#go".into() },
        BrowserAction::Fill { selector: "#name".into(), value: "Ada".into() },
        BrowserAction::Type { selector: "#name".into(), text: "Ada".into() },
        BrowserAction::GetText { selector: "main".into() },
        BrowserAction::Screenshot { path: None, full_page: false },
        BrowserAction::Wait { selector: Some("#ready".into()), ms: None, text: None },
        BrowserAction::Press { key: "Enter".into() },
        BrowserAction::Hover { selector: "#menu".into() },
        BrowserAction::Scroll { direction: "down".into(), pixels: Some(10) },
        BrowserAction::IsVisible { selector: "#modal".into() },
        BrowserAction::Find {
            by: "text".into(),
            value: "Save".into(),
            action: "click".into(),
            fill_value: None,
        },
    ] {
        let err = state
            .execute_action(action, true, "http://127.0.0.1:9", None)
            .await
            .expect_err("action should require an open session");
        assert!(err.to_string().contains("Run browser action='open' first"));
    }
}
