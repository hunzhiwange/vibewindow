use super::{danger_action_btn_style, primary_action_btn_style, rounded_action_btn_style};
use iced::Theme;
use iced::widget::button;

#[test]
fn action_button_styles_produce_text_colors_for_dark_theme() {
    let theme = Theme::Dark;

    assert_ne!(
        rounded_action_btn_style(&theme, button::Status::Active).text_color,
        iced::Color::TRANSPARENT
    );
    assert_ne!(
        primary_action_btn_style(&theme, button::Status::Active).text_color,
        iced::Color::TRANSPARENT
    );
    assert_ne!(
        danger_action_btn_style(&theme, button::Status::Active).text_color,
        iced::Color::TRANSPARENT
    );
}
