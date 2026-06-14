#[test]
fn formatted_outputs_preserve_alpha_percent() {
    let outputs = super::format_outputs(iced::Color::from_rgba8(0x11, 0x22, 0x33, 128.0 / 255.0));
    assert_eq!(outputs.hex, "#11223380");
    assert_eq!(outputs.alpha_percent, 50);
}

#[test]
fn hsla_and_hsva_format_values_with_rounded_components() {
    assert_eq!(super::format_hsla(154.4, 0.444, 0.466, 0.5), "hsla(154, 44%, 47%, 0.50)");
    assert_eq!(super::format_hsva(154.6, 0.614, 0.684, 0.25), "hsva(155, 61%, 68%, 0.25)");
}

#[test]
fn format_outputs_populates_all_formats() {
    let outputs = super::format_outputs(iced::Color::from_rgba8(67, 173, 127, 0.5));
    assert_eq!(outputs.hex, "#43AD7F80");
    assert!(outputs.rgb.starts_with("rgba("));
    assert!(outputs.hsl.starts_with("hsla("));
    assert!(outputs.hsv.starts_with("hsva("));
    assert!(outputs.hsv_info.h >= 0.0);
}

#[test]
fn color_view_builders_cover_responsive_layouts_and_rows() {
    let mut app = crate::app::App::new().0;
    app.color_hex_input = "#43ad7f".to_string();
    app.color_rgb_input = "rgba(67, 173, 127, 0.5)".to_string();
    app.color_hsl_input = "hsla(154, 44%, 47%, 0.5)".to_string();
    app.color_hsv_input = "hsva(154, 61%, 68%, 0.5)".to_string();

    let _ = super::view(&app);
    let _ = super::build_workspace(&app, iced::Size::new(900.0, 700.0));
    let _ = super::build_workspace(&app, iced::Size::new(1200.0, 700.0));
    let _ = super::build_preview_panel(&app);
    let _ = super::build_forms_panel(&app);
    let _ = super::build_input_row(
        "HEX",
        "desc",
        "placeholder",
        "#fff",
        |value| {
            crate::app::Message::ColorTool(crate::app::message::ColorToolMessage::HexInputChanged(
                value,
            ))
        },
        crate::app::Message::ColorTool(crate::app::message::ColorToolMessage::HexValidate),
    );
    let _ = super::build_output_row("HEX", "desc", "#FFFFFF".to_string());
    let _ = super::form_row("HEX", "desc", iced::widget::text("control"));
    let _ = super::readonly_value_field("#FFFFFF".to_string());
    let _ = super::preview_box(iced::Color::WHITE, 12.0, 16.0);
}

#[test]
fn status_badge_covers_idle_success_and_error() {
    let mut app = crate::app::App::new().0;
    let _ = super::build_status_badge(&app);

    app.color_notification = Some("已复制".to_string());
    let _ = super::build_status_badge(&app);

    app.color_notification = Some("格式错误".to_string());
    let _ = super::build_status_badge(&app);
}
