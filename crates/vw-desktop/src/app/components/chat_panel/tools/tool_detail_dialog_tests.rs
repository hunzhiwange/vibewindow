use super::tool_detail_dialog::tool_detail_dialog_view;
use crate::app::App;
use crate::app::state::ToolDetailDialog;
use iced::widget::{Id, text_editor};

fn app() -> App {
    App::new().0
}

#[test]
fn tool_detail_dialog_test_module_is_linked() {
    assert_eq!("tool_detail_dialog", "tool_detail_dialog");
}

#[test]
fn tool_detail_dialog_is_absent_without_state_and_present_with_state() {
    let mut app = app();
    assert!(tool_detail_dialog_view(&app).is_none());

    app.tool_detail_dialog = Some(ToolDetailDialog {
        msg_idx: 1,
        tool_idx: 2,
        title: "Tool output".to_string(),
        content: "line 1\nline 2".to_string(),
        editor: text_editor::Content::with_text("line 1\nline 2"),
        editor_id: Id::unique(),
        context_menu_open: true,
        context_menu_pos: Some((10.0, 20.0)),
        scroll_top_line: 3.0,
        scroll_remainder: 0.0,
        viewport_height: 100.0,
    });

    assert!(tool_detail_dialog_view(&app).is_some());
}
