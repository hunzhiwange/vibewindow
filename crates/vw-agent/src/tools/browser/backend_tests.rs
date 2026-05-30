use super::*;

#[test]
fn parse_accepts_stable_aliases() {
    assert_eq!(
        BrowserBackendKind::parse("agent-browser").unwrap(),
        BrowserBackendKind::AgentBrowser
    );
    assert_eq!(BrowserBackendKind::parse("native").unwrap(), BrowserBackendKind::RustNative);
    assert_eq!(BrowserBackendKind::parse("computer_use").unwrap(), BrowserBackendKind::ComputerUse);
    assert_eq!(BrowserBackendKind::parse(" AUTO ").unwrap(), BrowserBackendKind::Auto);
}

#[test]
fn names_are_stable_for_errors_and_config() {
    assert_eq!(BrowserBackendKind::RustNative.as_str(), "rust_native");
    assert_eq!(backend_name(ResolvedBackend::ComputerUse), "computer_use");
    assert_eq!(
        unavailable_action_for_backend_error("click", ResolvedBackend::AgentBrowser),
        "Action 'click' is unavailable for backend 'agent_browser'"
    );
}

#[test]
fn parse_rejects_unknown_backend() {
    let err = BrowserBackendKind::parse("mystery").expect_err("unknown backend should fail");
    assert!(err.to_string().contains("Unsupported browser backend"));
}
