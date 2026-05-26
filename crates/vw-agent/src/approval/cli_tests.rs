use super::{ApprovalRequest, ApprovalResponse};

#[test]
fn approval_response_serializes_as_lowercase_values() {
    assert_eq!(serde_json::to_string(&ApprovalResponse::Yes).unwrap(), "\"yes\"");
    assert_eq!(serde_json::to_string(&ApprovalResponse::No).unwrap(), "\"no\"");
    assert_eq!(serde_json::to_string(&ApprovalResponse::Always).unwrap(), "\"always\"");
}

#[test]
fn approval_request_round_trips_tool_and_arguments() {
    let request = ApprovalRequest {
        tool_name: "shell".to_string(),
        arguments: serde_json::json!({"command": "pwd"}),
    };

    let encoded = serde_json::to_string(&request).unwrap();
    let decoded: ApprovalRequest = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded.tool_name, "shell");
    assert_eq!(decoded.arguments["command"], "pwd");
}
