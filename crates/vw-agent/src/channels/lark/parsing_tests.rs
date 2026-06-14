use super::*;

#[test]
fn parse_post_content_details_extracts_text_links_and_mentions() {
    let content = serde_json::json!({
        "zh_cn": {
            "title": "Title",
            "content": [[
                { "tag": "text", "text": "hello " },
                { "tag": "a", "text": "site", "href": "https://example.com" },
                { "tag": "at", "user_name": "Bot", "user_id": "ou_bot" }
            ]]
        }
    })
    .to_string();

    let parsed = parse_post_content_details(&content).expect("post content");

    assert_eq!(parsed.text, "Title\n\nhello site@Bot");
    assert_eq!(parsed.mentioned_open_ids, vec!["ou_bot".to_string()]);
}

#[test]
fn parse_post_content_details_falls_back_to_en_or_first_locale_and_ignores_empty_content() {
    let en_content = serde_json::json!({
        "en_us": {
            "content": [[
                { "tag": "a", "href": "https://example.com" },
                { "tag": "at", "user_id": "ou_user" }
            ]]
        }
    })
    .to_string();

    let parsed = parse_post_content_details(&en_content).expect("en post content");
    assert_eq!(parsed.text, "https://example.com@ou_user");
    assert_eq!(parse_post_content(&en_content).as_deref(), Some("https://example.com@ou_user"));

    let first_locale = serde_json::json!({
        "fr_fr": { "title": "Bonjour", "content": [] }
    })
    .to_string();
    assert_eq!(parse_post_content(&first_locale).as_deref(), Some("Bonjour"));

    assert!(parse_post_content_details("not-json").is_none());
    assert!(parse_post_content_details(r#"{"zh_cn":{"content":[[]]}}"#).is_none());
}

#[test]
fn parse_image_key_and_strip_at_placeholders_handle_boundaries() {
    assert_eq!(parse_image_key(r#"{"image_key":"img_v2"}"#), Some("img_v2".to_string()));
    assert_eq!(parse_image_key(r#"{"image_key":42}"#), None);
    assert_eq!(parse_image_key("not json"), None);
    assert_eq!(strip_at_placeholders("hi @_user_1 there"), "hi there");
    assert_eq!(strip_at_placeholders("hi @alice there"), "hi @alice there");
}

#[test]
fn mention_matching_supports_nested_and_flat_open_id_shapes() {
    assert!(mention_matches_bot_open_id(
        &serde_json::json!({"id": {"open_id": "ou_bot"}}),
        "ou_bot"
    ));
    assert!(mention_matches_bot_open_id(&serde_json::json!({"open_id": "ou_bot"}), "ou_bot"));
    assert!(!mention_matches_bot_open_id(
        &serde_json::json!({"id": {"open_id": "ou_other"}}),
        "ou_bot"
    ));
}

#[test]
fn normalize_group_reply_sender_ids_trims_sorts_and_deduplicates() {
    assert_eq!(
        normalize_group_reply_allowed_sender_ids(vec![
            " ou_b ".to_string(),
            "".to_string(),
            "ou_a".to_string(),
            "ou_b".to_string(),
        ]),
        vec!["ou_a".to_string(), "ou_b".to_string()]
    );
}

#[test]
fn should_respond_in_group_covers_override_all_messages_and_mention_only() {
    assert!(should_respond_in_group(true, "ou_sender", &["*".to_string()], None, &[], &[]));
    assert!(!should_respond_in_group(true, "", &["*".to_string()], None, &[], &[]));
    assert!(should_respond_in_group(false, "ou_sender", &[], None, &[], &[]));
    assert!(!should_respond_in_group(true, "ou_sender", &[], Some("ou_bot"), &[], &[]));
    assert!(should_respond_in_group(
        true,
        "ou_sender",
        &[],
        Some("ou_bot"),
        &[serde_json::json!({"open_id": "ou_bot"})],
        &[]
    ));
    assert!(should_respond_in_group(
        true,
        "ou_sender",
        &[],
        Some("ou_bot"),
        &[],
        &["ou_bot".to_string()]
    ));
}
