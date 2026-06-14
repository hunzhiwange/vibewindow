use iced::{Background, Theme};

#[test]
fn card_style_uses_theme_background_border_and_shadow() {
    let style = super::card_style(&Theme::Dark);

    assert!(matches!(style.background, Some(Background::Color(_))));
    assert_eq!(style.border.width, 1.0);
    assert!(style.border.radius.top_left > 0.0);
    assert_eq!(style.shadow.offset.y, 10.0);
    assert_eq!(style.shadow.blur_radius, 30.0);
}
