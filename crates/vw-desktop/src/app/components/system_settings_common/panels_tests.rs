use super::{
    settings_divider, settings_error_banner, settings_modal_card, settings_modal_overlay,
    settings_page_intro, settings_panel, settings_section_card, settings_success_banner,
    settings_value_badge,
};
use crate::app::Message;
use iced::widget::text;
use iced::{Element, Length};

fn assert_size(element: &Element<'_, Message>, width: Length, height: Length) {
    let size = element.as_widget().size();

    assert_eq!(size.width, width);
    assert_eq!(size.height, height);
}

#[test]
fn settings_panel_fills_available_width() {
    let element: Element<'_, Message> = settings_panel(text("panel")).into();

    assert_size(&element, Length::Fill, Length::Shrink);
}

#[test]
fn settings_modal_card_shrinks_to_content() {
    let element: Element<'_, Message> = settings_modal_card(text("card")).into();

    assert_size(&element, Length::Shrink, Length::Shrink);
}

#[test]
fn settings_modal_overlay_without_base_stacks_backdrop_and_card() {
    let element = settings_modal_overlay(None, Message::None, text("card"));

    assert_eq!(element.as_widget().children().len(), 2);
    assert_size(&element, Length::Fill, Length::Fill);
}

#[test]
fn settings_modal_overlay_with_base_adds_base_layer() {
    let base: Element<'_, Message> = text("base").into();
    let element = settings_modal_overlay(Some(base), Message::None, text("card"));

    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn settings_section_card_fills_width() {
    let element: Element<'_, Message> = settings_section_card("Title", "Description").into();

    assert_size(&element, Length::Fill, Length::Shrink);
}

#[test]
fn settings_page_intro_renders_title_and_description() {
    let element = settings_page_intro("Title", "Description");

    assert_eq!(element.as_widget().children().len(), 2);
    assert_size(&element, Length::Shrink, Length::Shrink);
}

#[test]
fn settings_divider_uses_fill_width_and_fixed_height() {
    let element = settings_divider();

    assert_size(&element, Length::Fill, Length::Fixed(1.0));
}

#[test]
fn settings_value_badge_shrinks_to_content() {
    let element = settings_value_badge("128K");

    assert_size(&element, Length::Shrink, Length::Shrink);
}

#[test]
fn settings_value_badge_stringifies_non_string_values() {
    let element = settings_value_badge(42);

    assert_size(&element, Length::Shrink, Length::Shrink);
}

#[test]
fn settings_banners_fill_available_width() {
    let error = settings_error_banner("bad");
    let success = settings_success_banner("ok");

    assert_size(&error, Length::Fill, Length::Shrink);
    assert_size(&success, Length::Fill, Length::Shrink);
}
