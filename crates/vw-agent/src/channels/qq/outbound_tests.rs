use super::*;

#[test]
fn resolve_send_endpoints_sanitizes_direct_user_recipient() {
    let (messages, files) = resolve_send_endpoints("user:abc-123!");

    assert!(messages.ends_with("/v2/users/abc123/messages"));
    assert!(files.ends_with("/v2/users/abc123/files"));
}

#[test]
fn resolve_send_endpoints_preserves_group_recipient() {
    let (messages, files) = resolve_send_endpoints("group:g-1");

    assert!(messages.ends_with("/v2/groups/g-1/messages"));
    assert!(files.ends_with("/v2/groups/g-1/files"));
}
