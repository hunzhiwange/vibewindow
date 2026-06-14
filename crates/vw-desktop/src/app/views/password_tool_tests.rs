use crate::app::App;
use crate::app::message::PasswordToolMessage;
use iced::Size;
use iced::widget::text_editor;

fn password_app() -> App {
    let (app, _task) = App::new();
    app
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("password_tool_tests"));
}

#[test]
fn selected_groups_and_pool_size_follow_enabled_charsets() {
    let mut app = password_app();
    app.pwd_digits = true;
    app.pwd_lowercase = false;
    app.pwd_uppercase = true;
    app.pwd_special = false;

    assert_eq!(super::selected_group_count(&app), 2);
    assert_eq!(
        super::selected_pool_size(&app),
        super::DIGITS_CHARSET.len() + super::UPPERCASE_CHARSET.len()
    );

    app.pwd_digits = false;
    app.pwd_uppercase = false;
    assert_eq!(super::selected_group_count(&app), 0);
    assert_eq!(super::selected_pool_size(&app), 0);

    app.pwd_digits = true;
    app.pwd_lowercase = true;
    app.pwd_uppercase = true;
    app.pwd_special = true;
    assert_eq!(super::selected_group_count(&app), 4);
    assert_eq!(
        super::selected_pool_size(&app),
        super::DIGITS_CHARSET.len()
            + super::LOWERCASE_CHARSET.len()
            + super::UPPERCASE_CHARSET.len()
            + super::SPECIAL_CHARSET.len()
    );
}

#[test]
fn normalized_inputs_apply_defaults_and_bounds() {
    let mut app = password_app();

    app.pwd_length_input = "0".to_string();
    app.pwd_count_input = "0".to_string();
    assert_eq!(super::normalized_length(&app), 1);
    assert_eq!(super::normalized_count(&app), 1);

    app.pwd_length_input = "not-a-number".to_string();
    app.pwd_count_input = "not-a-number".to_string();
    assert_eq!(super::normalized_length(&app), 12);
    assert_eq!(super::normalized_count(&app), 1);

    app.pwd_length_input = "32".to_string();
    app.pwd_count_input = "900".to_string();
    assert_eq!(super::normalized_length(&app), 32);
    assert_eq!(super::normalized_count(&app), 500);

    app.pwd_length_input = String::new();
    app.pwd_count_input = String::new();
    assert_eq!(super::normalized_length(&app), 12);
    assert_eq!(super::normalized_count(&app), 1);

    app.pwd_length_input = " 16 ".to_string();
    app.pwd_count_input = " 2 ".to_string();
    assert_eq!(super::normalized_length(&app), 12);
    assert_eq!(super::normalized_count(&app), 1);
}

#[test]
fn output_password_count_ignores_blank_lines() {
    let mut app = password_app();
    app.pwd_output_editor = text_editor::Content::with_text("one\n\n  \ntwo\nthree");

    assert_eq!(super::output_password_count(&app), 3);
}

#[test]
fn password_rule_hint_reports_missing_charset() {
    let mut app = password_app();
    app.pwd_digits = false;
    app.pwd_lowercase = false;
    app.pwd_uppercase = false;
    app.pwd_special = false;

    let (hint, is_error) = super::password_rule_hint(&app);

    assert!(is_error);
    assert_eq!(hint, "至少选择一种字符集后才能生成密码。");
}

#[test]
fn password_rule_hint_reports_length_shorter_than_group_count() {
    let mut app = password_app();
    app.pwd_digits = true;
    app.pwd_lowercase = true;
    app.pwd_uppercase = true;
    app.pwd_special = true;
    app.pwd_length_input = "3".to_string();

    let (hint, is_error) = super::password_rule_hint(&app);

    assert!(is_error);
    assert_eq!(hint, "当前长度不足，至少需要 4 位才能覆盖所有已选字符集。");
}

#[test]
fn password_rule_hint_reports_valid_selected_groups() {
    let mut app = password_app();
    app.pwd_digits = true;
    app.pwd_lowercase = true;
    app.pwd_uppercase = false;
    app.pwd_special = false;
    app.pwd_length_input = "8".to_string();

    let (hint, is_error) = super::password_rule_hint(&app);

    assert!(!is_error);
    assert_eq!(hint, "每条密码都会至少包含 2 类已选字符。");
}

#[test]
fn workspace_builds_for_wide_and_narrow_sizes() {
    let app = password_app();

    let _wide = super::build_workspace(&app, Size::new(1200.0, 720.0));
    let _narrow = super::build_workspace(&app, Size::new(640.0, 720.0));
}

#[test]
fn controls_panel_builds_compact_and_regular_states() {
    let mut app = password_app();
    app.pwd_output_editor = text_editor::Content::with_text("generated-password");

    let _regular = super::build_controls_panel(&app, Size::new(960.0, 720.0));
    let _compact = super::build_controls_panel(&app, Size::new(640.0, 720.0));
}

#[test]
fn editor_card_builds_with_metrics() {
    let mut app = password_app();
    app.pwd_output_editor = text_editor::Content::with_text("one\ntwo\nthree");
    app.pwd_viewport_height = 120.0;
    app.current_line_height = 20.0;
    app.pwd_context_menu_open = true;
    app.pwd_context_menu_pos = Some((12.0, 24.0));

    let _card = super::build_editor_card(&app, Size::new(900.0, 600.0));
    let _panel = super::build_editor_panel(&app, Size::new(900.0, 600.0));
}

#[test]
fn status_badge_builds_idle_success_and_error_variants() {
    let mut app = password_app();
    let _idle = super::build_status_badge(&app);

    app.pwd_notification = Some("生成成功".to_string());
    app.pwd_notification_is_error = false;
    let _success = super::build_status_badge(&app);

    app.pwd_notification = Some("至少选择一种字符集".to_string());
    app.pwd_notification_is_error = true;
    let _error = super::build_status_badge(&app);
}

#[test]
fn row_builders_cover_compact_and_regular_layouts() {
    let _overview_regular =
        super::build_overview_row("字符池大小", "按已选字符集汇总。", "62 字符".to_string(), false);
    let _overview_compact =
        super::build_overview_row("字符池大小", "按已选字符集汇总。", "62 字符".to_string(), true);

    let _charset_regular = super::build_charset_row(
        "数字",
        "0-9 数字字符。",
        super::DIGITS_CHARSET,
        true,
        PasswordToolMessage::ToggleDigits,
        false,
    );
    let _charset_compact = super::build_charset_row(
        "数字",
        "0-9 数字字符。",
        super::DIGITS_CHARSET,
        false,
        PasswordToolMessage::ToggleDigits,
        true,
    );

    let _input_regular = super::build_input_row(
        "密码长度",
        "建议至少 12 位。",
        "12",
        "12",
        PasswordToolMessage::LengthChanged,
        false,
    );
    let _input_compact = super::build_input_row(
        "密码长度",
        "建议至少 12 位。",
        "12",
        "12",
        PasswordToolMessage::LengthChanged,
        true,
    );
}

#[test]
fn action_rows_cover_enabled_disabled_and_layout_variants() {
    let _regular = super::build_actions_row(false, false);
    let _compact = super::build_actions_row(true, true);

    let _primary = super::build_action_button(
        "生成密码",
        PasswordToolMessage::Generate,
        super::primary_action_btn_style,
        false,
    );
    let _disabled = super::build_action_button(
        "复制结果",
        PasswordToolMessage::Copy,
        super::rounded_action_btn_style,
        true,
    );
    let _danger = super::build_action_button(
        "清空结果",
        PasswordToolMessage::Clear,
        super::danger_action_btn_style,
        false,
    );
}

#[test]
fn hint_and_badge_builders_cover_error_and_neutral_paths() {
    let _hint_ok =
        super::build_hint_row("每条密码都会至少包含 1 类已选字符。".to_string(), false, false);
    let _hint_error =
        super::build_hint_row("至少选择一种字符集后才能生成密码。".to_string(), true, true);
    let _form_regular =
        super::build_form_row("标签", "说明", iced::widget::text("控件"), false);
    let _form_compact =
        super::build_form_row("标签", "说明", iced::widget::text("控件"), true);
    let _title = super::build_section_title("结果");
    let _metric = super::build_metric_badge("3 条".to_string());
}

#[test]
fn full_view_builds_from_app_state() {
    let app = password_app();

    let _view = super::view(&app);
}
