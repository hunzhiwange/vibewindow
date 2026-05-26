//! 验证 prompt 输入在纯文本、结构化块和合并场景下的解析规则。
//!
//! Prompt 是用户输入进入 ACP 会话的第一层边界；这些测试确保仅支持明确的块类型，
//! 并在无效媒体或资源结构出现时返回可诊断错误，而不是静默降级成普通文本。

use serde_json::json;
use vw_acp::{
    is_prompt_input, merge_prompt_source_with_text, parse_prompt_source, prompt_to_display_text,
    text_prompt,
};

/// 普通字符串应被裁剪并转换为单个文本 prompt。
#[test]
fn parse_prompt_source_returns_text_prompt_for_plain_text() {
    let prompt = parse_prompt_source("  hello ACP  ").unwrap();

    assert_eq!(prompt, text_prompt("hello ACP"));
}

/// 合法结构化 prompt 块应按显示顺序转换为可读文本摘要。
#[test]
fn parse_prompt_source_parses_structured_prompt_blocks() {
    let prompt = parse_prompt_source(
        r#"[
            {"type":"text","text":"Hello"},
            {"type":"resource_link","uri":"file:///tmp/demo.txt","name":"Demo file"},
            {"type":"resource","resource":{"uri":"file:///tmp/note.md","text":"Embedded note"}}
        ]"#,
    )
    .unwrap();

    assert_eq!(prompt_to_display_text(&prompt), "Hello\n\nDemo file\n\nEmbedded note");
}

/// 非图片 MIME 的 image 块必须报错，避免调用者把不支持的载荷误当图片传递。
#[test]
fn parse_prompt_source_reports_invalid_structured_prompt() {
    let error = parse_prompt_source(
        r#"[
            {"type":"image","mimeType":"text/plain","data":"aGVsbG8="}
        ]"#,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "prompt[0] image block mimeType must start with image/");
}

/// 附加文本时只追加有效内容，并保持结构化 prompt 原有块顺序。
#[test]
fn merge_prompt_source_with_text_appends_trimmed_text() {
    let prompt =
        merge_prompt_source_with_text(r#"[{"type":"text","text":"Hello"}]"#, "  World  ").unwrap();

    assert_eq!(prompt_to_display_text(&prompt), "Hello\n\nWorld");
}

/// 输入探测只接受当前明确支持的 prompt 块形状。
#[test]
fn is_prompt_input_validates_supported_blocks() {
    let valid = json!([
        {"type":"text","text":"Hello"},
        {"type":"resource_link","uri":"file:///tmp/demo.txt","name":"demo.txt"}
    ]);
    let invalid = json!([
        {"type":"resource","resource":{"text":"Missing uri"}}
    ]);

    assert!(is_prompt_input(&valid));
    assert!(!is_prompt_input(&invalid));
}
