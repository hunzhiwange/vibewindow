use super::*;
use crate::app::App;
use crate::app::state::{CronAddJobType, CronAddScheduleKind, CronSettingsTab};
use vw_gateway_client::{CronJobDto, CronRunDto};

fn app() -> App {
    App::new().0
}

fn job(id: &str) -> CronJobDto {
    CronJobDto {
        id: id.to_string(),
        name: Some(format!("job-{id}")),
        job_type: "shell".to_string(),
        schedule_kind: "cron".to_string(),
        expression: "* * * * *".to_string(),
        at: None,
        every_ms: None,
        command: "echo hi".to_string(),
        prompt: None,
        model: Some("openai/gpt-5".to_string()),
        agent: Some("main".to_string()),
        acp_agent: Some("codex".to_string()),
        project_path: Some("/tmp/project".to_string()),
        wake: false,
        fallbacks: vec!["slack".to_string()],
        full_access: false,
        task_pool: false,
        delivery_mode: "channel".to_string(),
        delivery_channel: Some("slack".to_string()),
        delivery_to: Some("#ops".to_string()),
        delivery_best_effort: true,
        delete_after_run: false,
        next_run: "2026-06-11T00:00:00Z".to_string(),
        last_run: None,
        last_status: None,
        last_output: None,
        enabled: true,
    }
}

#[test]
fn cron_helpers_format_history_and_infer_types() {
    let runs = vec![CronRunDto {
        id: 1,
        job_id: "job-1".to_string(),
        started_at: "start".to_string(),
        finished_at: "end".to_string(),
        status: "ok".to_string(),
        output: Some("hello".to_string()),
        duration_ms: Some(12),
    }];
    let text = format_run_history_text(&runs);
    assert!(text.contains("#1"));
    assert!(text.contains("状态: ok"));
    assert_eq!(schedule_kind_from_api("every", "", None, Some(1000)), CronAddScheduleKind::Every);
    assert_eq!(
        schedule_kind_from_api("指定时间", "", Some("10:00"), None),
        CronAddScheduleKind::At
    );
    assert_eq!(job_type_from_api(&job("1")), CronAddJobType::Shell);
    assert_eq!(model_provider_from_model("openai/gpt-5"), "openai");
}

#[test]
fn cron_update_tracks_selection_runs_and_help() {
    let mut app = app();
    app.cron_settings.selected_job_ids = vec!["missing".to_string()];

    let _ = update(&mut app, SettingsMessage::CronMaxRunHistoryChanged(0));
    assert_eq!(app.cron_settings.max_run_history, 1);

    let _ = update(&mut app, SettingsMessage::CronJobsLoaded(Ok(vec![job("a"), job("b")])));
    assert_eq!(app.cron_settings.jobs.len(), 2);
    assert!(app.cron_settings.selected_job_ids.is_empty());

    let _ = update(&mut app, SettingsMessage::CronJobSelectionToggled("a".to_string(), true));
    let _ = update(&mut app, SettingsMessage::CronJobsSelectAllToggled(true));
    assert_eq!(app.cron_settings.selected_job_ids.len(), 2);

    let _ = update(&mut app, SettingsMessage::CronJobRunsOpen("a".to_string()));
    assert_eq!(app.cron_settings.runs_modal_job_id.as_deref(), Some("a"));
    let _ = update(
        &mut app,
        SettingsMessage::CronJobRunsLoaded(
            "a".to_string(),
            Ok(vec![CronRunDto {
                id: 1,
                job_id: "a".to_string(),
                started_at: "start".to_string(),
                finished_at: "end".to_string(),
                status: "ok".to_string(),
                output: None,
                duration_ms: None,
            }]),
        ),
    );
    assert_eq!(app.cron_settings.runs_modal.len(), 1);
    assert!(app.cron_settings.runs_modal_editor.text().contains("状态: ok"));

    let _ = update(&mut app, SettingsMessage::CronHelpOpen);
    assert!(app.cron_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::CronHelpClose);
    assert!(!app.cron_settings.show_help_modal);

    let _ = update(&mut app, SettingsMessage::CronTabSelected(CronSettingsTab::Jobs));
    assert_eq!(app.cron_settings.active_tab, CronSettingsTab::Jobs);
}
