use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn clamps_scheduler_values_and_toggles_help() {
    let mut app = app();
    app.scheduler_settings.save_error = Some("old".to_string());
    let _ = update(&mut app, SettingsMessage::SchedulerEnabledToggled(true));
    assert!(app.scheduler_settings.enabled);
    assert!(app.scheduler_settings.save_error.is_none());
    let _ = update(&mut app, SettingsMessage::SchedulerMaxTasksChanged(0));
    assert_eq!(app.scheduler_settings.max_tasks, 1);
    let _ = update(&mut app, SettingsMessage::SchedulerMaxTasksChanged(50_000));
    assert_eq!(app.scheduler_settings.max_tasks, 10_000);
    let _ = update(&mut app, SettingsMessage::SchedulerMaxConcurrentChanged(0));
    assert_eq!(app.scheduler_settings.max_concurrent, 1);
    let _ = update(&mut app, SettingsMessage::SchedulerMaxConcurrentChanged(200));
    assert_eq!(app.scheduler_settings.max_concurrent, 100);
    let _ = update(&mut app, SettingsMessage::SchedulerSave);
    let _ = update(&mut app, SettingsMessage::SchedulerHelpOpen);
    assert!(app.scheduler_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::SchedulerHelpClose);
    assert!(!app.scheduler_settings.show_help_modal);
}
