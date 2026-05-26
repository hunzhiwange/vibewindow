use super::theme::is_dark_theme;
use iced::Theme;

#[test]
fn is_dark_theme_distinguishes_builtin_themes() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
}
