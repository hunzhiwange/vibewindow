use super::*;

#[test]
fn normalize_telegram_identity_trims_leading_at() {
    assert_eq!(normalize_telegram_identity("  @alice  "), "alice");
    assert_eq!(normalize_telegram_identity("bob"), "bob");
}

#[test]
fn maybe_restart_managed_daemon_service_is_currently_noop() {
    assert!(!maybe_restart_managed_daemon_service().expect("restart check"));
}
