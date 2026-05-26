use super::*;
use serde_json::json;

#[test]
fn allow_and_ask_expose_updated_input_and_reason() {
    let allow = PermissionDecision::allow(json!({"ok": true}));
    assert!(allow.reason().is_none());
    assert_eq!(allow.updated_input(), Some(&json!({"ok": true})));

    let ask = PermissionDecision::ask("needs approval", json!({"cmd": "git status"}));
    assert_eq!(ask.reason(), Some("needs approval"));
    assert_eq!(ask.updated_input(), Some(&json!({"cmd": "git status"})));
}

#[test]
fn deny_only_exposes_input_after_explicit_snapshot() {
    let deny = PermissionDecision::deny("blocked").with_warning(Some("careful".into()));
    assert_eq!(deny.reason(), Some("blocked"));
    assert_eq!(deny.warning(), Some("careful"));
    assert!(deny.updated_input().is_none());

    let deny = deny.with_updated_input(json!({"path": "secret"}));
    assert_eq!(deny.updated_input(), Some(&json!({"path": "secret"})));
}
