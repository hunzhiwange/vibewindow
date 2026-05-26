use super::*;
use serde_json::json;

#[test]
fn connected_account_usability_accepts_active_lifecycle_states() {
    for status in ["ACTIVE", "initializing", "Initiated"] {
        let account =
            ComposioConnectedAccount { id: "acct".into(), status: status.into(), toolkit: None };
        assert!(account.is_usable());
    }
    let disabled =
        ComposioConnectedAccount { id: "acct".into(), status: "FAILED".into(), toolkit: None };
    assert!(!disabled.is_usable());
}

#[test]
fn toolkit_slug_reads_nested_slug_without_name_fallback() {
    let account = ComposioConnectedAccount {
        id: "acct".into(),
        status: "ACTIVE".into(),
        toolkit: Some(ComposioToolkitRef {
            slug: Some("github".into()),
            name: Some("GitHub".into()),
        }),
    };
    assert_eq!(account.toolkit_slug(), Some("github"));
}

#[test]
fn action_serialization_keeps_app_name_alias_and_optional_schema() {
    let action = ComposioAction {
        name: "GITHUB_GET_REPO".into(),
        app_name: Some("github".into()),
        description: None,
        enabled: true,
        input_parameters: Some(json!({"type":"object"})),
    };
    let encoded = serde_json::to_value(action).unwrap();
    assert_eq!(encoded["appName"], "github");
    assert!(encoded.get("input_parameters").is_some());
}
