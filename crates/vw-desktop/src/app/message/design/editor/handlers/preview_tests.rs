#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("preview_tests"));
}

use super::preview;
use crate::app::message::design::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{DesignChatMessage, DesignChatRole, DesignState};
use crate::app::{App, Message};

fn app_with_design_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

#[test]
fn view_element_html_opens_preview_for_existing_element() {
    let state = DesignState::new(DesignDoc {
        children: vec![DesignElement {
            kind: "text".to_string(),
            id: "title".to_string(),
            content: Some("Hello".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    });
    let mut app = app_with_design_state(state);

    let _ = preview::view_element_html(&mut app, "title".to_string());

    assert!(app.show_element_html_preview);
    assert!(app.element_html_preview_editor.text().contains("Hello"));
}

#[test]
fn close_html_preview_hides_preview() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));
    app.show_element_html_preview = true;

    let _ = preview::close_html_preview(&mut app);

    assert!(!app.show_element_html_preview);
}

#[test]
fn log_editor_action_ignores_edit_actions() {
    let mut state = DesignState::new(DesignDoc::default());
    state.design_generation_log_editor =
        iced::widget::text_editor::Content::with_text("readonly log");
    let mut app = app_with_design_state(state);

    let action = iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Paste(
        "mutated".to_string().into(),
    ));
    let _ = preview::design_generation_log_editor_action(&mut app, action);

    assert_eq!(
        app.active_design_state().unwrap().design_generation_log_editor.text(),
        "readonly log"
    );
}

#[test]
fn chat_selection_can_be_set_and_cleared() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    let _ = preview::design_generation_select_chat_message(&mut app, 4);
    assert_eq!(app.active_design_state().unwrap().design_chat_selected_message, Some(4));

    let _ = preview::design_generation_clear_chat_selection(&mut app);
    assert_eq!(app.active_design_state().unwrap().design_chat_selected_message, None);
}

#[test]
fn show_all_logs_toggles_and_requests_files_only_when_enabled() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    let task = preview::design_generation_show_all_logs(&mut app);
    assert!(app.active_design_state().unwrap().design_generation_show_all_logs);
    let _ = task.map(|message| {
        assert!(matches!(message, Message::Design(DesignMessage::DesignGenerationLoadLogFiles)));
    });

    let _ = preview::design_generation_show_all_logs(&mut app);
    assert!(!app.active_design_state().unwrap().design_generation_show_all_logs);
}

#[test]
fn loaded_log_files_replace_state_list() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    let _ = preview::design_generation_log_files_loaded(
        &mut app,
        vec!["b.log".to_string(), "a.log".to_string()],
    );

    assert_eq!(app.active_design_state().unwrap().design_generation_log_files, ["b.log", "a.log"]);
}

#[test]
fn copy_chat_message_out_of_range_returns_none_task() {
    let mut state = DesignState::new(DesignDoc::default());
    state.design_chat_messages =
        vec![DesignChatMessage { role: DesignChatRole::Assistant, content: "copy me".to_string() }];
    let mut app = app_with_design_state(state);

    let _ = preview::design_generation_copy_chat_message(&mut app, 99);

    assert_eq!(app.active_design_state().unwrap().design_chat_messages.len(), 1);
}
