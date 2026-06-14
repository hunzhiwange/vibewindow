use iced::Length;

use super::drag::{drag_spacer, traffic_light_spacer};

#[cfg(target_os = "macos")]
#[test]
fn traffic_light_spacer_reserves_macos_window_controls_width() {
    let element = traffic_light_spacer();
    let size = element.as_widget().size();

    assert_eq!(size.width, Length::Fixed(75.0));
    assert_eq!(size.height, Length::Fill);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn traffic_light_spacer_collapses_without_macos_window_controls() {
    let element = traffic_light_spacer();
    let size = element.as_widget().size();

    assert_eq!(size.width, Length::Fixed(0.0));
}

#[test]
fn drag_spacer_fills_available_top_bar_space() {
    let element = drag_spacer();
    let size = element.as_widget().size();

    assert_eq!(size.width, Length::Fill);
    assert_eq!(size.height, Length::Fill);
}
