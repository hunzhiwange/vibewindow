use super::{icon_btn, icon_svg, provider_logo_svg};
use crate::app::Message;
use crate::app::assets::Icon;
use iced::{Element, Length};

fn assert_size(element: &Element<'_, Message>, width: Length, height: Length) {
    let size = element.as_widget().size();

    assert_eq!(size.width, width);
    assert_eq!(size.height, height);
}

#[test]
fn icon_svg_uses_requested_square_size() {
    let element: Element<'_, Message> = icon_svg(Icon::Gear, 18.0).into();

    assert_size(&element, Length::Fixed(18.0), Length::Fixed(18.0));
}

#[test]
fn provider_logo_svg_uses_requested_square_size() {
    let element: Element<'_, Message> = provider_logo_svg("openai", 22.0).into();

    assert_size(&element, Length::Fixed(22.0), Length::Fixed(22.0));
}

#[test]
fn provider_logo_svg_accepts_unknown_provider_ids() {
    let element: Element<'_, Message> = provider_logo_svg("unknown-provider", 1.0).into();

    assert_size(&element, Length::Fixed(1.0), Length::Fixed(1.0));
}

#[test]
fn icon_svg_accepts_zero_size_without_clamping() {
    let element: Element<'_, Message> = icon_svg(Icon::Gear, 0.0).into();

    assert_size(&element, Length::Fixed(0.0), Length::Fixed(0.0));
}

#[test]
fn icon_button_wraps_content_in_tooltip_when_enabled() {
    let element = icon_btn(Icon::Gear, "Settings", Some(Message::None));

    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn icon_button_wraps_content_in_tooltip_when_disabled() {
    let element = icon_btn(Icon::QuestionCircle, "Help", None);

    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn icon_button_keeps_shrink_size_inside_tooltip() {
    let element = icon_btn(Icon::Gear, "Settings", Some(Message::None));

    assert_size(&element, Length::Shrink, Length::Shrink);
}
