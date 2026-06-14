use super::*;
use iced::Element;
use crate::app::state::{CronAddJobType, CronAddScheduleKind, CronSettingsTab};
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

fn job(id: &str) -> CronJobDto {
    CronJobDto {
        id: id.to_string(),
        name: None,
        job_type: String::new(),
        schedule_kind: "cron".to_string(),
        expression: "0 9 * * *".to_string(),
        at: None,
        every_ms: None,
        command: "echo ok".to_string(),
        prompt: None,
        model: None,
        agent: None,
        acp_agent: None,
        project_path: None,
        wake: false,
        fallbacks: Vec::new(),
        full_access: false,
        task_pool: false,
        delivery_mode: String::new(),
        delivery_channel: None,
        delivery_to: None,
        delivery_best_effort: true,
        delete_after_run: false,
        next_run: "2026-06-12T01:02:03Z".to_string(),
        last_run: None,
        last_status: None,
        last_output: None,
        enabled: true,
    }
}

fn run(status: &str) -> CronRunDto {
    CronRunDto {
        id: 1,
        job_id: "job-1".to_string(),
        started_at: "2026-06-12T01:02:03Z".to_string(),
        finished_at: "2026-06-12T01:02:04Z".to_string(),
        status: status.to_string(),
        output: Some("ok".to_string()),
        duration_ms: Some(1000),
    }
}

#[test]
fn small_building_blocks_return_elements() {
    keep_element(field_row("标签", "说明", text("control")));
    keep_element(tab_button(CronSettingsTab::Jobs, CronSettingsTab::Jobs, "任务"));
    keep_element(tab_button(CronSettingsTab::Add, CronSettingsTab::Jobs, "添加"));
    keep_element(segment_button("Shell", true, Message::None));
    keep_element(segment_button("Agent", false, Message::None));
    keep_element(small_button("删除", Message::None, true));
    keep_element(small_button("刷新", Message::None, false));
    keep_element(primary_button("保存", Message::None));
    keep_element(secondary_button("取消", Message::None));
    keep_element(text_field("placeholder", "value", |_| Message::None));
    keep_element(pick_field(vec!["a".to_string()], Some("a".to_string()), |_| Message::None));
}

#[test]
fn option_helpers_preserve_selected_values() {
    assert_eq!(
        with_selected_option(vec!["b".to_string(), "a".to_string()], "c"),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
    assert_eq!(
        with_selected_option(vec!["b".to_string(), "b".to_string()], " "),
        vec!["b".to_string()]
    );
    assert_eq!(selected_acp_agent_label(""), NO_ACP_AGENT_LABEL);
    assert_eq!(selected_acp_agent_label("agent-a"), "agent-a");
    assert_eq!(acp_agent_value_from_label(NO_ACP_AGENT_LABEL.to_string()), "");
    assert_eq!(acp_agent_value_from_label("agent-a".to_string()), "agent-a");
}

#[test]
fn app_backed_option_helpers_include_current_selection() {
    let mut app = test_app();
    app.recent_projects = vec!["/tmp/a".to_string()];
    app.project_path = Some("/tmp/current".into());
    app.acp_agents = vec!["acp-a".to_string()];

    assert!(project_options(&app, "/tmp/selected").contains(&"/tmp/selected".to_string()));
    assert!(project_options(&app, "").contains(&"/tmp/current".to_string()));
    assert!(acp_agent_options(&app, "acp-b").contains(&"acp-b".to_string()));
    assert!(acp_agent_options(&app, "").contains(&NO_ACP_AGENT_LABEL.to_string()));
    assert!(models_for_provider(&app, "missing").is_empty());
}

#[test]
fn job_helpers_handle_fallback_labels_and_types() {
    let mut shell = job("shell");
    assert_eq!(job_title(&shell), "shell");
    assert!(!job_is_agent(&shell));

    shell.name = Some("  ".to_string());
    assert_eq!(job_title(&shell), "shell");

    let mut agent = job("agent");
    agent.job_type = "agent".to_string();
    agent.prompt = Some("Summarize".to_string());
    agent.command.clear();
    assert!(job_is_agent(&agent));

    let mut inferred_agent = job("inferred");
    inferred_agent.prompt = Some("Do work".to_string());
    inferred_agent.command.clear();
    assert!(job_is_agent(&inferred_agent));
}

#[test]
fn date_and_preview_helpers_cover_valid_invalid_and_empty_values() {
    assert_ne!(
        format_cron_datetime("2026-06-12T01:02:03Z"),
        "2026-06-12T01:02:03Z"
    );
    assert_eq!(format_cron_datetime("not-a-date"), "not-a-date");
    assert_eq!(format_optional_cron_datetime(None), "无");
    assert_eq!(format_optional_cron_datetime(Some("not-a-date")), "not-a-date");
    assert_eq!(preview_lines("", 3), "");
    assert_eq!(preview_lines("a\nb\nc", 3), "a\nb\nc");
    assert_eq!(preview_lines("a\nb\nc\nd", 3), "a\nb\nc\n...");
}

#[test]
fn job_project_label_prefers_job_project_then_current_project_then_default() {
    let mut app = test_app();
    let mut item = job("job-1");
    assert_eq!(job_project_label(&app, &item), "默认工作区");

    app.project_path = Some("/workspace/current".into());
    assert_eq!(job_project_label(&app, &item), "当前项目: /workspace/current");

    item.project_path = Some("/workspace/job".into());
    assert_eq!(job_project_label(&app, &item), "/workspace/job");
}

#[test]
fn rows_and_forms_build_shell_agent_and_editing_states() {
    let mut app = test_app();
    let mut shell = job("job-1");
    shell.last_output = Some("line1\nline2\nline3\nline4".to_string());
    keep_element(job_row(&app, &shell));

    app.cron_settings.selected_job_ids.push("job-1".to_string());
    app.cron_settings.editing_job_id = Some("job-1".to_string());
    app.cron_settings.edit_draft.job_type = CronAddJobType::Agent;
    app.cron_settings.edit_draft.schedule_kind = CronAddScheduleKind::At;
    app.cron_settings.edit_draft.delivery_enabled = true;
    keep_element(job_row(&app, &shell));

    app.cron_settings.edit_draft.schedule_kind = CronAddScheduleKind::Every;
    keep_element(edit_form(&app));

    app.cron_settings.edit_draft.job_type = CronAddJobType::Shell;
    app.cron_settings.edit_draft.schedule_kind = CronAddScheduleKind::Cron;
    app.cron_settings.edit_draft.delivery_enabled = false;
    keep_element(edit_form(&app));
}

#[test]
fn run_row_handles_missing_output_and_duration() {
    keep_element(run_row(&run("success")));

    let missing = CronRunDto {
        output: None,
        duration_ms: None,
        ..run("failed")
    };
    keep_element(run_row(&missing));
}

#[test]
fn tabs_and_view_cover_loading_empty_jobs_config_and_add_forms() {
    let mut app = test_app();
    app.cron_settings.jobs_loading = true;
    keep_element(jobs_tab(&app));
    keep_element(view(&app));

    app.cron_settings.jobs_loading = false;
    keep_element(jobs_tab(&app));

    app.cron_settings.jobs = vec![job("job-1"), job("job-2")];
    app.cron_settings.selected_job_ids = vec!["job-1".to_string(), "job-2".to_string()];
    keep_element(jobs_tab(&app));

    app.cron_settings.active_tab = CronSettingsTab::Config;
    app.cron_settings.action_status = Some("已保存".to_string());
    app.cron_settings.save_error = Some("保存失败".to_string());
    keep_element(config_tab(&app));
    keep_element(view(&app));

    app.cron_settings.active_tab = CronSettingsTab::Add;
    app.cron_settings.add_draft.job_type = CronAddJobType::Shell;
    app.cron_settings.add_draft.schedule_kind = CronAddScheduleKind::Cron;
    keep_element(add_tab(&app));
    keep_element(view(&app));
}

#[test]
fn add_tab_covers_agent_schedule_and_delivery_variants() {
    let mut app = test_app();
    app.cron_settings.active_tab = CronSettingsTab::Add;
    app.cron_settings.add_draft.job_type = CronAddJobType::Agent;
    app.cron_settings.add_draft.schedule_kind = CronAddScheduleKind::At;
    app.cron_settings.add_draft.model_provider = "openai".to_string();
    app.cron_settings.add_draft.model = "openai/gpt-5".to_string();
    app.cron_settings.add_draft.delivery_enabled = true;
    keep_element(add_tab(&app));

    app.cron_settings.add_draft.schedule_kind = CronAddScheduleKind::Every;
    app.cron_settings.add_draft.model_provider.clear();
    app.cron_settings.add_draft.model = "gpt-5".to_string();
    keep_element(add_tab(&app));
}

#[test]
fn overlays_cover_no_modal_runs_modal_states_and_help() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.cron_settings.runs_modal_job_id = Some("job-1".to_string());
    app.cron_settings.runs_modal_loading = true;
    keep_element(view_overlays(&app, text("dialog").into()));

    app.cron_settings.runs_modal_loading = false;
    app.cron_settings.runs_modal_error = Some("加载失败".to_string());
    keep_element(view_overlays(&app, text("dialog").into()));

    app.cron_settings.runs_modal_error = None;
    keep_element(view_overlays(&app, text("dialog").into()));

    app.cron_settings.jobs = vec![job("job-1")];
    app.cron_settings.runs_modal = vec![run("success")];
    keep_element(view_overlays(&app, text("dialog").into()));

    app.cron_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}
