use super::*;
use serde_json::json;

fn validation_error(value: serde_json::Value) -> String {
    content_block_from_value(&value, 0).expect_err("invalid content block").to_string()
}

#[test]
fn non_empty_string_validation_trims_and_rejects_non_strings() {
    assert!(is_non_empty_string(Some(&json!(" value "))));
    assert!(!is_non_empty_string(Some(&json!("   "))));
    assert!(!is_non_empty_string(Some(&json!(7))));
    assert!(!is_non_empty_string(None));
}

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
    assert!(!is_base64_data(""));
    assert!(!is_base64_data("QUJD==="));
    assert!(!is_base64_data("QUJD*==="));
    assert!(is_image_mime_type("IMAGE/PNG"));
    assert!(is_resource_payload(Some(&json!({"uri": "file:///a", "text": "body"}))));
}

#[test]
fn mime_type_validation_accepts_image_subtypes_only() {
    assert!(is_image_mime_type("image/svg+xml"));
    assert!(is_image_mime_type("image/vnd.test-icon"));
    assert!(!is_image_mime_type("text/plain"));
    assert!(!is_image_mime_type("image/"));
    assert!(!is_image_mime_type("image/bad_type"));
}

#[test]
fn resource_payload_validation_requires_object_uri_and_string_text() {
    assert!(!is_resource_payload(None));
    assert!(!is_resource_payload(Some(&json!("file:///a"))));
    assert!(!is_resource_payload(Some(&json!({"uri": "  "}))));
    assert!(!is_resource_payload(Some(&json!({"uri": "file:///a", "text": 7}))));
    assert!(is_resource_payload(Some(&json!({"uri": "file:///a"}))));
}

#[test]
fn content_block_validation_reports_invalid_block_shapes() {
    let cases = [
        (json!("text"), "must be an ACP content block object"),
        (json!({}), "must be an ACP content block object"),
        (json!({"type": 1}), "must be an ACP content block object"),
        (json!({"type": "text"}), "text block must include a string text field"),
        (json!({"type": "image", "data": "AAAA"}), "image block must include a non-empty mimeType"),
        (
            json!({"type": "image", "mimeType": "image/png"}),
            "image block must include non-empty base64 data",
        ),
        (
            json!({"type": "image", "mimeType": "image/png", "data": ""}),
            "image block must include non-empty base64 data",
        ),
        (
            json!({"type": "image", "mimeType": "image/png", "data": "not base64"}),
            "image block data must be valid base64",
        ),
        (
            json!({"type": "resource_link", "uri": ""}),
            "resource_link block must include a non-empty uri",
        ),
        (
            json!({"type": "resource_link", "uri": "file:///a", "title": 1}),
            "resource_link block title must be a string when present",
        ),
        (
            json!({"type": "resource_link", "uri": "file:///a", "name": 1}),
            "resource_link block name must be a string when present",
        ),
        (json!({"type": "resource"}), "resource block must include a resource object"),
        (
            json!({"type": "resource", "resource": {"uri": "file:///a", "text": 1}}),
            "resource block resource must include a non-empty uri and optional text",
        ),
        (json!({"type": "audio", "data": "AAAA"}), "has unsupported content block type \"audio\""),
    ];

    for (value, expected) in cases {
        let error = validation_error(value);
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn parse_prompt_source_accepts_resource_blocks_and_discards_extra_fields() {
    let prompt = parse_prompt_source(
        r#"[
            {"type":"resource_link","uri":"file:///a","name":"alpha","title":"Alpha","extra":true},
            {"type":"resource_link","uri":"file:///named","name":"Named"},
            {"type":"resource","resource":{"uri":"file:///b","text":"body","extra":true}}
        ]"#,
    )
    .expect("valid resource prompt");

    assert_eq!(prompt.len(), 3);
    assert_eq!(prompt_to_display_text(&prompt), "Alpha\n\nNamed\n\nbody");
}

#[test]
fn resource_display_falls_back_to_name_uri_and_skips_blank_segments() {
    let prompt = parse_prompt_source(
        r#"[
            {"type":"text","text":"  "},
            {"type":"resource_link","uri":"file:///named","name":"Named"},
            {"type":"resource_link","uri":"file:///uri","name":""},
            {"type":"resource","resource":{"uri":"file:///resource","text":""}},
            {"type":"image","mimeType":"image/jpeg","data":"AAAA"}
        ]"#,
    )
    .expect("valid prompt");

    assert_eq!(
        prompt_to_display_text(&prompt),
        "Named\n\nfile:///uri\n\nfile:///resource\n\n[image] image/jpeg"
    );
}

#[test]
fn parse_prompt_source_reports_conversion_errors_for_acp_required_fields() {
    let link_error = parse_prompt_source(r#"[{"type":"resource_link","uri":"file:///a"}]"#)
        .expect_err("resource_link without name cannot become ACP content");
    assert!(link_error.to_string().contains("could not be converted"));

    let resource_error =
        parse_prompt_source(r#"[{"type":"resource","resource":{"uri":"file:///a"}}]"#)
            .expect_err("resource without text cannot become ACP content");
    assert!(resource_error.to_string().contains("could not be converted"));
}

#[test]
fn parse_prompt_source_handles_empty_invalid_and_non_array_sources() {
    assert!(parse_prompt_source("  ").expect("empty prompt").is_empty());
    assert!(parse_prompt_source("[]").expect("empty structured prompt").is_empty());

    let invalid_json_prompt = parse_prompt_source("[not json").expect("plain text fallback");
    assert_eq!(prompt_to_display_text(&invalid_json_prompt), "[not json");

    let non_array_error = parse_prompt_input_value(&json!({"type": "text"}))
        .expect_err("structured value must be an array");
    assert!(non_array_error.to_string().contains("Structured prompt JSON must be an array"));
}

#[test]
fn merge_prompt_source_with_text_trims_and_appends_non_empty_suffix() {
    let prompt = merge_prompt_source_with_text(r#"[{"type":"text","text":"first"}]"#, "  second  ")
        .expect("merged prompt");
    assert_eq!(prompt_to_display_text(&prompt), "first\n\nsecond");

    let unchanged = merge_prompt_source_with_text(r#"[{"type":"text","text":"first"}]"#, "  ")
        .expect("unchanged prompt");
    assert_eq!(prompt_to_display_text(&unchanged), "first");
}

#[test]
fn is_prompt_input_accepts_arrays_where_every_entry_validates() {
    assert!(is_prompt_input(&json!([
        {"type": "text", "text": "hello"},
        {"type": "resource_link", "uri": "file:///a"},
    ])));
    assert!(!is_prompt_input(&json!({"type": "text", "text": "hello"})));
    assert!(!is_prompt_input(&json!([
        {"type": "text", "text": "hello"},
        {"type": "text"},
    ])));
}

#[test]
fn display_text_ignores_supported_blocks_without_text_display() {
    let audio = serde_json::from_value(json!({
        "type": "audio",
        "mimeType": "audio/wav",
        "data": "AAAA"
    }))
    .expect("valid audio block");

    assert_eq!(prompt_to_display_text(&[audio]), "");
}

#[test]
fn display_text_handles_direct_content_blocks_and_trims_joined_output() {
    let prompt = vec!["  first  ".to_string().into(), "second".to_string().into()];

    assert_eq!(prompt_to_display_text(&prompt), "first  \n\nsecond");
}
