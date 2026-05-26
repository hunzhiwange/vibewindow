use super::*;

use crate::app::agent::coordination::util::parse_delegate_context_correlation_from_key;
use serde_json::json;

/// 测试上下文补丁版本冲突时消息进入死信队列
///
/// 场景：
/// - 发布第一个上下文补丁，将 key 设为版本 1
/// - 发布使用过期版本（0）的第二个补丁
/// - 验证第二个补丁失败并返回 `ContextVersionMismatch` 错误
/// - 验证死信队列中包含该冲突消息
#[test]
fn context_patch_conflict_goes_to_dead_letter() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("lead").expect("register lead");

    let first_patch = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-ctx",
        "context",
        CoordinationPayload::ContextPatch {
            key: "task-99/state".to_string(),
            expected_version: 0,
            value: json!({"phase": "started"}),
        },
    );
    bus.publish(first_patch).expect("first patch must succeed");

    let stale_patch = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-ctx",
        "context",
        CoordinationPayload::ContextPatch {
            key: "task-99/state".to_string(),
            expected_version: 0,
            value: json!({"phase": "stale"}),
        },
    );
    let error = bus.publish(stale_patch).expect_err("stale expected_version must fail");
    assert_eq!(
        error,
        CoordinationError::ContextVersionMismatch {
            key: "task-99/state".to_string(),
            expected: 0,
            actual: 1
        }
    );

    let entry = bus.context_entry("task-99/state").expect("context entry must exist");
    assert_eq!(entry.version, 1);
    assert_eq!(entry.value, json!({"phase": "started"}));
    assert_eq!(bus.dead_letters().len(), 1);
}

/// 测试委派上下文补丁必须包含关联ID
///
/// 验证规则：以 "delegate/" 为前缀的上下文键要求信封必须包含关联ID，
/// 否则应返回 `MissingDelegateContextCorrelation` 错误。
#[test]
fn delegate_context_patch_requires_correlation_id() {
    let bus = InMemoryMessageBus::new();

    let mut patch = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context-correlation",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch.id = "msg-delegate-corr-required".to_string();
    let error =
        bus.publish(patch).expect_err("delegate context patch without correlation must fail");
    assert_eq!(
        error,
        CoordinationError::MissingDelegateContextCorrelation {
            key: "delegate/corr-a/state".to_string(),
            message_id: "msg-delegate-corr-required".to_string(),
        }
    );
    assert_eq!(bus.dead_letter_count(), 1);
}

/// 测试委派上下文补丁拒绝关联ID不匹配
///
/// 验证规则：委派上下文键中包含的关联ID（如 "delegate/corr-a/..." 中的 "corr-a"）
/// 必须与信封中的 `correlation_id` 一致，否则返回 `DelegateContextCorrelationMismatch` 错误。
#[test]
fn delegate_context_patch_rejects_mismatched_correlation_id() {
    let bus = InMemoryMessageBus::new();

    let mut patch = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context-correlation-mismatch",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch.id = "msg-delegate-corr-mismatch".to_string();
    patch.correlation_id = Some("corr-b".to_string());
    let error = bus.publish(patch).expect_err("delegate context patch with mismatch must fail");
    assert_eq!(
        error,
        CoordinationError::DelegateContextCorrelationMismatch {
            key: "delegate/corr-a/state".to_string(),
            message_id: "msg-delegate-corr-mismatch".to_string(),
            key_correlation_id: "corr-a".to_string(),
            envelope_correlation_id: "corr-b".to_string(),
        }
    );
    assert_eq!(bus.dead_letter_count(), 1);
}

/// 测试委派上下文补丁拒绝无效的键格式
///
/// 验证规则：委派上下文键必须遵循 "delegate/{correlation_id}/{suffix}" 格式，
/// 缺少后缀部分（如 "delegate/corr-a"）应返回 `InvalidDelegateContextKey` 错误。
#[test]
fn delegate_context_patch_rejects_invalid_delegate_key_shape() {
    let bus = InMemoryMessageBus::new();

    let mut patch = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context-key-shape",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch.id = "msg-delegate-key-shape".to_string();
    patch.correlation_id = Some("corr-a".to_string());
    let error =
        bus.publish(patch).expect_err("delegate context patch with invalid key shape must fail");
    assert_eq!(
        error,
        CoordinationError::InvalidDelegateContextKey {
            key: "delegate/corr-a".to_string(),
            message_id: "msg-delegate-key-shape".to_string(),
        }
    );
    assert_eq!(bus.dead_letter_count(), 1);
}

/// 测试委派上下文补丁拒绝空尾部段
///
/// 验证规则：委派上下文键不能以斜杠结尾（如 "delegate/corr-a/"），
/// 这种格式意味着空的尾部段，应返回 `InvalidDelegateContextKey` 错误。
#[test]
fn delegate_context_patch_rejects_empty_tail_segment() {
    let bus = InMemoryMessageBus::new();

    let mut patch = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context-key-tail",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch.id = "msg-delegate-key-tail".to_string();
    patch.correlation_id = Some("corr-a".to_string());
    let error = bus.publish(patch).expect_err("delegate context patch with empty tail must fail");
    assert_eq!(
        error,
        CoordinationError::InvalidDelegateContextKey {
            key: "delegate/corr-a/".to_string(),
            message_id: "msg-delegate-key-tail".to_string(),
        }
    );
    assert_eq!(bus.dead_letter_count(), 1);
}

/// 测试上下文限制驱逐最旧条目并跟踪统计
///
/// 场景：
/// - 配置上下文条目限制为 2
/// - 发布 3 个不同的上下文补丁
/// - 验证上下文快照只包含最新的 2 个条目
/// - 验证统计中记录了 1 次上下文驱逐
#[test]
fn context_limit_evicts_oldest_entries_and_tracks_stats() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 16,
        max_dead_letters: 16,
        max_context_entries: 2,
        max_seen_message_ids: 32,
    });

    for index in 0..3 {
        let mut patch = CoordinationEnvelope::new_broadcast(
            "lead",
            "conv-context-limit",
            "delegate.state",
            CoordinationPayload::ContextPatch {
                key: format!("delegate/corr-{index}/state"),
                expected_version: 0,
                value: json!({"phase":"queued","index":index}),
            },
        );
        patch.id = format!("context-msg-{index}");
        patch.correlation_id = Some(format!("corr-{index}"));
        bus.publish(patch).expect("context patch should publish");
    }

    let snapshot = bus.context_snapshot();
    assert_eq!(snapshot.len(), 2);
    assert!(!snapshot.contains_key("delegate/corr-0/state"));
    assert!(snapshot.contains_key("delegate/corr-1/state"));
    assert!(snapshot.contains_key("delegate/corr-2/state"));

    let stats = bus.stats();
    assert_eq!(stats.publish_attempts_total, 3);
    assert_eq!(stats.deliveries_total, 0);
    assert_eq!(stats.dead_letters_total, 0);
    assert_eq!(stats.context_evictions_total, 1);
    assert_eq!(stats.seen_message_id_evictions_total, 0);
}

/// 测试上下文限制使用写入时间最近性并保留热点键
///
/// 场景：
/// - 配置上下文条目限制为 2
/// - 先后创建键 A 和 B
/// - 更新键 A，使其成为最近写入的键
/// - 创建键 C，触发驱逐
/// - 验证 B（最久未写入）被驱逐，A 和 C 保留
/// - 验证键 A 的版本正确递增到 2
#[test]
fn context_limit_uses_write_recency_and_preserves_hot_keys() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 16,
        max_dead_letters: 16,
        max_context_entries: 2,
        max_seen_message_ids: 32,
    });

    let mut patch_a = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-context-lru",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch_a.id = "ctx-lru-a0".to_string();
    patch_a.correlation_id = Some("corr-a".to_string());
    bus.publish(patch_a).expect("first patch should publish");

    let mut patch_b = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-context-lru",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-b/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch_b.id = "ctx-lru-b0".to_string();
    patch_b.correlation_id = Some("corr-b".to_string());
    bus.publish(patch_b).expect("second patch should publish");

    let mut patch_a_update = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-context-lru",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 1,
            value: json!({"phase":"running"}),
        },
    );
    patch_a_update.id = "ctx-lru-a1".to_string();
    patch_a_update.correlation_id = Some("corr-a".to_string());
    bus.publish(patch_a_update).expect("recency update patch should publish");

    let mut patch_c = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-context-lru",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-c/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch_c.id = "ctx-lru-c0".to_string();
    patch_c.correlation_id = Some("corr-c".to_string());
    bus.publish(patch_c).expect("new key should trigger eviction under limit");

    let snapshot = bus.context_snapshot();
    assert_eq!(snapshot.len(), 2);
    assert!(snapshot.contains_key("delegate/corr-a/state"));
    assert!(snapshot.contains_key("delegate/corr-c/state"));
    assert!(!snapshot.contains_key("delegate/corr-b/state"));
    assert_eq!(snapshot.get("delegate/corr-a/state").expect("A key should remain").version, 2);

    let stats = bus.stats();
    assert_eq!(stats.context_evictions_total, 1);
    assert_eq!(stats.seen_message_id_evictions_total, 0);
}

/// 测试上下文条目按最近性分页返回（从新到旧）
///
/// 场景：
/// - 创建 3 个委派上下文条目
/// - 使用偏移量 1、限制 2 进行分页查询
/// - 验证返回的条目按最近写入时间降序排列
#[test]
fn context_entries_recent_with_offset_returns_newest_first_pages() {
    let bus = InMemoryMessageBus::new();
    for key in ["delegate/corr-a/state", "delegate/corr-b/state", "delegate/corr-c/state"] {
        let mut patch = CoordinationEnvelope::new_broadcast(
            "lead",
            "conv-context-page",
            "delegate.state",
            CoordinationPayload::ContextPatch {
                key: key.to_string(),
                expected_version: 0,
                value: json!({"phase":"queued"}),
            },
        );
        patch.id = format!("ctx-page-{key}");
        patch.correlation_id = parse_delegate_context_correlation_from_key(key).map(str::to_string);
        bus.publish(patch).expect("context patch should publish");
    }

    let page = bus.context_entries_recent_with_offset(1, 2);
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].0, "delegate/corr-b/state");
    assert_eq!(page[1].0, "delegate/corr-a/state");
}

/// 测试按关联ID查询上下文条目支持分页和计数
///
/// 场景：
/// - 为 corr-a 和 corr-b 各创建上下文条目
/// - corr-a 有 2 个条目（state 和 output），corr-b 有 1 个条目
/// - 更新 corr-a 的 state 条目
/// - 验证按关联ID统计的上下文数量正确
/// - 验证分页查询返回正确的条目顺序
#[test]
fn context_entries_recent_for_correlation_support_paging_and_count() {
    let bus = InMemoryMessageBus::new();

    let mut patch_a_state = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-correlation-context",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch_a_state.id = "ctx-corr-a-state-0".to_string();
    patch_a_state.correlation_id = Some("corr-a".to_string());
    bus.publish(patch_a_state).expect("corr-a state patch should publish");

    let mut patch_b_state = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-correlation-context",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-b/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    patch_b_state.id = "ctx-corr-b-state-0".to_string();
    patch_b_state.correlation_id = Some("corr-b".to_string());
    bus.publish(patch_b_state).expect("corr-b state patch should publish");

    let mut patch_a_state_update = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-correlation-context",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 1,
            value: json!({"phase":"running"}),
        },
    );
    patch_a_state_update.id = "ctx-corr-a-state-1".to_string();
    patch_a_state_update.correlation_id = Some("corr-a".to_string());
    bus.publish(patch_a_state_update).expect("corr-a state update should publish");

    let mut patch_a_output = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-correlation-context",
        "delegate.output",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/output".to_string(),
            expected_version: 0,
            value: json!({"summary":"done"}),
        },
    );
    patch_a_output.id = "ctx-corr-a-output-0".to_string();
    patch_a_output.correlation_id = Some("corr-a".to_string());
    bus.publish(patch_a_output).expect("corr-a output patch should publish");

    assert_eq!(bus.context_count_for_correlation("corr-a"), 2);
    assert_eq!(bus.context_count_for_correlation("corr-b"), 1);
    assert_eq!(bus.context_count_for_correlation("corr-missing"), 0);

    let page = bus.context_entries_recent_for_correlation_with_offset("corr-a", 0, 2);
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].0, "delegate/corr-a/output");
    assert_eq!(page[1].0, "delegate/corr-a/state");

    let second_page = bus.context_entries_recent_for_correlation_with_offset("corr-a", 1, 1);
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].0, "delegate/corr-a/state");
}

/// 测试委派上下文索引排除非委派键并支持分页
///
/// 场景：
/// - 创建委派上下文条目和非委派上下文条目
/// - 验证总上下文计数为 3，委派上下文计数为 2
/// - 验证按关联ID查询委派上下文数量正确
/// - 验证分页查询只返回委派上下文条目
#[test]
fn delegate_context_indexes_exclude_non_delegate_keys_and_support_paging() {
    let bus = InMemoryMessageBus::new();

    let mut delegate_a_state = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context",
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/state".to_string(),
            expected_version: 0,
            value: json!({"phase":"queued"}),
        },
    );
    delegate_a_state.id = "delegate-a-state-0".to_string();
    delegate_a_state.correlation_id = Some("corr-a".to_string());
    bus.publish(delegate_a_state).expect("delegate a state patch should publish");

    let mut non_delegate = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context",
        "context",
        CoordinationPayload::ContextPatch {
            key: "shared/other".to_string(),
            expected_version: 0,
            value: json!({"k":"v"}),
        },
    );
    non_delegate.id = "shared-other-0".to_string();
    non_delegate.correlation_id = Some("corr-a".to_string());
    bus.publish(non_delegate).expect("non-delegate patch should publish");

    let mut delegate_a_output = CoordinationEnvelope::new_broadcast(
        "lead",
        "conv-delegate-context",
        "delegate.output",
        CoordinationPayload::ContextPatch {
            key: "delegate/corr-a/output".to_string(),
            expected_version: 0,
            value: json!({"summary":"done"}),
        },
    );
    delegate_a_output.id = "delegate-a-output-0".to_string();
    delegate_a_output.correlation_id = Some("corr-a".to_string());
    bus.publish(delegate_a_output).expect("delegate a output patch should publish");

    assert_eq!(bus.context_count(), 3);
    assert_eq!(bus.delegate_context_count(), 2);
    assert_eq!(bus.delegate_context_count_for_correlation("corr-a"), 2);
    assert_eq!(bus.delegate_context_count_for_correlation("corr-missing"), 0);

    let all_delegate = bus.delegate_context_entries_recent_with_offset(0, 0);
    assert_eq!(all_delegate.len(), 2);
    assert_eq!(all_delegate[0].0, "delegate/corr-a/output");
    assert_eq!(all_delegate[1].0, "delegate/corr-a/state");

    let delegate_page =
        bus.delegate_context_entries_recent_for_correlation_with_offset("corr-a", 1, 1);
    assert_eq!(delegate_page.len(), 1);
    assert_eq!(delegate_page[0].0, "delegate/corr-a/state");
}
