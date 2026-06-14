use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn coordination_update_clamps_values_and_toggles_help() {
    let mut app = app();
    app.coordination_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::CoordinationEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::CoordinationLeadAgentChanged(" main ".to_string()));
    let _ = update(&mut app, SettingsMessage::CoordinationMaxInboxMessagesPerAgentChanged(0));
    let _ = update(&mut app, SettingsMessage::CoordinationMaxDeadLettersChanged(20_000));
    let _ = update(&mut app, SettingsMessage::CoordinationMaxContextEntriesChanged(30_000));
    let _ = update(&mut app, SettingsMessage::CoordinationMaxSeenMessageIdsChanged(200_000));

    assert!(app.coordination_settings.enabled);
    assert_eq!(app.coordination_settings.lead_agent_input, " main ");
    assert_eq!(app.coordination_settings.max_inbox_messages_per_agent, 1);
    assert_eq!(app.coordination_settings.max_dead_letters, 10_000);
    assert_eq!(app.coordination_settings.max_context_entries, 20_000);
    assert_eq!(app.coordination_settings.max_seen_message_ids, 100_000);
    assert!(app.coordination_settings.save_error.is_none());

    let _ = update(&mut app, SettingsMessage::CoordinationHelpOpen);
    assert!(app.coordination_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::CoordinationHelpClose);
    assert!(!app.coordination_settings.show_help_modal);
}
