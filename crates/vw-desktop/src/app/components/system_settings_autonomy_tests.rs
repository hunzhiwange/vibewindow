use super::*;
use crate::app::{App, Message};
use iced::widget::text;
use vw_config_types::security::{
    AutonomyLevel, NonCliNaturalLanguageApprovalMode, ShellRedirectPolicy,
};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn system_settings_autonomy_tests_are_wired() {
    assert!(module_path!().contains("system_settings_autonomy_tests"));
}

#[test]
fn view_builds_all_level_policy_mode_and_error_branches() {
    let mut app = test_app();

    for level in [AutonomyLevel::ReadOnly, AutonomyLevel::Supervised, AutonomyLevel::Full] {
        app.autonomy_settings.level = level;
        keep_element(view(&app));
    }

    app.autonomy_settings.workspace_only = false;
    app.autonomy_settings.require_approval_for_medium_risk = false;
    app.autonomy_settings.block_high_risk_commands = false;
    app.autonomy_settings.shell_redirect_policy = ShellRedirectPolicy::Strip;
    app.autonomy_settings.non_cli_natural_language_approval_mode =
        NonCliNaturalLanguageApprovalMode::Disabled;
    keep_element(view(&app));

    app.autonomy_settings.non_cli_natural_language_approval_mode =
        NonCliNaturalLanguageApprovalMode::RequestConfirm;
    app.autonomy_settings.allowed_commands_input = "git\ncargo".to_string();
    app.autonomy_settings.forbidden_paths_input = "/etc\n/root".to_string();
    app.autonomy_settings.shell_env_passthrough_input = "RUST_LOG".to_string();
    app.autonomy_settings.auto_approve_input = "file_read".to_string();
    app.autonomy_settings.always_ask_input = "shell".to_string();
    app.autonomy_settings.allowed_roots_input = "/workspace".to_string();
    app.autonomy_settings.non_cli_excluded_tools_input = "browser".to_string();
    app.autonomy_settings.non_cli_approval_approvers_input = "telegram:alice".to_string();
    app.autonomy_settings.non_cli_natural_language_approval_mode_by_channel_input =
        "telegram:direct".to_string();
    app.autonomy_settings.max_actions_per_hour = 10_000;
    app.autonomy_settings.max_cost_per_day_cents = 1_000_000;
    app.autonomy_settings.save_error = Some("save failed".to_string());
    keep_element(view(&app));
}

#[test]
fn overlays_return_dialog_or_help_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.autonomy_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}
