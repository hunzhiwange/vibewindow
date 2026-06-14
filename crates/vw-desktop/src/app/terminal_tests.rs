#[test]
fn shell_display_and_all_are_stable() {
    let shells = super::Shell::all();
    assert_eq!(shells, [super::Shell::Bash, super::Shell::Zsh]);
    assert_eq!(shells[0].to_string(), "bash");
    assert_eq!(shells[1].to_string(), "zsh");
    assert_eq!(super::TerminalTheme::System.to_string(), "跟随系统");
}

#[test]
fn truncate_string_preserves_utf8_boundary() {
    assert_eq!(super::truncate_string_to_limit("abc", 4), "abc");
    assert_eq!(super::truncate_string_to_limit("a你好b", 5), "好b");
    assert_eq!(super::truncate_string_to_limit("🙂abc", 4), "abc");
    assert_eq!(super::truncate_string_to_limit("abcdef", 0), "");
}

#[test]
fn truncate_terminal_content_keeps_last_four_thousand_bytes() {
    let mut content = iced::widget::text_editor::Content::with_text(&format!(
        "{}{}",
        "x".repeat(10),
        "y".repeat(4_000)
    ));

    super::truncate_terminal_content(&mut content);

    let text = content.text();
    assert_eq!(text.len(), 4_000);
    assert!(text.chars().all(|ch| ch == 'y'));
}

#[test]
fn default_terminal_state_uses_expected_settings() {
    let state = super::TerminalState::default();

    assert!(!state.is_visible);
    assert!(state.tabs.is_empty());
    assert_eq!(state.active_id, None);
    assert_eq!(state.next_id, 1);
    assert_eq!(state.height, 200.0);
    assert_eq!(state.shell, super::Shell::Bash);
    assert_eq!(state.theme, super::TerminalTheme::System);
    assert_eq!(state.font_family, "JetBrains Mono");
    assert_eq!(state.font_size, 13.0);
}

#[test]
fn blank_with_settings_preserves_user_preferences() {
    let state = super::TerminalState::blank_with_settings(
        true,
        super::Shell::Zsh,
        super::TerminalTheme::System,
        "Fira Code".into(),
        16.0,
        240.0,
    );

    assert!(state.is_visible);
    assert_eq!(state.shell, super::Shell::Zsh);
    assert_eq!(state.font_family, "Fira Code");
    assert_eq!(state.font_size, 16.0);
    assert_eq!(state.height, 240.0);
}

#[test]
fn selecting_unknown_terminal_still_opens_panel_and_restores_min_height() {
    let mut state = super::TerminalState::default();
    state.height = 120.0;

    state.select_terminal(99);

    assert_eq!(state.active_id, Some(99));
    assert!(state.is_visible);
    assert_eq!(state.height, 200.0);
}

#[test]
fn close_and_rename_unknown_terminal_are_noops_for_empty_state() {
    let mut state = super::TerminalState::default();

    state.close_terminal(1);
    state.start_rename(1);
    state.update_rename(1, "renamed".into());
    state.save_rename(1);
    state.cancel_rename(1);

    assert!(state.tabs.is_empty());
    assert_eq!(state.active_id, None);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn color_channels_are_clamped_before_hex_conversion() {
    assert_eq!(super::color_channel_to_u8(-1.0), 0);
    assert_eq!(super::color_channel_to_u8(2.0), 255);
    assert_eq!(super::to_hex(iced::Color::from_rgb(1.0, 0.5, 0.0)), "#ff8000");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn font_settings_scale_is_clamped_for_tiny_sizes() {
    let tiny = super::build_font_settings("JetBrains Mono", 1.0);
    let normal = super::build_font_settings("JetBrains Mono", 13.0);

    assert_eq!(tiny.size, 13.0);
    assert_eq!(tiny.scale_factor, 0.5);
    assert!(normal.scale_factor > tiny.scale_factor);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn enter_bindings_cover_enter_newline_and_carriage_return() {
    let bindings = super::enter_sends_cr_bindings();

    assert_eq!(bindings.len(), 6);
    assert!(
        bindings
            .iter()
            .all(|(_, action)| matches!(action, iced_term::bindings::BindingAction::Char('\n')))
    );
}
