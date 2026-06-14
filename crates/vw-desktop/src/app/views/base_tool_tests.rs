#[test]
fn bases_vec_contains_supported_range() {
    let bases = super::bases_vec();
    assert_eq!(bases.first().copied(), Some(2));
    assert_eq!(bases.last().copied(), Some(36));
    assert_eq!(bases.len(), 35);
}

#[test]
fn digit_char_and_valid_digits_cover_supported_and_fallback_ranges() {
    assert_eq!(super::digit_char(0), '0');
    assert_eq!(super::digit_char(9), '9');
    assert_eq!(super::digit_char(10), 'A');
    assert_eq!(super::digit_char(35), 'Z');
    assert_eq!(super::digit_char(36), '?');

    assert_eq!(super::valid_digits_label(2), "0-1");
    assert_eq!(super::valid_digits_label(10), "0-9");
    assert_eq!(super::valid_digits_label(16), "0-9 / A-F");
    assert_eq!(super::valid_digits_label(99), "0-9");
}

#[test]
fn build_status_badge_covers_idle_success_and_error_states() {
    let mut app = crate::app::App::new().0;
    let _ = super::build_status_badge(&app);

    app.base_input = "1010".to_string();
    let _ = super::build_status_badge(&app);

    app.base_notification = Some("已复制结果".to_string());
    let _ = super::build_status_badge(&app);

    app.base_notification = Some("超出范围".to_string());
    let _ = super::build_status_badge(&app);
}

#[test]
fn workspace_builders_cover_compact_and_wide_layouts() {
    let mut app = crate::app::App::new().0;
    app.base_input = "ff".to_string();
    app.base_output = "255".to_string();
    app.base_from = 16;
    app.base_to = 10;

    let _ = super::view(&app);
    let _ = super::build_workspace(&app, iced::Size::new(900.0, 700.0));
    let _ = super::build_workspace(&app, iced::Size::new(1200.0, 700.0));
    let _ = super::build_conversion_panel(&app, iced::Size::new(800.0, 700.0));
    let _ = super::build_conversion_panel(&app, iced::Size::new(1200.0, 700.0));
    let _ = super::build_side_panel(&app);
}

#[test]
fn number_card_builders_cover_source_target_and_copy_disabled_state() {
    let mut app = crate::app::App::new().0;
    let _ = super::build_number_card(&app, super::NumberCard::Source, true);
    let _ = super::build_number_card(&app, super::NumberCard::Target, false);
    let _ = super::build_copy_button(&app);

    app.base_output = "1010".to_string();
    let _ = super::build_copy_button(&app);
    let _ = super::build_base_selector(16, true, false);
    let _ = super::build_base_selector(2, false, true);
    let _ = super::build_base_button(8, true, true);
    let _ = super::build_base_button(10, false, false);
    let _ = super::build_swap_bridge();
    let _ = super::build_metric_badge("3 字符".to_string());
    let _ = super::status_row("源进制", "16 进制");
    let _ = super::form_row("进制", "说明", iced::widget::text("control"), true);
    let _ = super::form_row("进制", "说明", iced::widget::text("control"), false);
}
