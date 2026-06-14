use super::common::{
    GROUP_REPLY_MODE_OPTIONS, LARK_RECEIVE_MODE_OPTIONS, QQ_RECEIVE_MODE_OPTIONS, bool_row,
    group_reply_mode_value, hint_row, is_multiline_list_key, lark_receive_mode_value,
    localized_label, multiline_placeholder, number_row, panel, pick_row, qq_receive_mode_value,
    text_row,
};
use crate::app::{App, Message};
use iced::widget::text;
use vw_config_types::channels::{GroupReplyConfig, GroupReplyMode, LarkReceiveMode, QQReceiveMode};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn multiline_list_keys_identify_collection_fields() {
    for key in [
        "telegram.allowed_users",
        "telegram.group_reply.allowed_sender_ids",
        "discord.allowed_users",
        "discord.group_reply.allowed_sender_ids",
        "slack.allowed_users",
        "slack.group_reply.allowed_sender_ids",
        "mattermost.allowed_users",
        "mattermost.group_reply.allowed_sender_ids",
        "imessage.allowed_contacts",
        "matrix.allowed_users",
        "signal.allowed_from",
        "whatsapp.allowed_numbers",
        "linq.allowed_senders",
        "wati.allowed_numbers",
        "nextcloud_talk.allowed_users",
        "email.allowed_senders",
        "irc.channels",
        "irc.allowed_users",
        "lark.allowed_users",
        "lark.group_reply.allowed_sender_ids",
        "feishu.allowed_users",
        "feishu.group_reply.allowed_sender_ids",
        "dingtalk.allowed_users",
        "qq.allowed_users",
        "nostr.relays",
        "nostr.allowed_pubkeys",
        "clawdtalk.allowed_destinations",
    ] {
        assert!(is_multiline_list_key(key), "{key} should be multiline");
    }
    assert!(!is_multiline_list_key("telegram.enabled"));
}

#[test]
fn localized_label_falls_back_to_original_key() {
    for (key, expected) in [
        ("bot_token", "机器人令牌"),
        ("allowed_users", "允许用户"),
        ("group_reply.mode", "群聊回复模式"),
        ("group_reply.allowed_sender_ids", "允许发送者 ID"),
        ("mention_only", "仅提及时回复"),
        ("homeserver", "Homeserver 地址"),
        ("access_token", "访问令牌"),
        ("user_id", "用户 ID"),
        ("device_id", "设备 ID"),
        ("room_id", "房间 ID"),
        ("http_url", "HTTP 地址"),
        ("allowed_destinations", "允许目的地"),
    ] {
        assert_eq!(localized_label(key), expected);
    }
    assert_eq!(localized_label("unknown.key"), "unknown.key");
}

#[test]
fn multiline_placeholder_falls_back_to_original_text() {
    assert_eq!(multiline_placeholder("逗号或换行分隔"), "每行一个，亦支持逗号分隔");
    assert_eq!(
        multiline_placeholder("电话号码或邮箱，逗号或换行分隔"),
        "每行一个电话号码或邮箱，亦支持逗号分隔"
    );
    assert_eq!(
        multiline_placeholder("逗号或换行分隔，支持 *"),
        "每行一个，亦支持逗号分隔，可使用 *"
    );
    assert_eq!(multiline_placeholder("unknown.key"), "unknown.key");
}

#[test]
fn mode_value_helpers_cover_defaults_and_variants() {
    assert_eq!(group_reply_mode_value(&None), "all_messages");
    assert_eq!(
        group_reply_mode_value(&Some(GroupReplyConfig {
            mode: Some(GroupReplyMode::MentionOnly),
            allowed_sender_ids: vec![],
        })),
        "mention_only"
    );
    assert_eq!(lark_receive_mode_value(LarkReceiveMode::Webhook), "webhook");
    assert_eq!(lark_receive_mode_value(LarkReceiveMode::Websocket), "websocket");
    assert_eq!(qq_receive_mode_value(QQReceiveMode::Webhook), "webhook");
    assert_eq!(qq_receive_mode_value(QQReceiveMode::Websocket), "websocket");
}

#[test]
fn labeled_options_display_human_labels() {
    assert_eq!(GROUP_REPLY_MODE_OPTIONS[0].to_string(), "全部消息");
    assert_eq!(GROUP_REPLY_MODE_OPTIONS[1].to_string(), "仅被提及");
    assert_eq!(LARK_RECEIVE_MODE_OPTIONS[0].to_string(), "WebSocket");
    assert_eq!(QQ_RECEIVE_MODE_OPTIONS[0].to_string(), "Webhook");
}

#[test]
fn row_helpers_build_single_line_multiline_bool_number_pick_and_hint_rows() {
    let mut app = test_app();
    app.channels_settings.text_editors.insert(
        "telegram.allowed_users".to_string(),
        iced::widget::text_editor::Content::with_text("alice\nbob"),
    );
    app.channels_settings
        .text_inputs
        .insert("telegram.bot_token".to_string(), "edited-token".to_string());

    keep_element(text_row(
        &app,
        "bot_token",
        "Telegram token",
        "fallback-token",
        "telegram.bot_token",
        true,
    ));
    keep_element(text_row(
        &app,
        "allowed_users",
        "逗号或换行分隔",
        "",
        "telegram.allowed_users",
        false,
    ));
    keep_element(bool_row("mention_only", true, "仅提及时回复", "telegram.mention_only"));
    keep_element(number_row("port", 3000, 1, 65_535, "", "lark.port"));
    keep_element(number_row("draft_update_interval_ms", 3000, 100, 60_000, "ms", "lark.delay"));
    keep_element(pick_row(
        "receive_mode",
        "webhook",
        &LARK_RECEIVE_MODE_OPTIONS,
        "lark.receive_mode",
    ));
    keep_element(pick_row(
        "receive_mode",
        "missing",
        &LARK_RECEIVE_MODE_OPTIONS,
        "lark.receive_mode",
    ));
    keep_element(hint_row("提示"));
}

#[test]
fn panel_builds_collapsed_disabled_and_expanded_enabled_content() {
    keep_element(panel("x", "标题", "说明", false, false, text("body").into()));
    keep_element(panel("x", "标题", "说明", false, true, text("body").into()));
    keep_element(panel("x", "标题", "说明", true, true, text("body").into()));
}
