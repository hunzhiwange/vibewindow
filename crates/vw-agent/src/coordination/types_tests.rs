use super::types::{InMemoryMessageBusLimits, InMemoryMessageBusStats, PublishReceipt};

#[test]
fn recommended_limits_match_default() {
    assert_eq!(InMemoryMessageBusLimits::recommended(), InMemoryMessageBusLimits::default());
}

#[test]
fn stats_and_receipts_are_copyable_values() {
    let stats = InMemoryMessageBusStats::default();
    let receipt = PublishReceipt { sequence: 7, delivered_to: 2 };

    assert_eq!(stats.deliveries_total, 0);
    assert_eq!(receipt.sequence, 7);
}
