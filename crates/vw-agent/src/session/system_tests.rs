use super::*;

#[test]
fn yes_no_cn_is_stable() {
    assert_eq!(yes_no_cn(true), "是");
    assert_eq!(yes_no_cn(false), "否");
}

#[test]
fn environment_text_contains_core_fields() {
    let text = environment_text("gpt-5", "openai", "/tmp/work", true, "no extra dirs");
    assert!(text.contains("gpt-5"));
    assert!(text.contains("openai/gpt-5"));
    assert!(text.contains("/tmp/work"));
    assert!(text.contains("是"));
    assert!(text.contains(platform()));
}
