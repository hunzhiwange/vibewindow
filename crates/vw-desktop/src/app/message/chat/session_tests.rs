#![allow(unused_must_use)]
#[test]
fn session_tests_module_is_wired() {
    assert!(module_path!().ends_with("session_tests"));
}

fn chat_message(
    role: crate::app::models::ChatRole,
    content: &str,
) -> crate::app::models::ChatMessage {
    crate::app::models::ChatMessage { role, content: content.to_string(), think_timing: Vec::new() }
}

fn session_info(id: &str, title: &str, directory: &str) -> vw_shared::session::info::Info {
    vw_shared::session::info::Info {
        id: id.to_string(),
        slug: format!("{id}-slug"),
        project_id: "project-1".to_string(),
        directory: directory.to_string(),
        parent_id: None,
        summary: None,
        share: None,
        title: title.to_string(),
        version: "0.0.0".to_string(),
        time: vw_shared::session::info::TimeInfo {
            created: 10,
            updated: 20,
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: None,
    }
}

#[test]
fn message_id_for_index_uses_previous_known_id_when_current_is_missing() {
    let ids = vec![Some("msg-1".to_string()), None];

    assert_eq!(super::message_id_for_index_in_ids(&ids, 1), Some("msg-1".to_string()));
}

#[test]
fn message_id_for_index_handles_gateway_rows_shorter_than_local_chat() {
    let ids = vec![Some("msg-1".to_string())];

    assert_eq!(super::message_id_for_index_in_ids(&ids, 3), Some("msg-1".to_string()));
}

#[test]
fn message_id_for_index_returns_current_id_before_previous_fallback() {
    let ids = vec![Some("msg-1".to_string()), Some("msg-2".to_string())];

    assert_eq!(super::message_id_for_index_in_ids(&ids, 1), Some("msg-2".to_string()));
    assert_eq!(super::message_id_for_index_in_ids(&ids, 0), Some("msg-1".to_string()));
    assert_eq!(super::message_id_for_index_in_ids(&[], 0), None);
}

#[test]
fn parse_inline_mode_command_accepts_known_commands_case_insensitively() {
    assert_eq!(
        super::parse_inline_mode_command("/task build"),
        Some(super::InlineModeCommand::TaskMode)
    );
    assert_eq!(
        super::parse_inline_mode_command(" /NEW topic"),
        Some(super::InlineModeCommand::ChatMode)
    );
    assert_eq!(
        super::parse_inline_mode_command("/clear"),
        Some(super::InlineModeCommand::ChatMode)
    );
    assert_eq!(
        super::parse_inline_mode_command("/session x"),
        Some(super::InlineModeCommand::ChatMode)
    );
    assert_eq!(super::parse_inline_mode_command("/unknown"), None);
    assert_eq!(super::parse_inline_mode_command(""), None);
}

#[test]
fn apply_inline_mode_command_toggles_task_mode_and_clears_runtime_editor() {
    let (mut app, _task) = crate::app::App::new();
    app.current_session_runtime_mut().input_editor =
        iced::widget::text_editor::Content::with_text("draft");

    super::apply_inline_mode_command(&mut app, super::InlineModeCommand::TaskMode);

    assert!(app.current_session_runtime().task_mode_enabled);
    assert_eq!(app.current_session_runtime().input_editor.text(), "");

    super::apply_inline_mode_command(&mut app, super::InlineModeCommand::ChatMode);

    assert!(!app.current_session_runtime().task_mode_enabled);
}

#[test]
fn normalize_delegate_agent_trims_and_drops_main_agent() {
    assert_eq!(
        super::normalize_delegate_agent(Some(" worker ".to_string())),
        Some("worker".to_string())
    );
    assert_eq!(
        super::normalize_delegate_agent(Some(crate::app::state::MAIN_AGENT_KEY.to_string())),
        None
    );
    assert_eq!(super::normalize_delegate_agent(Some("   ".to_string())), None);
    assert_eq!(super::normalize_delegate_agent(None), None);
}

#[test]
fn request_delegate_agent_defaults_to_main_agent() {
    assert_eq!(
        super::request_delegate_agent(Some("worker".to_string())),
        Some("worker".to_string())
    );
    assert_eq!(
        super::request_delegate_agent(None),
        Some(crate::app::state::MAIN_AGENT_KEY.to_string())
    );
}

#[test]
fn compose_query_with_session_context_appends_selected_tools_and_skills() {
    assert_eq!(super::compose_query_with_session_context("  hi  ", &[], &[]), "hi");

    let composed = super::compose_query_with_session_context(
        "run this",
        &["shell".to_string(), "git".to_string()],
        &["rust".to_string()],
    );

    assert!(composed.starts_with("run this\n\n<session_control_selection>"));
    assert!(composed.contains("工具：shell, git"));
    assert!(composed.contains("技能：rust"));
    assert!(composed.ends_with("</session_control_selection>"));
}

#[test]
fn compose_query_with_session_context_can_stand_alone_without_query() {
    let composed = super::compose_query_with_session_context("   ", &["shell".to_string()], &[]);

    assert!(composed.starts_with("<session_control_selection>"));
    assert!(composed.contains("工具：shell"));
}

#[test]
fn is_supported_image_attachment_matches_known_extensions_case_insensitively() {
    assert!(super::is_supported_image_attachment("/tmp/a.PNG"));
    assert!(super::is_supported_image_attachment("/tmp/a.jpeg"));
    assert!(super::is_supported_image_attachment("/tmp/a.webp"));
    assert!(!super::is_supported_image_attachment("/tmp/a.pdf"));
    assert!(!super::is_supported_image_attachment("/tmp/no-extension"));
}

#[test]
fn compose_query_with_attachments_adds_document_and_image_markers() {
    assert_eq!(super::compose_query_with_attachments("  hello  ", &[]), "hello");
    assert_eq!(
        super::compose_query_with_attachments(
            "",
            &["/tmp/a.png".to_string(), "/tmp/b.pdf".to_string()]
        ),
        "[IMAGE:/tmp/a.png]\n[DOCUMENT:/tmp/b.pdf]"
    );
    assert_eq!(
        super::compose_query_with_attachments("look", &["/tmp/a.gif".to_string()]),
        "look\n\n[IMAGE:/tmp/a.gif]"
    );
}

#[test]
fn gateway_endpoint_defaults_empty_host_to_loopback() {
    let (mut app, _task) = crate::app::App::new();
    app.gateway_settings.host_input = "   ".to_string();
    app.gateway_settings.port = 4321;

    assert_eq!(super::gateway_endpoint(&app), ("127.0.0.1".to_string(), 4321));

    app.gateway_settings.host_input = "0.0.0.0".to_string();
    assert_eq!(super::gateway_endpoint(&app), ("0.0.0.0".to_string(), 4321));
}

#[test]
fn current_session_directory_uses_active_session_directory() {
    let (mut app, _task) = crate::app::App::new();
    app.active_session_id = Some("s1".to_string());
    app.sessions = vec![session_info("s1", "Session", "/repo")];

    assert_eq!(super::current_session_directory(&app).as_deref(), Some("/repo"));

    app.active_session_id = Some("missing".to_string());
    assert_eq!(super::current_session_directory(&app), None);
}

#[test]
fn restore_cached_chat_for_session_uses_matching_cached_message_ids() {
    let (mut app, _task) = crate::app::App::new();
    app.store_session_chat_snapshot(
        "s1".to_string(),
        crate::app::session::shared_chat_messages(vec![
            chat_message(crate::app::models::ChatRole::User, "hi"),
            chat_message(crate::app::models::ChatRole::Assistant, "hello"),
        ]),
        vec![Some("m1".to_string()), Some("m2".to_string())],
    );

    super::restore_cached_chat_for_session(&mut app, "s1");

    assert_eq!(app.chat.len(), 2);
    assert_eq!(app.chat_message_ids, vec![Some("m1".to_string()), Some("m2".to_string())]);
}

#[test]
fn restore_cached_chat_for_session_resets_ids_when_cached_ids_have_wrong_len() {
    let (mut app, _task) = crate::app::App::new();
    app.store_session_chat_snapshot(
        "s1".to_string(),
        crate::app::session::shared_chat_messages(vec![
            chat_message(crate::app::models::ChatRole::User, "hi"),
            chat_message(crate::app::models::ChatRole::Assistant, "hello"),
        ]),
        vec![Some("m1".to_string())],
    );

    super::restore_cached_chat_for_session(&mut app, "s1");

    assert_eq!(app.chat.len(), 2);
    assert_eq!(app.chat_message_ids, vec![None, None]);

    super::restore_cached_chat_for_session(&mut app, "missing");
    assert!(app.chat.is_empty());
    assert!(app.chat_message_ids.is_empty());
}
