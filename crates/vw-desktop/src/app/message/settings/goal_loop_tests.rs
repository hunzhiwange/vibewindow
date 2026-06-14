use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn goal_loop_parser_validates_positive_numbers() {
    assert_eq!(parse_positive_u32("7", "field").unwrap(), 7);
    assert!(parse_positive_u32("", "field").is_err());
    assert!(parse_positive_u32("0", "field").is_err());
    assert!(parse_positive_u32("x", "field").is_err());
}

#[test]
fn goal_loop_update_sets_errors_and_accepts_valid_values() {
    let mut app = app();

    let _ = update(
        &mut app,
        SettingsMessage::GoalLoop(GoalLoopMessage::IntervalMinutesChanged("0".to_string())),
    );
    assert!(
        app.goal_loop_settings.save_error.as_deref().unwrap_or("").contains("interval_minutes")
    );

    let _ = update(&mut app, SettingsMessage::GoalLoop(GoalLoopMessage::EnabledToggled(true)));
    let _ = update(
        &mut app,
        SettingsMessage::GoalLoop(GoalLoopMessage::IntervalMinutesChanged("15".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::GoalLoop(GoalLoopMessage::StepTimeoutSecsChanged("30".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::GoalLoop(GoalLoopMessage::MaxStepsPerCycleChanged("5".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::GoalLoop(GoalLoopMessage::ChannelChanged("slack".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::GoalLoop(GoalLoopMessage::TargetChanged("ops".to_string())),
    );

    assert!(app.goal_loop_settings.enabled);
    assert_eq!(app.goal_loop_settings.interval_minutes_input, "15");
    assert_eq!(app.goal_loop_settings.step_timeout_secs_input, "30");
    assert_eq!(app.goal_loop_settings.max_steps_per_cycle_input, "5");
    assert_eq!(app.goal_loop_settings.channel_input, "slack");
    assert_eq!(app.goal_loop_settings.target_input, "ops");
    assert!(app.goal_loop_settings.save_error.is_none());
}
