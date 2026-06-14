use super::{ExpandDirection, GitMessage, update};
use crate::app::state::{GitDiffLineRange, GitDiffSelectedLine};
use crate::app::{App, DiffTheme};

fn test_app() -> App {
    App::new().0
}

fn selected(file: &str, line: usize, is_old: bool, text: &str) -> GitDiffSelectedLine {
    GitDiffSelectedLine { file: file.to_string(), line, is_old, text: text.to_string() }
}

fn range(file: &str, start: usize, end: usize, is_old: bool) -> GitDiffLineRange {
    GitDiffLineRange { file: file.to_string(), start, end, is_old }
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("diff_tests"));
}

#[test]
fn toggle_diff_highlight_updates_flag() {
    let mut app = test_app();

    let _ = update(&mut app, GitMessage::ToggleDiffHighlight(false));
    assert!(!app.show_git_diff_highlight);

    let _ = update(&mut app, GitMessage::ToggleDiffHighlight(true));
    assert!(app.show_git_diff_highlight);
}

#[test]
fn toggle_diff_line_selection_adds_and_removes_same_line() {
    let mut app = test_app();

    let _ = update(
        &mut app,
        GitMessage::ToggleDiffLineSelection(
            "src/main.rs".to_string(),
            4,
            false,
            "+new".to_string(),
        ),
    );
    assert_eq!(app.git_diff_selected_lines.len(), 1);
    assert_eq!(app.git_diff_selected_lines[0].text, "+new");

    let _ = update(
        &mut app,
        GitMessage::ToggleDiffLineSelection(
            "src/main.rs".to_string(),
            4,
            false,
            "+new".to_string(),
        ),
    );
    assert_eq!(app.git_diff_selected_lines.len(), 0);
}

#[test]
fn drag_select_single_line_toggles_selection_and_double_click_opens_comment_draft() {
    let mut app = test_app();

    let _ = update(
        &mut app,
        GitMessage::DiffDragSelectStart("src/main.rs".to_string(), 10, false, "+line".to_string()),
    );
    let _ = update(&mut app, GitMessage::DiffDragSelectEnd);
    assert_eq!(app.git_diff_selected_lines.len(), 1);
    assert!(app.git_diff_selected_range.is_none());

    let _ = update(
        &mut app,
        GitMessage::DiffDragSelectStart("src/main.rs".to_string(), 10, false, "+line".to_string()),
    );
    let _ = update(&mut app, GitMessage::DiffDragSelectEnd);
    assert_eq!(app.git_diff_selected_lines.len(), 1);
    assert!(app.git_diff_comment_draft.is_some());
}

#[test]
fn drag_hover_only_updates_matching_active_range() {
    let mut app = test_app();

    let _ = update(
        &mut app,
        GitMessage::DiffDragSelectStart("src/main.rs".to_string(), 2, false, "+a".to_string()),
    );
    let _ = update(&mut app, GitMessage::DiffDragSelectHover("src/lib.rs".to_string(), 9, false));
    assert_eq!(app.git_diff_drag_range.as_ref().map(|value| value.end), Some(2));

    let _ = update(&mut app, GitMessage::DiffDragSelectHover("src/main.rs".to_string(), 5, false));
    assert_eq!(app.git_diff_drag_range.as_ref().map(|value| value.end), Some(5));
}

#[test]
fn context_menu_selects_line_when_nothing_is_selected() {
    let mut app = test_app();

    let _ = update(
        &mut app,
        GitMessage::OpenDiffContextMenu {
            file: "src/main.rs".to_string(),
            line: 8,
            is_old: true,
            text: "-old".to_string(),
            x: 12.0,
            y: 24.0,
        },
    );

    assert_eq!(app.git_diff_selected_lines.len(), 1);
    assert_eq!(app.git_diff_selected_range.as_ref().map(|value| value.start), Some(8));
    assert_eq!(app.git_diff_context_menu.as_ref().map(|value| value.x), Some(12.0));

    let _ = update(&mut app, GitMessage::CloseDiffContextMenu);
    assert!(app.git_diff_context_menu.is_none());
}

#[test]
fn file_menu_revert_and_cancel_discard_update_file_state() {
    let mut app = test_app();

    let _ = update(&mut app, GitMessage::OpenDiffFileMenu("src/main.rs".to_string()));
    assert_eq!(
        app.git_diff_file_menu.as_ref().map(|value| value.file.as_str()),
        Some("src/main.rs")
    );

    let _ = update(&mut app, GitMessage::RevertDiffFile("src/main.rs".to_string()));
    assert!(app.git_diff_file_menu.is_none());
    assert_eq!(app.file_to_discard.as_deref(), Some("src/main.rs"));

    let _ = update(&mut app, GitMessage::CancelDiscardFile);
    assert_eq!(app.file_to_discard, None);
}

#[test]
fn open_and_cancel_comment_draft_use_context_range() {
    let mut app = test_app();
    app.git_diff_selected_range = Some(range("src/main.rs", 3, 5, false));

    let _ = update(&mut app, GitMessage::OpenDiffCommentDraft);
    assert!(app.git_diff_comment_draft.is_some());
    assert_eq!(app.git_diff_comment_draft.as_ref().map(|draft| draft.range.end), Some(5));

    let _ = update(&mut app, GitMessage::DiffCommentCancel);
    assert!(app.git_diff_comment_draft.is_none());
    assert!(app.git_diff_selected_range.is_none());
}

#[test]
fn hover_exit_only_clears_matching_line() {
    let mut app = test_app();

    let _ = update(&mut app, GitMessage::DiffHoverEnter("a.rs".to_string(), 1, false));
    let _ = update(&mut app, GitMessage::DiffHoverExit("a.rs".to_string(), 2, false));
    assert_eq!(app.git_diff_hovered_line, Some(("a.rs".to_string(), 1, false)));

    let _ = update(&mut app, GitMessage::DiffHoverExit("a.rs".to_string(), 1, false));
    assert_eq!(app.git_diff_hovered_line, None);
}

#[test]
fn empty_copy_and_insert_selection_are_noops_but_close_context_menu() {
    let mut app = test_app();
    app.git_diff_context_menu = Some(crate::app::state::GitDiffContextMenuState {
        file: "a.rs".to_string(),
        line: 1,
        is_old: false,
        x: 0.0,
        y: 0.0,
    });

    let _ = update(&mut app, GitMessage::CopyDiffSelection);
    assert!(app.git_diff_context_menu.is_none());

    app.git_diff_context_menu = Some(crate::app::state::GitDiffContextMenuState {
        file: "a.rs".to_string(),
        line: 1,
        is_old: false,
        x: 0.0,
        y: 0.0,
    });
    let _ = update(&mut app, GitMessage::InsertDiffSelectionToChat);
    assert!(app.git_diff_context_menu.is_none());
}

#[test]
fn discard_selection_without_repo_clears_context_menu_only() {
    let mut app = test_app();
    app.project_path = None;
    app.git_diff_context_menu = Some(crate::app::state::GitDiffContextMenuState {
        file: "a.rs".to_string(),
        line: 1,
        is_old: false,
        x: 0.0,
        y: 0.0,
    });
    app.git_diff_selected_lines = vec![selected("a.rs", 1, false, "+line")];

    let _ = update(&mut app, GitMessage::DiscardDiffSelection);

    assert!(app.git_diff_context_menu.is_none());
    assert_eq!(app.git_diff_selected_lines.len(), 1);
}

#[test]
fn fullscreen_toggles_reset_competing_chat_fullscreen_state() {
    let mut app = test_app();
    app.chat_panel_fullscreen = true;
    app.chat_panel_half_fullscreen = true;

    let _ = update(&mut app, GitMessage::ToggleFullscreen);

    assert!(app.git_diff_fullscreen);
    assert!(!app.git_diff_half_fullscreen);
    assert!(!app.chat_panel_fullscreen);
    assert!(!app.chat_panel_half_fullscreen);
    assert!(app.show_diff);
    assert!(app.fullscreen_layout_settling);
}

#[test]
fn hunk_file_context_scroll_and_theme_state_are_updated() {
    let mut app = test_app();

    let _ = update(&mut app, GitMessage::ToggleExpandHunk("a.rs".to_string(), 2));
    assert_eq!(app.expanded_hunks, vec![("a.rs".to_string(), 2)]);
    let _ = update(&mut app, GitMessage::ToggleExpandHunk("a.rs".to_string(), 2));
    assert!(app.expanded_hunks.is_empty());

    let _ =
        update(&mut app, GitMessage::ExpandContext("a.rs".to_string(), 1, ExpandDirection::Down));
    let _ = update(&mut app, GitMessage::ExpandContext("a.rs".to_string(), 1, ExpandDirection::Up));
    assert_eq!(app.context_expansions.get(&("a.rs".to_string(), 1)), Some(&(20, 20)));

    let _ = update(&mut app, GitMessage::DiffScrollChanged { offset_y: 1.5, viewport_h: -10.0 });
    assert_eq!(app.git_diff_scroll_offset_y, 1.0);
    assert_eq!(app.git_diff_scroll_viewport_h, 0.0);

    let _ = update(&mut app, GitMessage::DiffThemeSelected(DiffTheme::Monokai));
    assert_eq!(app.diff_theme, DiffTheme::Monokai);
}
