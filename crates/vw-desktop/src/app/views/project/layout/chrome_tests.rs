#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("chrome_tests"));
}

#[test]
fn right_column_chrome_builds_with_positive_radius() {
    let content = iced::widget::text("right panel").into();
    let chrome = super::right_column_chrome(content, 8.0);

    std::hint::black_box(chrome);
}

#[test]
fn right_column_chrome_builds_with_zero_radius() {
    let content = iced::widget::text("right panel").into();
    let chrome = super::right_column_chrome(content, 0.0);

    std::hint::black_box(chrome);
}

#[test]
fn right_column_chrome_builds_with_negative_radius() {
    let content = iced::widget::text("right panel").into();
    let chrome = super::right_column_chrome(content, -1.0);

    std::hint::black_box(chrome);
}

#[test]
fn overlay_divider_builds_when_spacing_is_below_hit_width() {
    let divider = super::overlay_divider(2.0);

    std::hint::black_box(divider);
}

#[test]
fn overlay_divider_builds_when_spacing_matches_hit_width() {
    let divider = super::overlay_divider(super::HResizeHandle::HIT_WIDTH);

    std::hint::black_box(divider);
}

#[test]
fn overlay_divider_builds_when_spacing_exceeds_hit_width() {
    let divider = super::overlay_divider(super::HResizeHandle::HIT_WIDTH + 8.0);

    std::hint::black_box(divider);
}
