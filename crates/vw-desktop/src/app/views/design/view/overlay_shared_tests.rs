#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("overlay_shared_tests"));
}

#[test]
fn overlay_theme_helpers_choose_contrast_and_shadow_alpha() {
    assert_eq!(super::OVERLAY_ICON_PICKER_RESULT_LIMIT, 50);
    assert_eq!(super::OVERLAY_ICON_PICKER_REQUIRE_QUERY_THRESHOLD, 200);
    assert!(super::design_overlay_is_dark(&iced::Theme::Dark));
    assert!(!super::design_overlay_is_dark(&iced::Theme::Light));

    assert_eq!(super::design_overlay_contrast_text_color(iced::Color::WHITE), iced::Color::BLACK);
    assert_eq!(super::design_overlay_contrast_text_color(iced::Color::BLACK), iced::Color::WHITE);

    let dark_shadow = super::design_overlay_surface_shadow(&iced::Theme::Dark, 0.7, 0.2);
    let light_shadow = super::design_overlay_surface_shadow(&iced::Theme::Light, 0.7, 0.2);
    assert!((dark_shadow.a - 0.7).abs() < 0.001);
    assert!((light_shadow.a - 0.2).abs() < 0.001);
}
