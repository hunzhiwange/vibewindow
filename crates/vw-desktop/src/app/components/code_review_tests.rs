use super::code_review::{render_unified_line, unified_style_for_line};
use iced::Color;

#[test]
fn unified_line_style_distinguishes_add_delete_and_context() {
    assert_ne!(unified_style_for_line("+new"), unified_style_for_line("-old"));
    assert_eq!(unified_style_for_line(" context"), unified_style_for_line("context"));
    let _ = render_unified_line("+new".to_string());
}

#[test]
fn unified_line_style_handles_headers_and_hunks() {
    assert_eq!(
        unified_style_for_line("@@ -1 +1 @@"),
        (Color::from_rgb8(0x17, 0x1B, 0x22), Color::from_rgb8(0x79, 0xC0, 0xFF))
    );
    assert_eq!(
        unified_style_for_line("diff --git a/a.rs b/a.rs"),
        (Color::from_rgb8(0x0D, 0x11, 0x17), Color::from_rgb8(0xC9, 0xD1, 0xD9))
    );
}
