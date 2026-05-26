use super::*;

use serde_json::json;

/// 测试 peek 操作不消费消息
///
/// 验证：使用 `peek_for_agent` 查看消息后，消息仍保留在收件箱中，
/// `pending_for_agent` 返回的待处理数量应保持不变。
#[test]
fn peek_does_not_consume_messages() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker").expect("register worker");

    let mut envelope = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-peek",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "task-1".to_string(),
            summary: "peek test".to_string(),
            metadata: json!({}),
        },
    );
    envelope.id = "msg-peek".to_string();
    bus.publish(envelope).expect("publish");

    let peeked = bus.peek_for_agent("worker", 10).expect("peek");
    assert_eq!(peeked.len(), 1);
    assert_eq!(peeked[0].envelope.id, "msg-peek");

    let pending = bus.pending_for_agent("worker").expect("pending");
    assert_eq!(pending, 1);
}

/// 测试关联待处理计数和 peek 分页遵循收件箱生命周期
///
/// 场景：
/// - 发布 4 条消息，分别属于 corr-a 和 corr-b 两个关联
/// - 验证按关联ID统计的待处理数量正确
/// - 验证使用偏移量的 peek 分页返回正确的消息
/// - 消费一条消息后，验证关联待处理数量正确减少
#[test]
fn correlation_pending_and_peek_paging_follow_inbox_lifecycle() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker").expect("register worker");

    for (message_id, correlation_id) in [
        ("msg-corr-0", "corr-a"),
        ("msg-corr-1", "corr-b"),
        ("msg-corr-2", "corr-a"),
        ("msg-corr-3", "corr-a"),
    ] {
        let mut envelope = CoordinationEnvelope::new_direct(
            "lead",
            "worker",
            "conv-peek-correlation",
            "coordination",
            CoordinationPayload::DelegateTask {
                task_id: message_id.to_string(),
                summary: "peek correlation".to_string(),
                metadata: json!({}),
            },
        );
        envelope.id = message_id.to_string();
        envelope.correlation_id = Some(correlation_id.to_string());
        bus.publish(envelope).expect("publish should succeed");
    }

    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-a")
            .expect("pending corr-a should succeed"),
        3
    );
    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-b")
            .expect("pending corr-b should succeed"),
        1
    );

    let page = bus
        .peek_for_agent_correlation_with_offset("worker", "corr-a", 1, 1)
        .expect("peek corr-a page should succeed");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].envelope.id, "msg-corr-2");

    let drained_one = bus.drain_for_agent("worker", 1).expect("drain one should succeed");
    assert_eq!(drained_one.len(), 1);
    assert_eq!(drained_one[0].envelope.id, "msg-corr-0");
    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-a")
            .expect("pending corr-a should succeed after drain"),
        2
    );
}

/// 测试收件箱溢出驱逐时关联计数保持一致
///
/// 场景：
/// - 配置收件箱大小限制为 2
/// - 发布 3 条消息，其中 m0 和 m2 属于 corr-a，m1 属于 corr-b
/// - m0 被收件箱溢出驱逐
/// - 验证 corr-a 和 corr-b 的待处理计数正确反映驱逐后的状态
/// - 验证 peek 返回正确的剩余消息
#[test]
fn inbox_correlation_counts_stay_consistent_with_overflow_evictions() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 2,
        max_dead_letters: 16,
        max_context_entries: 16,
        max_seen_message_ids: 32,
    });
    bus.register_agent("worker").expect("register worker");

    for (id, corr) in [("m0", "corr-a"), ("m1", "corr-b"), ("m2", "corr-a")] {
        let mut envelope = CoordinationEnvelope::new_direct(
            "lead",
            "worker",
            "conv-overflow-corr",
            "coordination",
            CoordinationPayload::DelegateTask {
                task_id: id.to_string(),
                summary: "overflow".to_string(),
                metadata: json!({}),
            },
        );
        envelope.id = id.to_string();
        envelope.correlation_id = Some(corr.to_string());
        bus.publish(envelope).expect("publish should succeed");
    }

    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-a")
            .expect("corr-a pending should work"),
        1
    );
    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-b")
            .expect("corr-b pending should work"),
        1
    );

    let corr_a_page = bus
        .peek_for_agent_correlation_with_offset("worker", "corr-a", 0, 10)
        .expect("corr-a peek should work");
    assert_eq!(corr_a_page.len(), 1);
    assert_eq!(corr_a_page[0].envelope.id, "m2");

    let corr_b_page = bus
        .peek_for_agent_correlation_with_offset("worker", "corr-b", 0, 10)
        .expect("corr-b peek should work");
    assert_eq!(corr_b_page.len(), 1);
    assert_eq!(corr_b_page[0].envelope.id, "m1");
}

/// 测试关联 peek 操作规范化消息关联ID中的空白字符
///
/// 验证：消息关联ID中的前后空白字符（如 " corr-a "）应被规范化为 "corr-a"，
/// 使得按 "corr-a" 查询时能正确匹配到该消息。
#[test]
fn correlation_peek_normalizes_whitespace_in_message_correlation_id() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker").expect("register worker");

    let mut envelope = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-corr-normalize",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "task-1".to_string(),
            summary: "normalize".to_string(),
            metadata: json!({}),
        },
    );
    envelope.id = "msg-corr-whitespace".to_string();
    envelope.correlation_id = Some(" corr-a ".to_string());
    bus.publish(envelope).expect("publish should succeed");

    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-a")
            .expect("pending by normalized correlation should succeed"),
        1
    );
    let page = bus
        .peek_for_agent_correlation_with_offset("worker", "corr-a", 0, 10)
        .expect("peek by normalized correlation should succeed");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].envelope.id, "msg-corr-whitespace");
}

/// 测试已注册代理列表和上下文快照的可用性
///
/// 验证：
/// - `registered_agents` 返回按字母顺序排列的已注册代理列表
/// - `context_snapshot` 返回完整的上下文键值对映射
#[test]
fn registered_agents_and_context_snapshot_are_available() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker-b").expect("register worker-b");
    bus.register_agent("worker-a").expect("register worker-a");

    let patch = CoordinationEnvelope::new_broadcast(
        "worker-a",
        "conv-snapshot",
        "context",
        CoordinationPayload::ContextPatch {
            key: "shared/key".to_string(),
            expected_version: 0,
            value: json!({"ok": true}),
        },
    );
    bus.publish(patch).expect("publish patch");

    let agents = bus.registered_agents();
    assert_eq!(agents, vec!["worker-a".to_string(), "worker-b".to_string()]);

    let snapshot = bus.context_snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(
        snapshot.get("shared/key").expect("shared key should exist").value,
        json!({"ok": true})
    );
}

/// 测试收件箱限制驱逐最旧消息并记录死信
///
/// 场景：
/// - 配置收件箱大小限制为 2
/// - 发布 3 条消息，第一条（最旧）应被驱逐
/// - 验证待处理数量为 2
/// - 验证关联待处理计数正确
/// - 验证消费后得到后两条消息（msg-limit-1, msg-limit-2）
/// - 验证死信队列包含被驱逐的 msg-limit-0
/// - 验证各项统计数据正确
#[test]
fn inbox_limit_drops_oldest_and_records_dead_letter() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 2,
        max_dead_letters: 8,
        max_context_entries: 16,
        max_seen_message_ids: 32,
    });
    bus.register_agent("worker").expect("register worker");

    for index in 0..3 {
        let mut envelope = CoordinationEnvelope::new_direct(
            "lead",
            "worker",
            "conv-limit",
            "coordination",
            CoordinationPayload::DelegateTask {
                task_id: format!("task-{index}"),
                summary: format!("work-{index}"),
                metadata: json!({}),
            },
        );
        envelope.id = format!("msg-limit-{index}");
        envelope.correlation_id = Some("corr-limit".to_string());
        bus.publish(envelope).expect("publish should succeed");
    }

    let pending = bus.pending_for_agent("worker").expect("pending should work");
    assert_eq!(pending, 2);
    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-limit")
            .expect("pending by correlation should work"),
        2
    );

    let drained = bus.drain_for_agent("worker", 0).expect("drain should work");
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].envelope.id, "msg-limit-1");
    assert_eq!(drained[1].envelope.id, "msg-limit-2");
    assert_eq!(
        bus.pending_for_agent_correlation("worker", "corr-limit")
            .expect("pending by correlation after drain should work"),
        0
    );

    let dead_letters = bus.dead_letters();
    assert_eq!(dead_letters.len(), 1);
    assert_eq!(dead_letters[0].envelope.id, "msg-limit-0");
    assert!(dead_letters[0].reason.contains("inbox overflow"));

    let stats = bus.stats();
    assert_eq!(stats.publish_attempts_total, 3);
    assert_eq!(stats.deliveries_total, 3);
    assert_eq!(stats.inbox_overflow_evictions_total, 1);
    assert_eq!(stats.dead_letters_total, 1);
    assert_eq!(stats.dead_letter_evictions_total, 0);
    assert_eq!(stats.context_evictions_total, 0);
    assert_eq!(stats.seen_message_id_evictions_total, 0);
}
