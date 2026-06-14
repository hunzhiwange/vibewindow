use super::ChatMessage;
use crate::app::TodoPanelPlacement;

#[test]
fn input_changed_updates_legacy_input_text() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, ChatMessage::InputChanged("hello".to_string()));

    assert_eq!(app.input_text, "hello");
}

#[test]
fn scroll_changed_clamps_offsets_and_updates_autoscroll() {
    let (mut app, _) = crate::app::App::new();
    app.chat_auto_scroll = false;

    super::update(&mut app, ChatMessage::ScrollChanged { offset_y: 1.5, viewport_h: -10.0 });

    assert_eq!(app.chat_scroll_offset_y, 1.0);
    assert_eq!(app.chat_scroll_viewport_h, 0.0);
    assert!(app.chat_auto_scroll);
}

#[test]
fn fullscreen_toggles_clear_competing_git_fullscreen_modes() {
    let (mut app, _) = crate::app::App::new();
    app.git_diff_fullscreen = true;
    app.git_diff_half_fullscreen = true;

    super::update(&mut app, ChatMessage::ToggleFullscreen);

    assert!(app.chat_panel_fullscreen);
    assert!(!app.chat_panel_half_fullscreen);
    assert!(!app.git_diff_fullscreen);
    assert!(!app.git_diff_half_fullscreen);
    assert!(app.fullscreen_layout_settling);
}

#[test]
fn half_fullscreen_toggles_clear_fullscreen_mode() {
    let (mut app, _) = crate::app::App::new();
    app.chat_panel_fullscreen = true;

    super::update(&mut app, ChatMessage::ToggleHalfFullscreen);

    assert!(app.chat_panel_half_fullscreen);
    assert!(!app.chat_panel_fullscreen);
    assert!(app.fullscreen_layout_settling);
}

#[test]
fn hover_messages_set_and_clear_state() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, ChatMessage::ThinkHover(2, 3));
    assert_eq!(app.chat_think_hovered_idx, Some((2_u64 << 32) | 3));
    super::update(&mut app, ChatMessage::ThinkHoverLeave);
    assert_eq!(app.chat_think_hovered_idx, None);

    super::update(&mut app, ChatMessage::ToolHover(4, 5));
    assert_eq!(app.chat_tool_hovered_idx, Some((4_u64 << 32) | 5));
    app.chat_tool_file_hovered = Some("file".to_string());
    super::update(&mut app, ChatMessage::ToolHoverLeave);
    assert_eq!(app.chat_tool_hovered_idx, None);
    assert_eq!(app.chat_tool_file_hovered, None);
}

#[test]
fn toggle_sets_add_and_remove_keys() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, ChatMessage::ToggleToolFile(1, 2, "src/main.rs".to_string()));
    assert!(app.chat_tool_file_expanded.contains("1:2:src/main.rs"));
    super::update(&mut app, ChatMessage::ToggleToolFile(1, 2, "src/main.rs".to_string()));
    assert!(!app.chat_tool_file_expanded.contains("1:2:src/main.rs"));

    super::update(&mut app, ChatMessage::ToggleTool(3, 4));
    assert!(app.chat_tool_expanded.contains(&((3_u64 << 32) | 4)));
    super::update(&mut app, ChatMessage::ToggleExploreSummary(5, 6));
    assert!(app.chat_explore_expanded.contains(&((5_u64 << 32) | 6)));
}

#[test]
fn todo_panel_messages_update_state_and_animation() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, ChatMessage::SetTodoPanelPlacement(TodoPanelPlacement::ChatTopRight));
    assert_eq!(app.chat_todo_placement, TodoPanelPlacement::ChatTopRight);
    assert!(app.chat_todo_expanded);
    assert_eq!(app.chat_todo_anim, 1.0);

    super::update(&mut app, ChatMessage::ToggleTodoPanel);
    assert!(!app.chat_todo_expanded);
    super::update(&mut app, ChatMessage::TodoAnimTick);
    assert!(app.chat_todo_anim < 1.0);
}

#[test]
fn simple_input_flags_update_directly() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, ChatMessage::InputAreaDragHoverChanged(true));
    assert!(app.input_drop_hovered);
    super::update(&mut app, ChatMessage::FileReferenceHoverChanged(Some(2)));
    assert_eq!(app.file_ref_hovered_index, Some(2));
    super::update(&mut app, ChatMessage::ToolFilesFilterChanged("rs".to_string()));
    assert_eq!(app.tool_files_filter, "rs");
    super::update(&mut app, ChatMessage::FullscreenOverlayEntered);
    assert!(app.show_chat_fullscreen_overlay);
    super::update(&mut app, ChatMessage::FullscreenOverlayExited);
    assert!(!app.show_chat_fullscreen_overlay);
}
