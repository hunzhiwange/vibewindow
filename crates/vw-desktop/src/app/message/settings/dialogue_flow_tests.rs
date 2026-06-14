use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn dialogue_flow_loaded_messages_update_ui_state() {
    let mut app = app();

    let _ = update(
        &mut app,
        SettingsMessage::DialogueFlowPermissionLoaded(Ok("{\n  \"allow\": true\n}".to_string())),
    );
    assert!(app.dialogue_flow_permission_editor.text().contains("\"allow\": true"));

    let _ =
        update(&mut app, SettingsMessage::DialogueFlowPermissionLoaded(Err("boom".to_string())));
    assert!(app.dialogue_flow_permission_editor.text().contains("读取失败: boom"));

    let _ =
        update(&mut app, SettingsMessage::DialogueFlowUiSettingsLoaded(Ok((false, true, true))));
    assert!(!app.dialogue_flow_show_reasoning_summary);
    assert!(app.dialogue_flow_expand_shell_tool_section);
    assert!(app.dialogue_flow_expand_edit_tool_section);
}

#[test]
fn dialogue_flow_toggle_and_save_feedback_update_messages() {
    let mut app = app();

    let _ = update(
        &mut app,
        SettingsMessage::DialogueFlowUiSettingsLoaded(Err("bad config".to_string())),
    );
    assert_eq!(
        app.dialogue_flow_settings_save_message.as_deref(),
        Some("读取配置失败: bad config")
    );

    let _ = update(&mut app, SettingsMessage::DialogueFlowShowReasoningSummaryToggled(true));
    let _ = update(&mut app, SettingsMessage::DialogueFlowExpandShellToolSectionToggled(false));
    let _ = update(&mut app, SettingsMessage::DialogueFlowExpandEditToolSectionToggled(false));
    assert!(app.dialogue_flow_show_reasoning_summary);
    assert!(!app.dialogue_flow_expand_shell_tool_section);
    assert!(!app.dialogue_flow_expand_edit_tool_section);

    let _ = update(&mut app, SettingsMessage::DialogueFlowUiSettingsSaved(Ok(())));
    assert_eq!(app.dialogue_flow_settings_save_message.as_deref(), Some("已保存对话流配置"));
    let _ = update(
        &mut app,
        SettingsMessage::DialogueFlowUiSettingsSaved(Err("write failed".to_string())),
    );
    assert_eq!(app.dialogue_flow_settings_save_message.as_deref(), Some("保存失败: write failed"));
}
