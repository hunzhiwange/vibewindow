use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn clamps_reliability_values_and_toggles_help() {
    let mut app = app();
    app.reliability_settings.save_error = Some("old".to_string());
    let _ = update(&mut app, SettingsMessage::ReliabilityProviderRetriesChanged(99));
    assert_eq!(app.reliability_settings.provider_retries, 20);
    assert!(app.reliability_settings.save_error.is_none());
    let _ = update(&mut app, SettingsMessage::ReliabilityProviderBackoffMsChanged(99_999));
    assert_eq!(app.reliability_settings.provider_backoff_ms, 60_000);
    let _ = update(&mut app, SettingsMessage::ReliabilityChannelInitialBackoffSecsChanged(500));
    let _ = update(&mut app, SettingsMessage::ReliabilityChannelMaxBackoffSecsChanged(2));
    assert_eq!(app.reliability_settings.channel_max_backoff_secs, 500);
    let _ = update(&mut app, SettingsMessage::ReliabilitySchedulerPollSecsChanged(0));
    let _ = update(&mut app, SettingsMessage::ReliabilitySchedulerRetriesChanged(99));
    assert_eq!(app.reliability_settings.scheduler_poll_secs, 1);
    assert_eq!(app.reliability_settings.scheduler_retries, 20);
    let _ = update(&mut app, SettingsMessage::ReliabilitySave);
    let _ = update(&mut app, SettingsMessage::ReliabilityHelpOpen);
    assert!(app.reliability_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::ReliabilityHelpClose);
    assert!(!app.reliability_settings.show_help_modal);
}
