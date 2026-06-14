//! 覆盖密码工具消息处理行为，验证生成参数和结果状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{
    DIGITS_CHARSET, LOWERCASE_CHARSET, PasswordToolMessage, SPECIAL_CHARSET, UPPERCASE_CHARSET,
    apply_scroll_lines, build_pool, generate_one, max_scroll_top_line, parse_count_input,
    parse_length_input, random_index, selected_charsets, update, visible_line_count,
};
use crate::app::App;
use iced::mouse;
use iced::widget::text_editor;
use rand::RngCore;
use rand::{SeedableRng, rngs::StdRng};

fn test_app() -> App {
    App::new().0
}

#[test]
fn build_pool_combines_selected_charsets() {
    let charsets = selected_charsets(true, false, true, false);
    let pool = build_pool(&charsets);

    assert_eq!(pool, [DIGITS_CHARSET.as_bytes(), UPPERCASE_CHARSET.as_bytes()].concat());
}

#[test]
fn generate_one_contains_each_selected_charset() {
    let charsets = selected_charsets(true, true, true, true);
    let pool = build_pool(&charsets);
    let mut rng = StdRng::seed_from_u64(7);

    let password = generate_one(16, &pool, &charsets, &mut rng).expect("password should generate");

    assert_eq!(password.len(), 16);
    assert!(password.chars().any(|ch| DIGITS_CHARSET.contains(ch)));
    assert!(password.chars().any(|ch| LOWERCASE_CHARSET.contains(ch)));
    assert!(password.chars().any(|ch| UPPERCASE_CHARSET.contains(ch)));
    assert!(password.chars().any(|ch| SPECIAL_CHARSET.contains(ch)));
    assert!(password.bytes().all(|byte| pool.contains(&byte)));
}

#[test]
fn selected_charsets_is_empty_when_nothing_enabled() {
    assert!(selected_charsets(false, false, false, false).is_empty());
}

#[test]
fn parse_inputs_apply_documented_defaults_and_bounds() {
    assert_eq!(parse_length_input(""), 12);
    assert_eq!(parse_length_input("0"), 1);
    assert_eq!(parse_length_input("24"), 24);
    assert_eq!(parse_count_input(""), 1);
    assert_eq!(parse_count_input("0"), 1);
    assert_eq!(parse_count_input("999"), 500);
}

#[test]
fn random_index_handles_small_bounds_and_rejection_loop() {
    struct FixedRng {
        values: Vec<u32>,
    }

    impl RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 {
            self.values.remove(0)
        }

        fn next_u64(&mut self) -> u64 {
            self.next_u32() as u64
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.try_fill_bytes(dest).expect("fixed rng bytes");
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            let bytes = self.next_u32().to_le_bytes();
            dest.copy_from_slice(&bytes[..dest.len()]);
            Ok(())
        }
    }

    assert_eq!(random_index(&mut FixedRng { values: vec![] }, 0).expect("zero"), 0);
    assert_eq!(random_index(&mut FixedRng { values: vec![] }, 1).expect("one"), 0);
    assert_eq!(random_index(&mut FixedRng { values: vec![u32::MAX, 5] }, 3).expect("index"), 2);
}

#[test]
fn scroll_helpers_clamp_output_editor_position() {
    let mut app = test_app();
    app.pwd_output_editor = text_editor::Content::with_text("1\n2\n3\n4\n5");
    app.current_line_height = 10.0;
    app.pwd_viewport_height = 20.0;

    assert_eq!(visible_line_count(&app), 2.0);
    assert_eq!(max_scroll_top_line(&app), 3.0);

    apply_scroll_lines(&mut app, 99);
    assert_eq!(app.pwd_scroll_top_line, 3.0);
    apply_scroll_lines(&mut app, -99);
    assert_eq!(app.pwd_scroll_top_line, 0.0);
}

#[test]
fn update_generates_valid_passwords_and_handles_validation_errors() {
    let mut app = test_app();

    app.pwd_digits = false;
    app.pwd_lowercase = false;
    app.pwd_uppercase = false;
    app.pwd_special = false;
    let _ = update(&mut app, PasswordToolMessage::Generate);
    assert_eq!(app.pwd_notification.as_deref(), Some("至少选择一种字符集"));
    assert!(app.pwd_notification_is_error);

    app.pwd_digits = true;
    app.pwd_lowercase = true;
    app.pwd_length_input = "1".to_string();
    let _ = update(&mut app, PasswordToolMessage::Generate);
    assert_eq!(app.pwd_notification.as_deref(), Some("密码长度至少为 2 位"));

    app.pwd_length_input = "8".to_string();
    app.pwd_count_input = "3".to_string();
    let _ = update(&mut app, PasswordToolMessage::Generate);
    let lines = app.pwd_output_editor.text();
    let passwords = lines.lines().collect::<Vec<_>>();
    assert_eq!(passwords.len(), 3);
    assert!(passwords.iter().all(|password| password.len() == 8));
    assert_eq!(app.pwd_notification.as_deref(), Some("生成成功"));
    assert!(!app.pwd_notification_is_error);
}

#[test]
fn update_toggles_inputs_scrolls_context_menu_and_clears() {
    let mut app = test_app();

    let _ = update(&mut app, PasswordToolMessage::ToggleDigits(false));
    let _ = update(&mut app, PasswordToolMessage::ToggleLowercase(false));
    let _ = update(&mut app, PasswordToolMessage::ToggleUppercase(true));
    let _ = update(&mut app, PasswordToolMessage::ToggleSpecial(true));
    let _ = update(&mut app, PasswordToolMessage::LengthChanged("18".to_string()));
    let _ = update(&mut app, PasswordToolMessage::CountChanged("5".to_string()));
    assert!(!app.pwd_digits);
    assert!(!app.pwd_lowercase);
    assert!(app.pwd_uppercase);
    assert!(app.pwd_special);
    assert_eq!(app.pwd_length_input, "18");
    assert_eq!(app.pwd_count_input, "5");

    let _ = update(&mut app, PasswordToolMessage::OpenContextMenu { x: 1.0, y: 2.0 });
    assert!(app.pwd_context_menu_open);
    assert_eq!(app.pwd_context_menu_pos, Some((1.0, 2.0)));

    app.pwd_output_editor = text_editor::Content::with_text("1\n2\n3\n4\n5");
    app.current_line_height = 10.0;
    let _ = update(
        &mut app,
        PasswordToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
            viewport_height: 20.0,
        },
    );
    assert!(!app.pwd_context_menu_open);
    assert!(app.pwd_scroll_top_line > 0.0);

    let _ = update(
        &mut app,
        PasswordToolMessage::ScrollbarChanged { top_line: 99.0, viewport_height: 20.0 },
    );
    assert_eq!(app.pwd_scroll_top_line, max_scroll_top_line(&app));

    let _ = update(&mut app, PasswordToolMessage::Copy);
    assert_eq!(app.pwd_notification.as_deref(), Some("已复制"));

    let _ = update(&mut app, PasswordToolMessage::Clear);
    assert!(app.pwd_output_editor.text().is_empty());
    assert_eq!(app.pwd_scroll_top_line, 0.0);
    assert_eq!(app.pwd_notification.as_deref(), Some("已清空"));

    let _ = update(&mut app, PasswordToolMessage::ClearNotification);
    assert!(app.pwd_notification.is_none());
    assert!(!app.pwd_notification_is_error);
}
