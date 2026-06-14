use iced::widget::{button, text};
use iced::widget::tooltip::Position as TooltipPosition;
use iced::{Color, Element, Length, Theme};

use super::widgets::{
    color_with_alpha, icon_button, icon_svg, icon_toggle_button, icon_toggle_button_opt, menu_btn,
    menu_container, menu_item_btn, menu_item_icon_btn, menu_separator,
};
use crate::app::assets::Icon;
use crate::app::message::view::MenuType;
use crate::app::Message;

fn assert_size(element: &Element<'_, Message>, width: Length, height: Length) {
    let size = element.as_widget().size();

    assert_eq!(size.width, width);
    assert_eq!(size.height, height);
}

#[test]
fn color_with_alpha_preserves_rgb_and_replaces_alpha() {
    let color = Color { r: 0.2, g: 0.4, b: 0.6, a: 0.8 };

    assert_eq!(color_with_alpha(color, 0.35), Color { a: 0.35, ..color });
}

#[test]
fn icon_svg_uses_compact_toolbar_size() {
    let element: Element<'_, Message> = icon_svg(Icon::Gear).into();

    assert_size(&element, Length::Fixed(14.0), Length::Fixed(14.0));
}

#[test]
fn icon_button_builds_tooltip_wrapped_button() {
    let element = icon_button(
        Icon::Gear,
        "Settings",
        TooltipPosition::Bottom,
        Message::None,
    );

    assert_size(&element, Length::Shrink, Length::Fixed(24.0));
    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn icon_toggle_button_builds_active_tooltip_wrapped_button() {
    let element = icon_toggle_button(
        Icon::LayoutSidebar,
        "Toggle sidebar",
        TooltipPosition::Bottom,
        true,
        Message::None,
    );

    assert_size(&element, Length::Shrink, Length::Fixed(24.0));
    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn icon_toggle_button_opt_without_message_still_renders_disabled_button() {
    let element = icon_toggle_button_opt(
        Icon::LayoutSidebarReverse,
        "Toggle panel",
        TooltipPosition::Bottom,
        false,
        None,
    );

    assert_size(&element, Length::Shrink, Length::Fixed(24.0));
    assert_eq!(element.as_widget().children().len(), 2);
}

#[test]
fn menu_btn_fills_top_bar_height_for_inactive_menu() {
    let element = menu_btn("File", MenuType::File, None);

    assert_size(&element, Length::Shrink, Length::Fill);
}

#[test]
fn menu_btn_fills_top_bar_height_for_active_menu() {
    let element = menu_btn("View", MenuType::View, Some(MenuType::View));

    assert_size(&element, Length::Shrink, Length::Fill);
}

#[test]
fn menu_container_uses_fixed_menu_width() {
    let content: Element<'_, Message> = text("content").into();
    let element = menu_container(content);

    assert_size(&element, Length::Fixed(220.0), Length::Shrink);
}

#[test]
fn menu_item_btn_with_message_fills_available_width() {
    let element = menu_item_btn("New session", Some("Cmd+N"), Some(Message::None));

    assert_size(&element, Length::Fill, Length::Shrink);
}

#[test]
fn menu_item_btn_without_message_fills_available_width() {
    let element = menu_item_btn("Unavailable", None, None);

    assert_size(&element, Length::Fill, Length::Shrink);
}

#[test]
fn menu_separator_fills_width_and_shrinks_height() {
    let element = menu_separator();

    assert_size(&element, Length::Shrink, Length::Shrink);
}

#[test]
fn menu_item_icon_btn_with_message_fills_available_width() {
    let icon: Element<'_, Message> = text("i").into();
    let element = menu_item_icon_btn(icon, "Open", Some("Cmd+O"), Some(Message::None));

    assert_size(&element, Length::Fill, Length::Shrink);
}

#[test]
fn menu_item_icon_btn_without_message_fills_available_width() {
    let icon: Element<'_, Message> = button(text("i")).into();
    let element = menu_item_icon_btn(icon, "Open", None, None);

    assert_size(&element, Length::Fill, Length::Shrink);
}
