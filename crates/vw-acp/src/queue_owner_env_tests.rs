//! 队列所有者环境载荷解析的单元测试。

use crate::{
    AuthPolicy, NonInteractivePermissionPolicy, PermissionMode, parse_queue_owner_payload,
};

#[test]
fn parse_queue_owner_payload_maps_supported_fields() {
    let options = parse_queue_owner_payload(
        r#"{
            "sessionId": "session-1",
            "permissionMode": "approve-reads",
            "mcpServers": [
                {
                    "name": "stdio-server",
                    "command": "/bin/echo"
                }
            ],
            "nonInteractivePermissions": "fail",
            "authCredentials": {
                "token": "secret",
                "numeric": 1
            },
            "authPolicy": "fail",
            "suppressSdkConsoleErrors": true,
            "verbose": true,
            "ttlMs": 300000,
            "maxQueueDepth": 1.2,
            "promptRetries": 2.6
        }"#,
    )
    .expect("payload should parse");

    assert_eq!(options.session_id, "session-1");
    assert_eq!(options.permission_mode, PermissionMode::ApproveReads);
    assert_eq!(options.mcp_servers.as_ref().map(Vec::len), Some(1));
    assert_eq!(options.non_interactive_permissions, Some(NonInteractivePermissionPolicy::Fail));
    assert_eq!(
        options
            .auth_credentials
            .as_ref()
            .and_then(|entries: &std::collections::HashMap<String, String>| entries.get("token")),
        Some(&"secret".to_string())
    );
    assert_eq!(
        options
            .auth_credentials
            .as_ref()
            .map(|entries: &std::collections::HashMap<String, String>| entries.len()),
        Some(1)
    );
    assert_eq!(options.auth_policy, Some(AuthPolicy::Fail));
    assert_eq!(options.suppress_sdk_console_errors, Some(true));
    assert_eq!(options.verbose, Some(true));
    assert_eq!(options.ttl_ms, Some(300000));
    assert_eq!(options.max_queue_depth, Some(1));
    assert_eq!(options.prompt_retries, Some(3));
}

#[test]
fn parse_queue_owner_payload_rejects_missing_session_id() {
    let error = parse_queue_owner_payload(
        r#"{
            "permissionMode": "approve-all"
        }"#,
    )
    .expect_err("missing sessionId should fail");

    assert_eq!(error.to_string(), "queue owner payload missing sessionId");
}

#[test]
fn parse_queue_owner_payload_rejects_invalid_permission_mode() {
    let error = parse_queue_owner_payload(
        r#"{
            "sessionId": "session-1",
            "permissionMode": "invalid"
        }"#,
    )
    .expect_err("invalid permission mode should fail");

    assert_eq!(error.to_string(), "queue owner payload has invalid permissionMode");
}
