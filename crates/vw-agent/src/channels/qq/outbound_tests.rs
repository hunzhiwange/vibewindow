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

#[test]
fn resolve_send_endpoints_treats_plain_recipient_as_user_and_keeps_underscores() {
    let (messages, files) = resolve_send_endpoints("abc_DEF-123!");

    assert!(messages.ends_with("/v2/users/abc_DEF123/messages"));
    assert!(files.ends_with("/v2/users/abc_DEF123/files"));
}

#[test]
fn resolve_send_endpoints_allows_empty_sanitized_user_id() {
    let (messages, files) = resolve_send_endpoints("user:!!!");

    assert!(messages.ends_with("/v2/users//messages"));
    assert!(files.ends_with("/v2/users//files"));
}
