#![allow(unused_must_use)]
#[test]
fn tool_detail_tests_module_is_wired() {
    assert!(module_path!().ends_with("tool_detail_tests"));
}

#[test]
fn tool_detail_from_raw_uses_output_text_for_success_payload() {
    let raw = r#"tool shell
{"status":"success","output":"done\n"}"#;

    let (title, content) = super::tool_detail_from_raw(raw).expect("valid tool detail");

    assert!(!title.is_empty());
    assert_eq!(content, "done");
}

#[test]
fn tool_detail_from_raw_marks_error_title_and_uses_error_text() {
    let raw = r#"tool shell
{"status":"error","error":"permission denied"}"#;

    let (title, content) = super::tool_detail_from_raw(raw).expect("valid tool detail");

    assert!(title.ends_with("失败"));
    assert_eq!(content, "permission denied");
}

#[test]
fn tool_detail_from_raw_marks_running_title() {
    let raw = r#"tool shell
{"status":"running","output":""}"#;

    let (title, _) = super::tool_detail_from_raw(raw).expect("valid tool detail");

    assert!(title.ends_with("运行中"));
}

#[test]
fn tool_detail_from_raw_falls_back_to_trimmed_rest_for_non_json() {
    let raw = "tool unknown\n  plain output  ";

    let (title, content) = super::tool_detail_from_raw(raw).expect("valid tool detail");

    assert_eq!(title, "工具");
    assert_eq!(content, "plain output");
}

#[test]
fn tool_detail_from_raw_rejects_missing_tool_header_or_body() {
    assert!(super::tool_detail_from_raw("tool shell").is_none());
    assert!(super::tool_detail_from_raw("shell\n{}").is_none());
}

#[test]
fn close_tool_detail_context_menu_clears_open_state_and_position() {
    let (mut app, _task) = crate::app::App::new();
    super::handle_open_tool_detail(&mut app, 2, 3, "tool shell\n{}".to_string());
    let dialog = app.tool_detail_dialog.as_mut().expect("dialog opens");
    dialog.context_menu_open = true;
    dialog.context_menu_pos = Some((12.0, 24.0));

    super::close_tool_detail_context_menu(dialog);

    assert!(!dialog.context_menu_open);
    assert_eq!(dialog.context_menu_pos, None);
}

#[test]
fn handle_open_tool_detail_sets_dialog_for_valid_raw_and_ignores_invalid_raw() {
    let (mut app, _task) = crate::app::App::new();

    super::handle_open_tool_detail(
        &mut app,
        4,
        5,
        "tool shell\n{\"status\":\"success\",\"output\":\"hello\"}".to_string(),
    );

    let dialog = app.tool_detail_dialog.as_ref().expect("dialog opens");
    assert_eq!(dialog.msg_idx, 4);
    assert_eq!(dialog.tool_idx, 5);
    assert_eq!(dialog.content, "hello");
    assert_eq!(dialog.editor.text(), "hello");

    super::handle_open_tool_detail(&mut app, 1, 1, "not a tool".to_string());

    assert_eq!(app.tool_detail_dialog.as_ref().expect("dialog remains").msg_idx, 4);
}

#[test]
fn tool_detail_scroll_helpers_clamp_to_available_lines() {
    let (mut app, _task) = crate::app::App::new();
    app.current_line_height = 10.0;
    super::handle_open_tool_detail(&mut app, 0, 0, "tool shell\nline1\nline2\nline3\nline4".into());
    app.tool_detail_dialog.as_mut().expect("dialog").viewport_height = 20.0;

    assert_eq!(super::tool_detail_max_scroll_top_line(&app), 2.0);

    super::apply_tool_detail_scroll_lines(&mut app, 10);
    assert_eq!(app.tool_detail_dialog.as_ref().expect("dialog").scroll_top_line, 2.0);

    super::apply_tool_detail_scroll_lines(&mut app, -10);
    assert_eq!(app.tool_detail_dialog.as_ref().expect("dialog").scroll_top_line, 0.0);
}
