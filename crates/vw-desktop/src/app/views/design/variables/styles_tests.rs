#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("styles_tests"));
}

#[test]
fn variables_palette_and_surfaces_follow_light_and_dark_themes() {
    let dark = super::variables_palette(&iced::Theme::Dark);
    let light = super::variables_palette(&iced::Theme::Light);

    assert!(dark.panel_bg.r < light.panel_bg.r);
    assert!(dark.title.r > dark.subtitle.r);
    assert!(light.danger_text.r > light.danger_text.g);

    let panel = super::panel_surface_style(&iced::Theme::Light);
    assert!(panel.background.is_some());
    assert_eq!(panel.border.width, 1.0);

    let backdrop = super::backdrop_style(&iced::Theme::Dark);
    assert!(backdrop.background.is_some());

    let menu = super::menu_surface_style(&iced::Theme::Dark);
    assert!(menu.background.is_some());
}

#[test]
fn variable_input_and_menu_styles_reflect_focus_hover_and_destructive_state() {
    let active = super::variable_text_input_style(
        &iced::Theme::Light,
        iced::widget::text_input::Status::Active,
    );
    let focused = super::variable_text_input_style(
        &iced::Theme::Light,
        iced::widget::text_input::Status::Focused { is_hovered: false },
    );
    assert_ne!(active.border.color, focused.border.color);

    let value_focused = super::variable_value_input_style(
        &iced::Theme::Dark,
        iced::widget::text_input::Status::Focused { is_hovered: true },
    );
    assert_eq!(value_focused.border.width, 1.0);

    let normal_button =
        super::menu_button_style(false)(&iced::Theme::Light, iced::widget::button::Status::Active);
    let destructive_hover =
        super::menu_button_style(true)(&iced::Theme::Light, iced::widget::button::Status::Hovered);
    assert!(normal_button.background.is_some());
    assert!(destructive_hover.background.is_some());
    assert_ne!(normal_button.text_color, destructive_hover.text_color);
}
