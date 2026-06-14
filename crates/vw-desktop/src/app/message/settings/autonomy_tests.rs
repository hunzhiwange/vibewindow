use super::*;
use crate::app::App;
use vw_config_types::security::{
    AutonomyLevel, NonCliNaturalLanguageApprovalMode, ShellRedirectPolicy,
};

fn app() -> App {
    App::new().0
}

#[test]
fn parses_non_cli_mode_by_channel_with_aliases() {
    let parsed = parse_non_cli_mode_by_channel(
        " Telegram : direct, slack:request-confirm\nDiscord:disabled\nbad-entry\nqq:unknown",
    );

    assert_eq!(parsed.get("telegram"), Some(&NonCliNaturalLanguageApprovalMode::Direct));
    assert_eq!(parsed.get("slack"), Some(&NonCliNaturalLanguageApprovalMode::RequestConfirm));
    assert_eq!(parsed.get("discord"), Some(&NonCliNaturalLanguageApprovalMode::Disabled));
    assert!(!parsed.contains_key("qq"));
}

#[test]
fn update_tracks_field_changes_and_help_modal() {
    let mut app = app();
    app.autonomy_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::AutonomyLevelChanged(AutonomyLevel::Full));
    let _ = update(&mut app, SettingsMessage::AutonomyWorkspaceOnlyToggled(true));
    let _ = update(
        &mut app,
        SettingsMessage::AutonomyAllowedCommandsChanged("git status\ncargo test".to_string()),
    );
    let _ = update(
        &mut app,
        SettingsMessage::AutonomyForbiddenPathsChanged(" /tmp , /var ".to_string()),
    );
    let _ = update(&mut app, SettingsMessage::AutonomyMaxActionsPerHourChanged(0));
    let _ = update(&mut app, SettingsMessage::AutonomyMaxCostPerDayCentsChanged(2_000_000));
    let _ = update(
        &mut app,
        SettingsMessage::AutonomyShellRedirectPolicyChanged(ShellRedirectPolicy::Strip),
    );
    let _ = update(
        &mut app,
        SettingsMessage::AutonomyNonCliNaturalLanguageApprovalModeChanged(
            NonCliNaturalLanguageApprovalMode::RequestConfirm,
        ),
    );
    let _ = update(
        &mut app,
        SettingsMessage::AutonomyNonCliNaturalLanguageApprovalModeByChannelChanged(
            "slack:direct".to_string(),
        ),
    );

    assert_eq!(app.autonomy_settings.level, AutonomyLevel::Full);
    assert!(app.autonomy_settings.workspace_only);
    assert_eq!(app.autonomy_settings.allowed_commands_input, "git status\ncargo test");
    assert_eq!(app.autonomy_settings.forbidden_paths_input, " /tmp , /var ");
    assert_eq!(app.autonomy_settings.max_actions_per_hour, 1);
    assert_eq!(app.autonomy_settings.max_cost_per_day_cents, 1_000_000);
    assert_eq!(app.autonomy_settings.shell_redirect_policy, ShellRedirectPolicy::Strip);
    assert_eq!(
        app.autonomy_settings.non_cli_natural_language_approval_mode,
        NonCliNaturalLanguageApprovalMode::RequestConfirm
    );
    assert_eq!(
        app.autonomy_settings.non_cli_natural_language_approval_mode_by_channel_input,
        "slack:direct"
    );
    assert!(app.autonomy_settings.save_error.is_none());

    let _ = update(&mut app, SettingsMessage::AutonomyHelpOpen);
    assert!(app.autonomy_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::AutonomyHelpClose);
    assert!(!app.autonomy_settings.show_help_modal);
}
