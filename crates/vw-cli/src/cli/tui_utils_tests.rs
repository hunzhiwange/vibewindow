use super::tui_utils::{cursor_position, neon_breath_color};

#[test]
fn cursor_position_counts_lines_and_wide_chars() {
    assert_eq!(cursor_position("a你\nb", 2), (0, 3));
    assert_eq!(cursor_position("a你\nb", 4), (1, 1));
    assert_eq!(neon_breath_color(0), neon_breath_color(8));
}
