use super::*;

#[test]
fn normalize_queue_owner_ttl_preserves_zero_and_uses_default() {
    assert_eq!(normalize_queue_owner_ttl_ms(None), DEFAULT_QUEUE_OWNER_TTL_MS);
    assert_eq!(normalize_queue_owner_ttl_ms(Some(0)), 0);
    assert_eq!(normalize_queue_owner_ttl_ms(Some(42)), 42);
}
