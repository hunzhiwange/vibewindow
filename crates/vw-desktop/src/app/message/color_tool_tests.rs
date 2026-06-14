#![allow(unused_must_use)]
use crate::app::views::design::models::ColorFormat;
use crate::app::{App, Message};
use iced::Color;

fn new_app() -> App {
    App::new().0
}

#[test]
fn parse_hsl_accepts_hsl_and_hsla_with_clamps() {
    assert_eq!(super::parse_hsl("hsl(180, 50%, 75%)"), Some((180.0, 0.5, 0.75, 1.0)));
    assert_eq!(super::parse_hsl("hsla(-90, 125%, -5%, 1.5)"), Some((270.0, 1.0, 0.0, 1.0)));
}

#[test]
fn parse_hsl_rejects_malformed_input() {
    assert_eq!(super::parse_hsl("rgb(1, 2, 3)"), None);
    assert_eq!(super::parse_hsl("hsl(1, 2%)"), None);
    assert_eq!(super::parse_hsl("hsla(1, 2%, 3%, nope)"), None);
}

#[test]
fn parse_hsv_accepts_hsv_and_hsva_with_clamps() {
    assert_eq!(super::parse_hsv("hsv(360, 50%, 25%)"), Some((0.0, 0.5, 0.25, 1.0)));
    assert_eq!(super::parse_hsv("hsva(720, -10%, 150%, -1)"), Some((0.0, 0.0, 1.0, 0.0)));
}

#[test]
fn normalize_h_only_wraps_one_negative_turn_and_mods_positive_values() {
    assert_eq!(super::normalize_h(-90.0), 270.0);
    assert_eq!(super::normalize_h(720.0), 0.0);
    assert_eq!(super::normalize_h(42.0), 42.0);
}

#[test]
fn format_hsl_and_hsv_round_percent_components() {
    assert_eq!(super::format_hsla(12.4, 0.456, 0.789, 0.5), "hsla(12, 46%, 79%, 0.50)");
    assert_eq!(super::format_hsva(12.6, 0.454, 0.781, 1.0), "hsva(13, 45%, 78%, 1.00)");
}

#[test]
fn input_change_messages_update_buffers_without_validation() {
    let mut app = new_app();

    super::update(&mut app, super::ColorToolMessage::HexInputChanged("#112233".to_string()));
    super::update(&mut app, super::ColorToolMessage::RgbInputChanged("rgb(1,2,3)".to_string()));
    super::update(&mut app, super::ColorToolMessage::HslInputChanged("hsl(1,2%,3%)".to_string()));
    super::update(&mut app, super::ColorToolMessage::HsvInputChanged("hsv(4,5%,6%)".to_string()));

    assert_eq!(app.color_hex_input, "#112233");
    assert_eq!(app.color_rgb_input, "rgb(1,2,3)");
    assert_eq!(app.color_hsl_input, "hsl(1,2%,3%)");
    assert_eq!(app.color_hsv_input, "hsv(4,5%,6%)");
    assert!(app.color_notification.is_none());
}

#[test]
fn color_changed_syncs_all_format_inputs() {
    let mut app = new_app();

    super::update(
        &mut app,
        super::ColorToolMessage::ColorChanged(Color::from_rgba(0.25, 0.5, 0.75, 0.8)),
    );

    assert_eq!(app.color_tool_color, Color::from_rgba(0.25, 0.5, 0.75, 0.8));
    assert_eq!(app.color_hex_input, "#4080BFCC");
    assert_eq!(app.color_rgb_input, "rgba(64, 128, 191, 0.80)");
    assert!(app.color_hsl_input.starts_with("hsla("));
    assert!(app.color_hsv_input.starts_with("hsva("));
}

#[test]
fn format_change_and_clear_notification_update_simple_state() {
    let mut app = new_app();
    app.color_notification = Some("pending".to_string());

    super::update(&mut app, super::ColorToolMessage::ColorFormatChanged(ColorFormat::Rgba));
    super::update(&mut app, super::ColorToolMessage::ClearNotification);

    assert_eq!(app.color_tool_format, ColorFormat::Rgba);
    assert!(app.color_notification.is_none());
}

#[test]
fn validate_hex_rgb_hsl_and_hsv_update_color_or_report_errors() {
    let mut app = new_app();

    app.color_hex_input = " #010203 ".to_string();
    super::update(&mut app, super::ColorToolMessage::HexValidate);
    assert_eq!(app.color_hex_input, "#010203FF");
    assert_eq!(app.color_notification.as_deref(), Some("已更新颜色"));

    app.color_hex_input = "bad".to_string();
    super::update(&mut app, super::ColorToolMessage::HexValidate);
    assert_eq!(app.color_notification.as_deref(), Some("HEX 格式错误"));

    app.color_rgb_input = "rgba(10, 20, 30, 0.5)".to_string();
    super::update(&mut app, super::ColorToolMessage::RgbValidate);
    assert_eq!(app.color_rgb_input, "rgba(10, 20, 30, 0.50)");
    assert_eq!(app.color_notification.as_deref(), Some("已更新颜色"));

    app.color_rgb_input = "rgb(bad)".to_string();
    super::update(&mut app, super::ColorToolMessage::RgbValidate);
    assert_eq!(app.color_notification.as_deref(), Some("RGB/A 格式错误"));

    app.color_hsl_input = "hsla(120, 100%, 50%, 0.25)".to_string();
    super::update(&mut app, super::ColorToolMessage::HslValidate);
    assert_eq!(app.color_notification.as_deref(), Some("已更新颜色"));

    app.color_hsl_input = "hsl(1, two, 3%)".to_string();
    super::update(&mut app, super::ColorToolMessage::HslValidate);
    assert_eq!(app.color_notification.as_deref(), Some("HSL/A 格式错误"));

    app.color_hsv_input = "hsva(240, 100%, 100%, 0.75)".to_string();
    super::update(&mut app, super::ColorToolMessage::HsvValidate);
    assert_eq!(app.color_notification.as_deref(), Some("已更新颜色"));

    app.color_hsv_input = "hsv(1, two, 3%)".to_string();
    super::update(&mut app, super::ColorToolMessage::HsvValidate);
    assert_eq!(app.color_notification.as_deref(), Some("HSV/A 格式错误"));
}

#[test]
fn copy_sets_success_notification() {
    let mut app = new_app();

    let _task = super::update(&mut app, super::ColorToolMessage::Copy("#112233".to_string()));

    assert_eq!(app.color_notification.as_deref(), Some("已复制结果"));
}

#[test]
fn color_tool_message_can_be_nested_in_app_message() {
    let message = Message::ColorTool(super::ColorToolMessage::ClearNotification);

    assert!(matches!(message, Message::ColorTool(super::ColorToolMessage::ClearNotification)));
}
