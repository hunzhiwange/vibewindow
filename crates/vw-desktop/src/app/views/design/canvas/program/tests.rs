#[test]
fn frame_header_label_uses_default_for_blank_values() {
    assert_eq!(super::frame_header_label(None), "画板");
    assert_eq!(super::frame_header_label(Some("   ")), "画板");
    assert_eq!(super::frame_header_label(Some("Home")), "Home");
}

#[test]
fn mix_color_clamps_blend_amount() {
    let base = iced::Color::from_rgb(0.0, 0.0, 0.0);
    let overlay = iced::Color::from_rgb(1.0, 0.5, 0.25);
    let color = super::mix_color(base, overlay, 2.0);
    assert_eq!((color.r, color.g, color.b), (1.0, 0.5, 0.25));
}

#[test]
fn estimate_text_width_and_luminance_are_deterministic() {
    assert!((super::estimate_text_width("ab中", 10.0) - 20.6).abs() < 0.001);
    assert_eq!(super::estimate_text_width("", 10.0), 0.0);

    assert!((super::color_luminance(iced::Color::WHITE) - 1.0).abs() < 0.001);
    assert!((super::color_luminance(iced::Color::BLACK)).abs() < 0.001);
}

#[test]
fn mix_color_clamps_low_amount_and_alpha() {
    let base = iced::Color::from_rgba(0.2, 0.4, 0.6, 0.8);
    let overlay = iced::Color::from_rgba(1.0, 0.0, 0.0, 0.2);

    let color = super::mix_color(base, overlay, -1.0);
    assert_eq!((color.r, color.g, color.b, color.a), (0.2, 0.4, 0.6, 0.8));

    let color = super::mix_color(base, overlay, 0.5);
    assert!((color.r - 0.6).abs() < 0.001);
    assert!((color.g - 0.2).abs() < 0.001);
    assert!((color.b - 0.3).abs() < 0.001);
    assert!((color.a - 0.5).abs() < 0.001);
}

#[test]
fn frame_header_tooltips_change_when_not_using_move_tool() {
    use crate::app::views::design::models::DesignTool;

    assert_eq!(super::frame_header_tooltip_label(DesignTool::Move, "点击适配画板"), "点击适配画板");
    assert_eq!(
        super::frame_header_tooltip_label(DesignTool::Rectangle, "点击适配画板"),
        "点击适配画板（移动工具）"
    );
    assert_eq!(super::frame_header_tooltip_label(DesignTool::Rectangle, "other"), "other");
}

#[test]
fn frame_header_layout_caps_title_width_and_positions_button() {
    let rect = iced::Rectangle::new(iced::Point::new(20.0, 100.0), iced::Size::new(260.0, 120.0));
    let layout = super::frame_header_layout(rect, "Home");

    assert_eq!(layout.btn_rect.x, 20.0);
    assert_eq!(layout.btn_rect.y, 64.0);
    assert_eq!(layout.btn_rect.width, 26.0);
    assert_eq!(layout.title_rect.x, 52.0);
    assert!(layout.title_rect.width >= 104.0);
    assert_eq!(layout.title_rect.height, 26.0);

    let narrow = super::frame_header_layout(
        iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(80.0, 80.0)),
        "Very long title",
    );
    assert_eq!(narrow.title_rect.width, 56.0);
}

#[test]
fn fit_text_with_ellipsis_respects_width() {
    assert_eq!(super::fit_text_with_ellipsis("Home", 10.0, 100.0), "Home");
    assert_eq!(super::fit_text_with_ellipsis("Home", 10.0, 0.0), "...");
    assert_eq!(super::fit_text_with_ellipsis("Home", 10.0, 5.0), "...");
    assert_eq!(super::fit_text_with_ellipsis("Long title", 10.0, 35.0), "Lon...");
}

#[test]
fn interaction_for_handle_maps_resize_and_rotate_shapes() {
    use crate::app::views::design::canvas::types::Handle;
    use iced::mouse::Interaction;

    assert_eq!(super::interaction_for_handle(Handle::Top), Interaction::ResizingVertically);
    assert_eq!(super::interaction_for_handle(Handle::Left), Interaction::ResizingHorizontally);
    assert_eq!(super::interaction_for_handle(Handle::TopLeft), Interaction::ResizingDiagonallyDown);
    assert_eq!(super::interaction_for_handle(Handle::TopRight), Interaction::ResizingDiagonallyUp);
    assert_eq!(super::interaction_for_handle(Handle::RotateBottomRight), Interaction::Crosshair);
}
