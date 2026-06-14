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

#[test]
fn parse_cli_response_accepts_yes_always_and_defaults_to_no() {
    assert_eq!(super::cli::parse_cli_response("y\n"), ApprovalResponse::Yes);
    assert_eq!(super::cli::parse_cli_response(" YES "), ApprovalResponse::Yes);
    assert_eq!(super::cli::parse_cli_response("a"), ApprovalResponse::Always);
    assert_eq!(super::cli::parse_cli_response("Always\n"), ApprovalResponse::Always);
    assert_eq!(super::cli::parse_cli_response(""), ApprovalResponse::No);
    assert_eq!(super::cli::parse_cli_response("maybe"), ApprovalResponse::No);
}
