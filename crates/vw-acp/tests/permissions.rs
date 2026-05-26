//! 验证 ACP 权限请求在不同运行策略下的解析、自动决策与结果分类。
//!
//! 这些用例覆盖读类工具自动允许、非交互模式拒绝写类工具，以及前端返回选项
//! 如何被映射为运行时权限结论。权限链路属于安全边界，因此测试固定请求形状
//! 与选项 ID，避免策略变更时悄然扩大工具能力。

use agent_client_protocol::RequestPermissionRequest;
use serde_json::json;
use vw_acp::{
    NonInteractivePermissionPolicy, PermissionDecision, PermissionMode,
    classify_permission_decision, permission_mode_satisfies, resolve_permission_request,
};

/// 构造一个最小权限请求夹具。
///
/// `kind` 用来模拟 ACP 工具调用上报的权限类别；返回值必须能反序列化为
/// `RequestPermissionRequest`，否则说明测试夹具已经偏离协议形状。
fn sample_request(kind: Option<&str>) -> RequestPermissionRequest {
    let mut tool_call = json!({
        "toolCallId": "tool-1",
        "title": "read: config file",
    });
    if let Some(kind) = kind {
        tool_call["kind"] = json!(kind);
    }

    serde_json::from_value(json!({
        "sessionId": "session-1",
        "toolCall": tool_call,
        "options": [
            {
                "optionId": "allow-once",
                "kind": "allow_once",
                "name": "Allow once"
            },
            {
                "optionId": "reject-once",
                "kind": "reject_once",
                "name": "Reject once"
            }
        ]
    }))
    .unwrap()
}

/// 确认权限模式只按预期方向满足更低风险的请求。
#[test]
fn permission_mode_satisfies_matches_expected_ordering() {
    assert!(permission_mode_satisfies(PermissionMode::ApproveAll, PermissionMode::ApproveReads));
    assert!(permission_mode_satisfies(PermissionMode::ApproveReads, PermissionMode::DenyAll));
    assert!(!permission_mode_satisfies(PermissionMode::DenyAll, PermissionMode::ApproveReads));
}

/// 读类工具在 `ApproveReads` 下应自动选择允许选项。
#[test]
fn resolve_permission_request_auto_approves_reads() {
    let response = resolve_permission_request(
        &sample_request(Some("read")),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();

    let value = serde_json::to_value(&response).unwrap();
    assert_eq!(value["outcome"]["outcome"], json!("selected"));
    assert_eq!(value["outcome"]["optionId"], json!("allow-once"));
}

/// 非交互策略为拒绝时，写类请求不能因为无人确认而被放行。
#[test]
fn resolve_permission_request_denies_when_non_interactive_and_policy_denies() {
    let response = resolve_permission_request(
        &sample_request(Some("edit")),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();

    let value = serde_json::to_value(&response).unwrap();
    assert_eq!(value["outcome"]["outcome"], json!("selected"));
    assert_eq!(value["outcome"]["optionId"], json!("reject-once"));
}

/// 前端选择允许选项后，运行时应把响应分类为已批准。
#[test]
fn classify_permission_decision_recognizes_selected_allow_option() {
    let request = sample_request(Some("edit"));
    let response = serde_json::from_value(json!({
        "outcome": {
            "outcome": "selected",
            "optionId": "allow-once"
        }
    }))
    .unwrap();

    assert_eq!(classify_permission_decision(&request, &response), PermissionDecision::Approved);
}
