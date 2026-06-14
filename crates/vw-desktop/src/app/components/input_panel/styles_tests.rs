use super::styles::*;
use iced::widget::{button, text_editor};
use iced::{Background, Color, Theme};

fn bg_color(background: Option<Background>) -> Color {
    match background {
        Some(Background::Color(color)) => color,
        other => panic!("expected color background, got {other:?}"),
    }
}

#[test]
fn task_738_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("styles_tests.rs"));
}

#[test]
fn bottom_bar_constants_keep_compact_control_dimensions() {
    assert_eq!(BOTTOM_BAR_ICON_BUTTON_SIZE, 24.0);
    assert_eq!(BOTTOM_BAR_ICON_SIZE, 12.0);
    assert_eq!(BOTTOM_BAR_LARGE_ICON_SIZE, 14.0);
    assert_eq!(BOTTOM_BAR_CHEVRON_ICON_SIZE, 11.0);
    assert_eq!(BOTTOM_BAR_LABEL_SIZE, 12.0);
}

#[test]
fn tooltip_and_popover_styles_have_expected_surface_treatment() {
    for theme in [Theme::Light, Theme::Dark] {
        let tooltip = tooltip_dark_style(&theme);
        assert_eq!(bg_color(tooltip.background), Color::from_rgba8(12, 13, 15, 0.97));
        assert_eq!(tooltip.text_color, Some(Color::WHITE));
        assert_eq!(tooltip.border.width, 1.0);
        assert!(tooltip.shadow.blur_radius > 0.0);

        let popover = popover_style(&theme);
        assert!(popover.background.is_some());
        assert_eq!(popover.border.width, 1.0);
        assert!(popover.shadow.blur_radius > tooltip.shadow.blur_radius);
    }
}

#[test]
fn square_and_round_icon_buttons_change_background_by_status_and_enabled_text_alpha() {
    for theme in [Theme::Light, Theme::Dark] {
        for enabled in [true, false] {
            let active = square_icon_button_style(&theme, button::Status::Active, enabled);
            let hovered = square_icon_button_style(&theme, button::Status::Hovered, enabled);
            let pressed = square_icon_button_style(&theme, button::Status::Pressed, enabled);
            let disabled = square_icon_button_style(&theme, button::Status::Disabled, enabled);

            assert_ne!(active.background, hovered.background);
            assert_ne!(hovered.background, pressed.background);
            assert_eq!(active.background, disabled.background);
            assert_eq!(active.border.width, 1.0);

            let round = round_icon_button_style(&theme, button::Status::Hovered, enabled);
            assert!(round.background.is_some());
            assert_eq!(round.border.width, 0.0);
        }

        let enabled = square_icon_button_style(&theme, button::Status::Active, true);
        let disabled = square_icon_button_style(&theme, button::Status::Active, false);
        assert!(enabled.text_color.a > disabled.text_color.a);
    }
}

#[test]
fn selector_text_and_chevron_colors_reflect_highlight_state() {
    for theme in [Theme::Light, Theme::Dark] {
        assert_ne!(selector_text_color(&theme, true), selector_text_color(&theme, false));
        assert_ne!(selector_chevron_color(&theme, true), selector_chevron_color(&theme, false));
        assert_eq!(selector_label_font().weight, iced::font::Weight::Medium);
    }
}

#[test]
fn selector_pill_style_covers_highlight_and_status_variants() {
    for theme in [Theme::Light, Theme::Dark] {
        for highlighted in [true, false] {
            let active = selector_pill_button_style(&theme, button::Status::Active, highlighted);
            let hovered = selector_pill_button_style(&theme, button::Status::Hovered, highlighted);
            let pressed = selector_pill_button_style(&theme, button::Status::Pressed, highlighted);

            assert!(active.background.is_some());
            assert_ne!(active.background, hovered.background);
            assert_ne!(hovered.background, pressed.background);
            assert_eq!(active.border.width, 1.0);

            if highlighted {
                assert!(active.shadow.blur_radius > 0.0);
            } else {
                assert_eq!(active.shadow.blur_radius, 0.0);
            }
        }
    }
}

#[test]
fn selectable_list_style_distinguishes_selected_hovered_pressed_and_idle() {
    for theme in [Theme::Light, Theme::Dark] {
        let idle = selectable_list_button_style(&theme, button::Status::Active, false);
        let hovered = selectable_list_button_style(&theme, button::Status::Hovered, false);
        let pressed = selectable_list_button_style(&theme, button::Status::Pressed, false);
        let selected = selectable_list_button_style(&theme, button::Status::Active, true);

        assert!(idle.background.is_none());
        assert!(hovered.background.is_some());
        assert_eq!(hovered.background, pressed.background);
        assert!(selected.background.is_some());
        assert_eq!(idle.border.width, 0.0);
        assert_eq!(selected.border.width, 1.0);
        assert_ne!(idle.text_color, selected.text_color);
    }
}

#[test]
fn card_styles_change_for_drop_hover_and_manual_context() {
    for theme in [Theme::Light, Theme::Dark] {
        let normal = input_card_style(&theme, false);
        let hovered = input_card_style(&theme, true);
        assert_ne!(normal.background, hovered.background);
        assert_ne!(normal.border.color, hovered.border.color);
        assert!(normal.shadow.blur_radius > 0.0);

        let manual = manual_context_card_style(&theme);
        assert!(manual.background.is_some());
        assert_eq!(manual.border.width, 1.0);

        let idle = manual_context_card_button_style(&theme, button::Status::Active);
        let hover = manual_context_card_button_style(&theme, button::Status::Hovered);
        let pressed = manual_context_card_button_style(&theme, button::Status::Pressed);
        assert_ne!(idle.background, hover.background);
        assert_eq!(hover.background, pressed.background);
    }
}

#[test]
fn editor_styles_keep_transparent_main_editor_and_bordered_subtask_editor() {
    for theme in [Theme::Light, Theme::Dark] {
        let idle = editor_style(&theme, text_editor::Status::Active, false);
        let requesting =
            editor_style(&theme, text_editor::Status::Focused { is_hovered: false }, true);
        assert_eq!(idle.background, Background::Color(Color::TRANSPARENT));
        assert_eq!(idle.border.width, 0.0);
        assert_ne!(idle.value, requesting.value);
        assert!(idle.selection.a > 0.0);
        assert!(idle.placeholder.a > 0.0);

        let subtask = subtask_editor_style(&theme, text_editor::Status::Active);
        assert!(matches!(subtask.background, Background::Color(_)));
        assert_eq!(subtask.border.width, 1.0);
        assert!(subtask.placeholder.a > 0.0);
    }
}
