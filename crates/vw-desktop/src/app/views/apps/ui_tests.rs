#[test]
fn dark_theme_detection_matches_theme_variant() {
    assert!(super::is_dark_theme(&iced::Theme::Dark));
    assert!(!super::is_dark_theme(&iced::Theme::Light));
}
