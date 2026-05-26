use super::*;

use serde_json::json;

/// 测试死信队列限制被正确执行
///
/// 场景：
/// - 配置死信队列大小限制为 2
/// - 发布 4 条无效消息（缺少关联ID），都将进入死信队列
/// - 验证死信队列只保留最新的 2 条（msg-dead-2, msg-dead-3）
/// - 验证统计中记录了 2 次死信驱逐
#[test]
fn dead_letter_limit_is_capped() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 16,
        max_dead_letters: 2,
        max_context_entries: 16,
        max_seen_message_ids: 32,
    });
    bus.register_agent("worker").expect("register worker");

    for index in 0..4 {
        let mut invalid = CoordinationEnvelope::new_direct(
            "worker",
            "worker",
            "conv-dead-letter-limit",
            "coordination",
            CoordinationPayload::TaskResult {
                task_id: format!("task-{index}"),
                success: false,
                output: "failed".to_string(),
            },
        );
        invalid.id = format!("msg-dead-{index}");
        let _ = bus.publish(invalid);
    }

    let dead_letters = bus.dead_letters();
    assert_eq!(dead_letters.len(), 2);
    assert_eq!(dead_letters[0].envelope.id, "msg-dead-2");
    assert_eq!(dead_letters[1].envelope.id, "msg-dead-3");
    assert_eq!(bus.dead_letter_count(), 2);

    let stats = bus.stats();
    assert_eq!(stats.publish_attempts_total, 0);
    assert_eq!(stats.deliveries_total, 0);
    assert_eq!(stats.inbox_overflow_evictions_total, 0);
    assert_eq!(stats.dead_letters_total, 4);
    assert_eq!(stats.dead_letter_evictions_total, 2);
    assert_eq!(stats.context_evictions_total, 0);
    assert_eq!(stats.seen_message_id_evictions_total, 0);
}

/// 测试死信队列按最近性分页返回（从新到旧）
///
/// 场景：
/// - 发布 4 条无效消息（缺少关联ID）进入死信队列
/// - 使用偏移量 1、限制 2 进行分页查询
/// - 验证返回的死信按时间降序排列（最新的在前）
#[test]
fn dead_letters_recent_returns_newest_first_pages() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker").expect("register worker");

    for index in 0..4 {
        let mut invalid = CoordinationEnvelope::new_direct(
            "lead",
            "worker",
            "conv-dead-letter-page",
            "delegate.result",
            CoordinationPayload::TaskResult {
                task_id: format!("task-{index}"),
                success: false,
                output: "failure".to_string(),
            },
        );
        invalid.id = format!("dead-page-{index}");
        let _ = bus.publish(invalid);
    }

    let page = bus.dead_letters_recent(1, 2);
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].envelope.id, "dead-page-2");
    assert_eq!(page[1].envelope.id, "dead-page-1");
}

/// 测试死信关联索引跟踪驱逐和分页
///
/// 场景：
/// - 配置死信队列大小限制为 2
/// - 发布 3 条针对不存在代理的消息，全部进入死信队列
/// - 其中 2 条属于 corr-a，1 条属于 corr-b
/// - 由于限制为 2，死信队列驱逐后应各有 1 条
/// - 验证按关联ID统计的死信数量正确
/// - 验证分页查询返回正确的死信条目
#[test]
fn dead_letter_correlation_index_tracks_evictions_and_paging() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 16,
        max_dead_letters: 2,
        max_context_entries: 16,
        max_seen_message_ids: 32,
    });
    bus.register_agent("worker").expect("register worker");

    let publish_invalid_with_correlation = |message_id: &str, correlation_id: &str| {
        let mut envelope = CoordinationEnvelope::new_direct(
            "lead",
            "missing-worker",
            "conv-correlation-dead-letters",
            "delegate.request",
            CoordinationPayload::DelegateTask {
                task_id: message_id.to_string(),
                summary: "should dead-letter".to_string(),
                metadata: json!({}),
            },
        );
        envelope.id = message_id.to_string();
        envelope.correlation_id = Some(correlation_id.to_string());
        let _ = bus.publish(envelope);
    };

    publish_invalid_with_correlation("dead-corr-a-0", "corr-a");
    publish_invalid_with_correlation("dead-corr-b-0", "corr-b");
    publish_invalid_with_correlation("dead-corr-a-1", "corr-a");

    assert_eq!(bus.dead_letter_count(), 2);
    assert_eq!(bus.dead_letter_count_for_correlation("corr-a"), 1);
    assert_eq!(bus.dead_letter_count_for_correlation("corr-b"), 1);
    assert_eq!(bus.dead_letter_count_for_correlation("corr-missing"), 0);

    let corr_a_page = bus.dead_letters_recent_for_correlation("corr-a", 0, 2);
    assert_eq!(corr_a_page.len(), 1);
    assert_eq!(corr_a_page[0].envelope.id, "dead-corr-a-1");

    let corr_a_offset_page = bus.dead_letters_recent_for_correlation("corr-a", 1, 2);
    assert!(corr_a_offset_page.is_empty());

    let corr_b_page = bus.dead_letters_recent_for_correlation("corr-b", 0, 2);
    assert_eq!(corr_b_page.len(), 1);
    assert_eq!(corr_b_page[0].envelope.id, "dead-corr-b-0");
}
