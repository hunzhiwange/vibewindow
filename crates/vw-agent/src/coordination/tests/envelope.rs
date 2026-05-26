use super::*;

use serde_json::json;

/// 测试任务委派消息必须指定目标代理
///
/// 验证规则：`DelegateTask` 类型的消息要求 `to` 字段必须存在，
/// 否则应返回 `MissingTarget` 错误。
#[test]
fn delegate_task_requires_direct_target() {
    let envelope = CoordinationEnvelope {
        id: "msg-1".to_string(),
        conversation_id: "conv-1".to_string(),
        correlation_id: None,
        causation_id: None,
        from: "lead".to_string(),
        to: None,
        topic: "coordination".to_string(),
        scope: DeliveryScope::Direct,
        payload: CoordinationPayload::DelegateTask {
            task_id: "task-1".to_string(),
            summary: "Investigate bug".to_string(),
            metadata: json!({}),
        },
    };

    let error = envelope.validate().expect_err("target agent must be required");
    assert_eq!(error, CoordinationError::MissingTarget { message_id: "msg-1".to_string() });
}

/// 测试任务结果消息必须包含关联ID
///
/// 验证规则：`TaskResult` 类型的消息要求 `correlation_id` 字段必须存在，
/// 以便追踪请求-响应关系，否则应返回 `MissingCorrelationId` 错误。
#[test]
fn task_result_requires_correlation_id() {
    let envelope = CoordinationEnvelope {
        id: "msg-2".to_string(),
        conversation_id: "conv-1".to_string(),
        correlation_id: None,
        causation_id: None,
        from: "worker".to_string(),
        to: Some("lead".to_string()),
        topic: "coordination".to_string(),
        scope: DeliveryScope::Direct,
        payload: CoordinationPayload::TaskResult {
            task_id: "task-1".to_string(),
            success: true,
            output: "done".to_string(),
        },
    };

    let error = envelope.validate().expect_err("task result must require correlation");
    assert_eq!(
        error,
        CoordinationError::MissingCorrelationId { message_id: "msg-2".to_string() }
    );
}

/// 测试消息的 JSON 序列化/反序列化往返保持负载形状
///
/// 验证：消息信封经过 JSON 序列化和反序列化后，其所有字段应保持完整一致。
#[test]
fn json_roundtrip_keeps_payload_shape() {
    let mut envelope = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-1",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "task-1".to_string(),
            summary: "Analyze logs".to_string(),
            metadata: json!({"priority": "high"}),
        },
    );
    envelope.correlation_id = Some("corr-1".to_string());

    let encoded = serde_json::to_string(&envelope).expect("serialize envelope");
    let decoded: CoordinationEnvelope =
        serde_json::from_str(&encoded).expect("deserialize envelope");
    assert_eq!(decoded, envelope);
}
