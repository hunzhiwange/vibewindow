#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("common_tests"));
}

#[test]
fn parse_clamped_u32_accepts_trimmed_values_and_applies_bounds() {
    assert_eq!(super::parse_clamped_u32(" 42 ", 7, 1, 100), 42);
    assert_eq!(super::parse_clamped_u32("0", 7, 1, 100), 1);
    assert_eq!(super::parse_clamped_u32("999", 7, 1, 100), 100);
}

#[test]
fn parse_clamped_u32_clamps_fallback_for_invalid_input() {
    assert_eq!(super::parse_clamped_u32("nope", 0, 3, 9), 3);
    assert_eq!(super::parse_clamped_u32("", 99, 3, 9), 9);
}

#[test]
fn parse_clamped_u64_accepts_trimmed_values_and_applies_bounds() {
    assert_eq!(super::parse_clamped_u64(" 123 ", 7, 10, 200), 123);
    assert_eq!(super::parse_clamped_u64("9", 7, 10, 200), 10);
    assert_eq!(super::parse_clamped_u64("201", 7, 10, 200), 200);
}

#[test]
fn parse_clamped_u64_clamps_fallback_for_invalid_input() {
    assert_eq!(super::parse_clamped_u64("nan", 1, 5, 8), 5);
    assert_eq!(super::parse_clamped_u64("nan", 99, 5, 8), 8);
}

#[test]
fn trim_to_option_discards_surrounding_whitespace_and_empty_values() {
    assert_eq!(super::trim_to_option("  value  ".to_string()), Some("value".to_string()));
    assert_eq!(super::trim_to_option("\n\t ".to_string()), None);
}

#[test]
fn clear_new_session_picker_messages_preserves_picker_selection() {
    let (mut app, _task) = crate::app::App::new();
    app.new_session_picker_project = Some("/project".to_string());
    app.new_session_picker_options = vec![("/project".to_string(), "Main".to_string())];
    app.new_session_worktree_name = "feature".to_string();
    app.new_session_confirm_delete_directory = Some("/project/wt".to_string());
    app.new_session_force_delete_directory = Some("/project/wt".to_string());
    app.new_session_delete_error = Some("delete failed".to_string());
    app.new_session_confirm_reset_directory = Some("/project/wt".to_string());
    app.new_session_reset_error = Some("reset failed".to_string());

    super::clear_new_session_picker_messages(&mut app);

    assert_eq!(app.new_session_picker_project.as_deref(), Some("/project"));
    assert_eq!(app.new_session_picker_options.len(), 1);
    assert_eq!(app.new_session_worktree_name, "feature");
    assert!(app.new_session_confirm_delete_directory.is_none());
    assert!(app.new_session_force_delete_directory.is_none());
    assert!(app.new_session_delete_error.is_none());
    assert!(app.new_session_confirm_reset_directory.is_none());
    assert!(app.new_session_reset_error.is_none());
}

#[test]
fn reset_new_session_picker_state_clears_selection_options_and_messages() {
    let (mut app, _task) = crate::app::App::new();
    app.new_session_picker_project = Some("/project".to_string());
    app.new_session_picker_options = vec![("/project".to_string(), "Main".to_string())];
    app.new_session_worktree_name = "feature".to_string();
    app.new_session_confirm_delete_directory = Some("/project/wt".to_string());
    app.new_session_force_delete_directory = Some("/project/wt".to_string());
    app.new_session_delete_error = Some("delete failed".to_string());
    app.new_session_confirm_reset_directory = Some("/project/wt".to_string());
    app.new_session_reset_error = Some("reset failed".to_string());

    super::reset_new_session_picker_state(&mut app);

    assert!(app.new_session_picker_project.is_none());
    assert!(app.new_session_picker_options.is_empty());
    assert!(app.new_session_worktree_name.is_empty());
    assert!(app.new_session_confirm_delete_directory.is_none());
    assert!(app.new_session_force_delete_directory.is_none());
    assert!(app.new_session_delete_error.is_none());
    assert!(app.new_session_confirm_reset_directory.is_none());
    assert!(app.new_session_reset_error.is_none());
}
