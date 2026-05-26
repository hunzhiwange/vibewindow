#[test]
fn color_resolution_supports_known_tokens_and_scopes() {
    assert_eq!(super::TailwindColors::resolve_color_token("white"), Some(iced::Color::WHITE));
    assert_eq!(
        super::TailwindColors::resolve_background_color("blue-500"),
        Some(super::TailwindColors::BLUE_500)
    );
    assert_eq!(
        super::TailwindColors::resolve_text_color("gray-800"),
        Some(super::TailwindColors::GRAY_800)
    );
    assert_eq!(
        super::TailwindColors::resolve_border_color("red-500"),
        Some(super::TailwindColors::RED_500)
    );
}

#[test]
fn unsupported_color_tokens_are_rejected() {
    assert_eq!(super::TailwindColors::resolve_color_token("secret-token"), None);
}
