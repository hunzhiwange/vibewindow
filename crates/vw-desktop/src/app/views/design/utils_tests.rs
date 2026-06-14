#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("utils_tests"));
}

#[test]
fn transparent_styles_remove_chrome_and_keep_text_visible() {
    let input = super::transparent_input_style(
        &iced::Theme::Dark,
        iced::widget::text_input::Status::Active,
    );
    assert_eq!(input.border.width, 0.0);
    assert_eq!(input.background, iced::Background::Color(iced::Color::TRANSPARENT));
    assert_ne!(input.value, iced::Color::TRANSPARENT);

    let editor = super::transparent_editor_style(iced::Color::from_rgb(0.2, 0.3, 0.4));
    assert_eq!(editor.border.width, 0.0);
    assert_eq!(editor.background, iced::Background::Color(iced::Color::TRANSPARENT));
    assert_eq!(editor.value, iced::Color::from_rgb(0.2, 0.3, 0.4));
}
