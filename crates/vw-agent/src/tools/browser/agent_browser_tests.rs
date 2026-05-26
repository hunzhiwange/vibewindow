use super::*;

#[test]
fn agent_browser_response_deserializes_success_and_error_shapes() {
    let ok: AgentBrowserResponse =
        serde_json::from_str(r#"{"success":true,"data":{"title":"Example"},"error":null}"#)
            .unwrap();
    assert!(ok.success);
    assert_eq!(ok.data.unwrap()["title"], "Example");
    assert!(ok.error.is_none());

    let err: AgentBrowserResponse =
        serde_json::from_str(r#"{"success":false,"data":null,"error":"not found"}"#).unwrap();
    assert!(!err.success);
    assert_eq!(err.error.as_deref(), Some("not found"));
}
