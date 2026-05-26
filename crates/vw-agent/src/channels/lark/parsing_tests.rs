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
fn parse_image_key_and_strip_at_placeholders_handle_boundaries() {
    assert_eq!(parse_image_key(r#"{"image_key":"img_v2"}"#), Some("img_v2".to_string()));
    assert_eq!(parse_image_key("not json"), None);
    assert_eq!(strip_at_placeholders("hi @_user_1 there"), "hi there");
}
