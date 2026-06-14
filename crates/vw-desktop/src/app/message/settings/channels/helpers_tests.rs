use super::*;
use vw_config_types::channels::{GroupReplyMode, LarkReceiveMode, QQReceiveMode};

#[test]
fn channel_helper_defaults_and_parsers_work() {
    let feishu = default_feishu_config();
    assert_eq!(feishu.receive_mode, LarkReceiveMode::Websocket);
    assert_eq!(feishu.max_draft_edits, 20);
    assert_eq!(trim_to_option(" hi "), Some("hi".to_string()));
    assert_eq!(trim_to_option("   "), None);
    assert_eq!(parse_receive_mode("webhook"), LarkReceiveMode::Webhook);
    assert_eq!(parse_receive_mode("other"), LarkReceiveMode::Websocket);
    assert_eq!(parse_qq_receive_mode("websocket"), QQReceiveMode::Websocket);
    assert_eq!(parse_qq_receive_mode("other"), QQReceiveMode::Webhook);
}

#[test]
fn channel_helper_group_reply_updates_insert_defaults() {
    let mut group_reply = None;
    set_group_reply_mode(&mut group_reply, "mention_only");
    set_group_reply_allowed(&mut group_reply, "u1, u2");

    let reply = group_reply.expect("group reply should exist");
    assert_eq!(reply.mode, Some(GroupReplyMode::MentionOnly));
    assert_eq!(reply.allowed_sender_ids, vec!["u1".to_string(), "u2".to_string()]);
}
