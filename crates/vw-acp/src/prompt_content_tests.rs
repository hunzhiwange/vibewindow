use super::*;
use serde_json::json;

#[test]
fn parse_prompt_source_accepts_structured_text_and_image_blocks() {
    let prompt = parse_prompt_source(
        r#"[{"type":"text","text":"hello"},{"type":"image","mimeType":"image/png","data":"QUJDRA=="}]"#,
    )
    .expect("valid prompt");

    assert_eq!(prompt.len(), 2);
    assert_eq!(prompt_to_display_text(&prompt), "hello\n\n[image] image/png");
}

#[test]
fn parse_prompt_source_falls_back_to_text_for_non_json_input() {
    let prompt = parse_prompt_source("  hello  ").expect("plain text prompt");

    assert_eq!(prompt_to_display_text(&prompt), "hello");
}

#[test]
fn invalid_structured_prompt_reports_block_index() {
    let error = parse_prompt_source(r#"[{"type":"image","mimeType":"text/plain","data":"AAAA"}]"#)
        .expect_err("invalid image mime type");

    assert!(error.to_string().contains("prompt[0] image block mimeType"));
}

#[test]
fn base64_validation_rejects_bad_padding() {
    assert!(is_base64_data("QUJDRA=="));
    assert!(!is_base64_data("QU=JD"));
    assert!(!is_base64_data("abc"));
    assert!(is_image_mime_type("IMAGE/PNG"));
    assert!(is_resource_payload(Some(&json!({"uri": "file:///a", "text": "body"}))));
}
