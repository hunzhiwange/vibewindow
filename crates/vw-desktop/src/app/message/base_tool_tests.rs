#![allow(unused_must_use)]
#[test]
fn sanitize_base_rejects_out_of_range_values() {
    assert_eq!(super::sanitize_base(1), 10);
    assert_eq!(super::sanitize_base(2), 2);
    assert_eq!(super::sanitize_base(36), 36);
    assert_eq!(super::sanitize_base(37), 10);
}

#[test]
fn sanitize_input_keeps_only_digits_valid_for_base() {
    assert_eq!(super::sanitize_input(" -10aZ ", 16), "-10a");
    assert_eq!(super::sanitize_input("--10102", 2), "-1010");
    assert_eq!(super::sanitize_input("xyz", 36), "xyz");
}

#[test]
fn convert_handles_negative_values_and_base_36_digits() {
    assert_eq!(super::convert("-ff", 16, 10).unwrap(), "-255");
    assert_eq!(super::convert("35", 10, 36).unwrap(), "Z");
    assert_eq!(super::convert("Z", 36, 10).unwrap(), "35");
}

#[test]
fn convert_reports_invalid_base_or_digit() {
    assert_eq!(super::convert("10", 1, 10).unwrap_err(), "仅支持 2-36 进制");
    assert_eq!(super::convert("2", 2, 10).unwrap_err(), "非法数字或超出范围");
}

#[test]
fn digit_helpers_cover_bounds() {
    assert_eq!(super::digit_value('0'), Some(0));
    assert_eq!(super::digit_value('A'), Some(10));
    assert_eq!(super::digit_value('z'), Some(35));
    assert_eq!(super::digit_value('_'), None);
    assert_eq!(super::digit_char(35), 'Z');
    assert_eq!(super::digit_char(36), '?');
}

#[test]
fn convert_handles_empty_zero_plus_and_overflow() {
    assert_eq!(super::convert("   ", 10, 2).unwrap(), "");
    assert_eq!(super::convert("0", 10, 2).unwrap(), "0");
    assert_eq!(super::convert("+15", 10, 16).unwrap(), "F");
    assert_eq!(
        super::convert("340282366920938463463374607431768211456", 10, 16).unwrap_err(),
        "非法数字或超出范围"
    );
}

#[test]
fn update_selects_bases_and_refreshes_output() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, super::BaseToolMessage::SelectFrom(16));
    super::update(&mut app, super::BaseToolMessage::InputChanged("ff".to_string()));
    super::update(&mut app, super::BaseToolMessage::SelectTo(2));

    assert_eq!(app.base_from, 16);
    assert_eq!(app.base_to, 2);
    assert_eq!(app.base_input, "ff");
    assert_eq!(app.base_output, "11111111");
    assert_eq!(app.base_notification, None);
}

#[test]
fn update_sanitizes_invalid_base_and_input() {
    let (mut app, _) = crate::app::App::new();
    app.base_from = 2;
    app.base_to = 10;

    super::update(&mut app, super::BaseToolMessage::InputChanged("102abc".to_string()));
    assert_eq!(app.base_input, "10");
    assert_eq!(app.base_output, "2");

    super::update(&mut app, super::BaseToolMessage::SelectFrom(99));
    assert_eq!(app.base_from, 10);
}

#[test]
fn update_swap_recomputes_from_new_source_base() {
    let (mut app, _) = crate::app::App::new();
    app.base_from = 10;
    app.base_to = 16;
    app.base_input = "255".to_string();
    app.base_output = "FF".to_string();

    super::update(&mut app, super::BaseToolMessage::Swap);

    assert_eq!(app.base_from, 16);
    assert_eq!(app.base_to, 10);
    assert_eq!(app.base_input, "FF");
    assert_eq!(app.base_output, "255");
}

#[test]
fn clear_notification_only_removes_copy_notification() {
    let (mut app, _) = crate::app::App::new();
    app.base_notification = Some("非法数字或超出范围".to_string());

    super::update(&mut app, super::BaseToolMessage::ClearNotification);
    assert_eq!(app.base_notification.as_deref(), Some("非法数字或超出范围"));

    app.base_notification = Some("已复制结果".to_string());
    super::update(&mut app, super::BaseToolMessage::ClearNotification);
    assert_eq!(app.base_notification, None);
}
