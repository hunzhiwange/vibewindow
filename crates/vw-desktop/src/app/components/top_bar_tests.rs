use crate::app::{App, Message};
use iced::{Background, Theme};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn view_builds_top_bar_for_default_app() {
    let app = test_app();

    keep_element(super::top_bar::view(&app));
}

#[test]
fn top_bar_style_uses_theme_palette_with_expected_opacity() {
    for theme in [Theme::Light, Theme::Dark] {
        let palette = theme.extended_palette();
        let style = super::top_bar::top_bar_style(&theme);

        assert_eq!(style.border.width, 1.0);
        assert_eq!(style.border.radius.top_left, 0.0);
        assert_eq!(style.border.color.a, palette.background.strong.color.a * 0.60);

        let Some(Background::Color(background)) = style.background else {
            panic!("expected top bar style to set a color background");
        };

        assert_eq!(background.a, palette.background.weak.color.a * 0.88);
    }
}
