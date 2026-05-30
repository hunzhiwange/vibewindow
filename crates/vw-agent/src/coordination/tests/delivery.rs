use super::*;

use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Barrier;

/// 测试重复消息ID被拒绝并记录到死信队列
///
/// 验证规则：
/// - 首次发布具有特定ID的消息应成功投递
/// - 使用相同ID再次发布应失败并返回 `DuplicateMessageId` 错误
/// - 重复消息应被记录到死信队列，且原因包含"重复消息ID"说明
#[test]
fn duplicate_message_ids_are_rejected_and_dead_lettered() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker").expect("register worker");

    let mut envelope = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-1",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "task-1".to_string(),
            summary: "Investigate".to_string(),
            metadata: json!({}),
        },
    );
    envelope.id = "fixed-id".to_string();

    let first = bus.publish(envelope.clone()).expect("first publish");
    assert_eq!(first.delivered_to, 1);

    let second = bus.publish(envelope).expect_err("duplicate id must fail");
    assert_eq!(
        second,
        CoordinationError::DuplicateMessageId { message_id: "fixed-id".to_string() }
    );

    let dead_letters = bus.dead_letters();
    assert_eq!(dead_letters.len(), 1);
    assert!(dead_letters[0].reason.contains("duplicate message id"));

    let stats = bus.stats();
    assert_eq!(stats.seen_message_id_evictions_total, 0);
}

/// 测试去重窗口驱逐旧ID并在驱逐后允许重用
///
/// 场景：
/// - 配置消息总线，使去重窗口大小为 2
/// - 发布 3 条消息（msg-0, msg-1, msg-2），导致 msg-0 被驱逐
/// - 验证被驱逐的 msg-0 可以被重用并成功发布
/// - 验证仍在窗口内的 msg-2 不能被重复使用
/// - 验证统计中记录了 2 次去重窗口驱逐
#[test]
fn dedupe_window_evicts_old_ids_and_allows_reuse_after_eviction() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 32,
        max_dead_letters: 32,
        max_context_entries: 32,
        max_seen_message_ids: 2,
    });
    bus.register_agent("worker").expect("register worker");

    for message_id in ["msg-0", "msg-1", "msg-2"] {
        let mut envelope = CoordinationEnvelope::new_direct(
            "lead",
            "worker",
            "conv-dedupe-window",
            "coordination",
            CoordinationPayload::DelegateTask {
                task_id: message_id.to_string(),
                summary: "Investigate".to_string(),
                metadata: json!({}),
            },
        );
        envelope.id = message_id.to_string();
        bus.publish(envelope).expect("publish should succeed");
    }

    let mut reused = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-dedupe-window",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "msg-0".to_string(),
            summary: "Investigate again".to_string(),
            metadata: json!({}),
        },
    );
    reused.id = "msg-0".to_string();
    bus.publish(reused).expect("reused id should be accepted after eviction");

    let mut duplicate_recent = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-dedupe-window",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "msg-2".to_string(),
            summary: "duplicate".to_string(),
            metadata: json!({}),
        },
    );
    duplicate_recent.id = "msg-2".to_string();
    let error = bus.publish(duplicate_recent).expect_err("recent duplicate should be rejected");
    assert_eq!(error, CoordinationError::DuplicateMessageId { message_id: "msg-2".to_string() });

    let stats = bus.stats();
    assert_eq!(stats.seen_message_id_evictions_total, 2);
}

/// 测试并发发布消息时保持收件箱顺序
///
/// 场景：
/// - 使用 4 个工作线程并发发布 32 条消息到同一收件箱
/// - 使用 Barrier 确保所有任务同时开始
/// - 验证所有消息都成功发布并被投递
/// - 验证消费后的消息按序号严格递增排序
/// - 验证所有任务的 task_id 都被正确记录
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_publish_keeps_inbox_order() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("lead").expect("register lead");
    bus.register_agent("worker").expect("register worker");

    let total = 32usize;
    let barrier = Arc::new(Barrier::new(total));
    let mut tasks = Vec::with_capacity(total);

    for index in 0..total {
        let bus_clone = bus.clone();
        let barrier_clone = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            let mut envelope = CoordinationEnvelope::new_direct(
                "lead",
                "worker",
                "conv-concurrent",
                "coordination",
                CoordinationPayload::DelegateTask {
                    task_id: format!("task-{index}"),
                    summary: format!("work-{index}"),
                    metadata: json!({"idx": index}),
                },
            );
            envelope.id = format!("msg-{index}");
            bus_clone.publish(envelope).expect("publish").sequence
        }));
    }

    let mut published_sequences = Vec::with_capacity(total);
    for handle in tasks {
        published_sequences.push(handle.await.expect("join"));
    }
    assert_eq!(published_sequences.len(), total);

    let drained = bus.drain_for_agent("worker", 0).expect("drain worker inbox should succeed");
    assert_eq!(drained.len(), total);

    for pair in drained.windows(2) {
        assert!(pair[0].sequence < pair[1].sequence);
    }

    let mut seen_tasks = HashSet::new();
    for item in drained {
        if let CoordinationPayload::DelegateTask { task_id, .. } = item.envelope.payload {
            seen_tasks.insert(task_id);
        }
    }
    assert_eq!(seen_tasks.len(), total);
}

/// 测试多代理委派流程中上下文更新和结果返回
///
/// 场景：
/// - Leader 向 Researcher 发送任务委派请求
/// - Researcher 更新上下文（添加调查发现）
/// - Researcher 向 Leader 返回任务结果
/// - 验证消息在各代理的收件箱中正确排列
/// - 验证上下文条目正确记录版本、更新者和消息ID
#[test]
fn multi_agent_delegation_flow_updates_context_and_returns_result() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("lead").expect("register lead");
    bus.register_agent("researcher").expect("register researcher");

    let mut request = CoordinationEnvelope::new_direct(
        "lead",
        "researcher",
        "conv-42",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "task-42".to_string(),
            summary: "Find root cause".to_string(),
            metadata: json!({"priority": "p1"}),
        },
    );
    request.id = "msg-request".to_string();
    request.correlation_id = Some("corr-42".to_string());
    bus.publish(request.clone()).expect("request should publish");

    let researcher_inbox = bus.drain_for_agent("researcher", 10).expect("researcher drain");
    assert_eq!(researcher_inbox.len(), 1);
    assert_eq!(researcher_inbox[0].envelope.id, "msg-request");

    let mut patch = CoordinationEnvelope::new_broadcast(
        "researcher",
        "conv-42",
        "context",
        CoordinationPayload::ContextPatch {
            key: "task-42/findings".to_string(),
            expected_version: 0,
            value: json!({"summary": "Root cause isolated"}),
        },
    );
    patch.id = "msg-patch".to_string();
    patch.correlation_id = Some("corr-42".to_string());
    patch.causation_id = Some("msg-request".to_string());
    bus.publish(patch).expect("context patch should publish");

    let mut result = CoordinationEnvelope::new_direct(
        "researcher",
        "lead",
        "conv-42",
        "coordination",
        CoordinationPayload::TaskResult {
            task_id: "task-42".to_string(),
            success: true,
            output: "Investigation complete".to_string(),
        },
    );
    result.id = "msg-result".to_string();
    result.correlation_id = Some("corr-42".to_string());
    result.causation_id = Some("msg-request".to_string());
    bus.publish(result).expect("result should publish");

    let lead_inbox = bus.drain_for_agent("lead", 10).expect("lead drain");
    assert_eq!(lead_inbox.len(), 2);
    assert_eq!(lead_inbox[0].envelope.id, "msg-patch");
    assert_eq!(lead_inbox[1].envelope.id, "msg-result");

    let context = bus.context_entry("task-42/findings").expect("context must exist");
    assert_eq!(context.version, 1);
    assert_eq!(context.updated_by, "researcher");
    assert_eq!(context.last_message_id, "msg-patch");
    assert_eq!(context.value, json!({"summary": "Root cause isolated"}));
}
