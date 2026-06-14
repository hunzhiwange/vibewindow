use super::*;
use crate::app::App;
use vw_config_types::automation::ResearchTrigger;

fn app() -> App {
    App::new().0
}

#[test]
fn updates_research_settings_and_help() {
    let mut app = app();
    app.research_settings.save_error = Some("old".to_string());
    let _ = update(&mut app, SettingsMessage::ResearchEnabledToggled(true));
    assert!(app.research_settings.enabled);
    assert!(app.research_settings.save_error.is_none());
    let _ = update(&mut app, SettingsMessage::ResearchTriggerChanged(ResearchTrigger::Always));
    assert_eq!(app.research_settings.trigger, ResearchTrigger::Always);
    let _ = update(&mut app, SettingsMessage::ResearchKeywordsChanged("rust,ai".to_string()));
    let _ = update(&mut app, SettingsMessage::ResearchMinMessageLengthChanged(0));
    assert_eq!(app.research_settings.min_message_length, 1);
    let _ = update(&mut app, SettingsMessage::ResearchMinMessageLengthChanged(20_000));
    assert_eq!(app.research_settings.min_message_length, 10_000);
    let _ = update(&mut app, SettingsMessage::ResearchMaxIterationsChanged(0));
    let _ = update(&mut app, SettingsMessage::ResearchMaxIterationsChanged(200));
    assert_eq!(app.research_settings.max_iterations, 100);
    let _ = update(&mut app, SettingsMessage::ResearchShowProgressToggled(false));
    let _ = update(
        &mut app,
        SettingsMessage::ResearchSystemPromptPrefixChanged(" prefix ".to_string()),
    );
    assert!(!app.research_settings.show_progress);
    assert_eq!(app.research_settings.system_prompt_prefix, " prefix ");
    let _ = update(&mut app, SettingsMessage::ResearchSave);
    let _ = update(&mut app, SettingsMessage::ResearchHelpOpen);
    assert!(app.research_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::ResearchHelpClose);
    assert!(!app.research_settings.show_help_modal);
}
