use super::*;

#[test]
fn normalize_group_reply_allowed_sender_ids_trims_sorts_and_dedups() {
    let ids = vec![
        "  bob  ".to_string(),
        "alice".to_string(),
        "".to_string(),
        "bob".to_string(),
        "   ".to_string(),
    ];

    assert_eq!(normalize_group_reply_allowed_sender_ids(ids), vec!["alice", "bob"]);
}

#[test]
fn normalize_incoming_content_requires_and_removes_bot_mention() {
    assert_eq!(
        normalize_incoming_content("hello", true, "42"),
        None
    );
    assert_eq!(
        normalize_incoming_content("<@42>  hello <@!42>", true, "42"),
        Some("hello".to_string())
    );
    assert_eq!(
        normalize_incoming_content("  hello  ", false, "42"),
        Some("hello".to_string())
    );
}
