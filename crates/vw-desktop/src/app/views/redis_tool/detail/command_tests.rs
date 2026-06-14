use crate::app::state::RedisCommandOutputEntry;
use crate::app::{App, Message};
use iced::{Element, Theme};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("command_tests"));
}

#[test]
fn command_panel_builds_empty_output_without_selected_connection() {
    let mut app = test_app();
    app.redis_tool.command_input = "INFO server".to_string();

    keep_element(super::build_command_panel(&app, false));
}

#[test]
fn command_panel_builds_output_history_for_selected_connection() {
    let mut app = test_app();
    app.redis_tool.selected_connection_id = Some("redis-local".to_string());
    app.redis_tool.command_input = "GET cache:key".to_string();
    app.redis_tool.command_output = vec![
        RedisCommandOutputEntry {
            command: "PING".to_string(),
            output: "PONG".to_string(),
            cost_ms: 3,
            is_error: false,
            time_ms: 1_725_000_000_000,
        },
        RedisCommandOutputEntry {
            command: "GET missing".to_string(),
            output: "ERR value is not available".to_string(),
            cost_ms: 7,
            is_error: true,
            time_ms: 1_725_000_000_100,
        },
    ];

    keep_element(super::build_command_panel(&app, false));
    keep_element(super::build_command_panel(&app, true));
}

#[test]
fn command_output_entry_builds_success_and_error_variants() {
    let success = RedisCommandOutputEntry {
        command: "SET cache:key value".to_string(),
        output: "OK".to_string(),
        cost_ms: 2,
        is_error: false,
        time_ms: 1_725_000_001_000,
    };
    let error = RedisCommandOutputEntry {
        command: "HGETALL broken".to_string(),
        output: "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        cost_ms: 5,
        is_error: true,
        time_ms: 1_725_000_001_500,
    };

    keep_element(super::build_command_output_entry(&success));
    keep_element(super::build_command_output_entry(&error));
}

#[test]
fn command_output_styles_cover_dark_success_and_error_states() {
    let success = super::command_success_text_style(&Theme::Dark);
    let error = super::command_error_text_style(&Theme::Dark);
    let card = super::command_output_entry_style(&Theme::Dark);

    assert_ne!(success.color, error.color);
    assert!(card.background.is_some());
    assert_eq!(card.border.width, 1.0);
    assert!(card.text_color.is_some());
}
