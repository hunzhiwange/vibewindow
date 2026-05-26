use super::code_review::{render_unified_line, unified_style_for_line};

#[test]
fn unified_line_style_distinguishes_add_delete_and_context() {
    assert_ne!(unified_style_for_line("+new"), unified_style_for_line("-old"));
    assert_eq!(unified_style_for_line(" context"), unified_style_for_line("context"));
    let _ = render_unified_line("+new".to_string());
}
