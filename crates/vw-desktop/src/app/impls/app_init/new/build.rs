//! 组织桌面应用初始化阶段的 build.rs 逻辑。
//! 本模块把启动输入、配置加载和初始状态装配拆开，便于定位启动失败路径。

use std::collections::{HashMap, HashSet};

use iced::Theme;

use super::settings;
use super::state::NewAppInit;
use super::*;

/// 模块内可见函数，执行 build_app 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn build_app(init: NewAppInit) -> App {
    let NewAppInit {
        cfg,
        system_settings_cfg,
        redis_tool_persisted,
        gateway_client_cfg,
        full_agent_cfg,
        global_acp_cfg,
        gateway_cfg_result,
        init_secs,
        init_ms,
        init_utc,
    } = init;

    let cfg_show_terminal =
        cfg.get("show_terminal").and_then(|value: &serde_json::Value| value.as_bool()).unwrap_or(false);
    let cfg_model = cfg
        .get("model")
        .and_then(|value: &serde_json::Value| value.as_str())
        .unwrap_or("auto")
        .to_string();
    let cfg_auto_model =
        cfg.get("auto_model").and_then(|value: &serde_json::Value| value.as_bool()).unwrap_or(true);
    let cfg_acp_agent = cfg
        .get("acp_agent")
        .and_then(|value: &serde_json::Value| value.as_str())
        .map(ToString::to_string);
    let cfg_acp_history_mode = crate::app::state::AcpHistoryReplayMode::from_str(
        cfg.get("acp_history_strategy")
            .and_then(|value: &serde_json::Value| value.as_str())
            .unwrap_or("discard"),
    );
    let cfg_acp_recent_count = cfg
        .get("acp_history_recent_count")
        .and_then(|value: &serde_json::Value| value.as_u64())
        .map(|value| value.clamp(1, 20) as usize)
        .unwrap_or(3);
    let cfg_auto_max_mode =
        cfg.get("auto_max_mode").and_then(|value: &serde_json::Value| value.as_bool()).unwrap_or(false);
    let cfg_file_tree_expanded = cfg
        .get("file_tree_expanded")
        .and_then(|value: &serde_json::Value| value.as_array())
        .map(|entries: &Vec<serde_json::Value>| {
            entries
                .iter()
                .filter_map(|entry: &serde_json::Value| entry.as_str().map(ToString::to_string))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let file_tree_menu_path = None;
    let file_tree_menu_anchor = None;
    let file_tree_menu_source = None;

    let cfg_shell = match system_settings_cfg.terminal_shell.as_str() {
        "zsh" => Shell::Zsh,
        _ => Shell::Bash,
    };
    let cfg_term_theme = TerminalTheme::System;
    let cfg_term_font_family = system_settings_cfg.terminal_font_family.clone();
    let cfg_term_font_size = system_settings_cfg.terminal_font_size;
    let cfg_app_theme = Theme::ALL
        .iter()
        .find(|theme| theme.to_string() == system_settings_cfg.app_theme)
        .cloned()
        .unwrap_or(Theme::Light);
    let cfg_editor_follow_system_theme = system_settings_cfg.editor_follow_system_theme;
    let cfg_editor_theme = Theme::ALL
        .iter()
        .find(|theme| theme.to_string() == system_settings_cfg.editor_theme)
        .cloned()
        .unwrap_or_else(|| cfg_app_theme.clone());

    let cfg_show_layer_panel =
        cfg.get("show_layer_panel").and_then(|value: &serde_json::Value| value.as_bool()).unwrap_or(true);
    let cfg_layer_panel_width = cfg
        .get("layer_panel_width")
        .and_then(|value: &serde_json::Value| value.as_f64())
        .map(|value| value as f32)
        .unwrap_or(250.0);
    let cfg_show_design_planner_panel = cfg
        .get("show_design_planner_panel")
        .and_then(|value: &serde_json::Value| value.as_bool())
        .unwrap_or(false);
    let cfg_design_planner_panel_width = cfg
        .get("design_planner_panel_width")
        .and_then(|value: &serde_json::Value| value.as_f64())
        .map(|value| value as f32)
        .unwrap_or(380.0)
        .clamp(260.0, 640.0);
    let cfg_design_planner_corner = match cfg
        .get("design_planner_corner")
        .and_then(|value: &serde_json::Value| value.as_str())
        .unwrap_or("bottom_left")
    {
        "top_left" => crate::app::views::design::state::DesignPlannerCorner::TopLeft,
        "top_right" => crate::app::views::design::state::DesignPlannerCorner::TopRight,
        "bottom_left" => crate::app::views::design::state::DesignPlannerCorner::BottomLeft,
        _ => crate::app::views::design::state::DesignPlannerCorner::BottomLeft,
    };
    let cfg_show_properties_panel = cfg
        .get("show_properties_panel")
        .and_then(|value: &serde_json::Value| value.as_bool())
        .unwrap_or(true);
    let cfg_properties_panel_width = cfg
        .get("properties_panel_width")
        .and_then(|value: &serde_json::Value| value.as_f64())
        .map(|value| value as f32)
        .unwrap_or(300.0);
    let cfg_file_manager_width = cfg
        .get("file_manager_width")
        .and_then(|value: &serde_json::Value| value.as_f64())
        .map(|value| value as f32)
        .unwrap_or(260.0)
        .clamp(180.0, 600.0);
    let cfg_file_manager_show_changes = cfg
        .get("file_manager_show_changes")
        .and_then(|value: &serde_json::Value| value.as_bool())
        .unwrap_or(true);
    let cfg_show_file_manager =
        cfg.get("show_file_manager").and_then(|value: &serde_json::Value| value.as_bool()).unwrap_or(true);
    let cfg_show_git_diff_summary = cfg
        .get("show_git_diff_summary")
        .and_then(|value: &serde_json::Value| value.as_bool())
        .unwrap_or(false);
    let cfg_show_git_diff_highlight = cfg
        .get("show_git_diff_highlight")
        .and_then(|value: &serde_json::Value| value.as_bool())
        .unwrap_or(false);
    let cfg_show_settings =
        cfg.get("show_settings").and_then(|value: &serde_json::Value| value.as_bool()).unwrap_or(true);
    let cfg_project_worktree_enabled = system_settings_cfg.project_worktree_enabled.clone();

    let acp_agents = sort_acp_agents(&global_acp_cfg);
    let cfg_acp_agent =
        cfg_acp_agent.filter(|agent| acp_agents.iter().any(|candidate| candidate == agent));
    let cfg_new_session_last_directory = cfg
        .get("new_session_last_directory")
        .and_then(|value: &serde_json::Value| value.as_str())
        .map(ToString::to_string);

    let (open_external_platform, open_external_exists, open_external_app) =
        external_apps::resolve_external_apps(&cfg);

    let sessions = Vec::new();
    #[cfg(not(target_arch = "wasm32"))]
    let cfg_archived_session_ids = crate::app::session_gateway::gateway_load_archived_session_ids(None);
    #[cfg(target_arch = "wasm32")]
    let cfg_archived_session_ids = HashSet::new();
    let active_session_id = None;
    let chat = Vec::new();
    let chat_message_editors = Vec::new();
    let usage = models::TokenUsage::default();
    let effective_app_theme = cfg_app_theme;
    let recent_meta = load_recent_projects_meta();
    let cfg_web_bookmarks = load_web_bookmarks(&cfg);

    #[cfg(not(target_arch = "wasm32"))]
    let (lsp_event_sender, lsp_events) = {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        (Some(event_tx), Some(event_rx))
    };
    #[cfg(not(target_arch = "wasm32"))]
    let lsp_manager = lsp_event_sender.as_ref().cloned().map(LspServiceManager::new);

    let channels_settings = settings_builders::build_channels_settings(&full_agent_cfg.channels_config);
    let chat_message_estimated_heights = crate::app::components::chat_panel::rough_message_heights(&chat);
    let chat_height_index = crate::app::components::chat_panel::height_index::ChatHeightIndex::from_heights(
        &chat_message_estimated_heights,
    );

    App {
        screen: Screen::Home,
        project_path: None,
        project_id: None,
        project_path_input: String::new(),
        input_text: String::new(),
        model: cfg_model.clone(),
        auto_model: cfg_auto_model,
        acp_agent: cfg_acp_agent.clone(),
        acp_history_mode: cfg_acp_history_mode,
        acp_recent_count: cfg_acp_recent_count,
        acp_agents,
        file_url_input: String::new(),
        files: vec![],
        chat,
        chat_message_ids: Vec::new(),
        chat_render_cache: HashMap::new(),
        chat_visible_text_cache: Vec::new(),
        chat_copy_hash_cache: Vec::new(),
        chat_message_expanded: std::collections::HashSet::new(),
        chat_message_estimated_heights,
        chat_height_index,
        chat_message_measured_heights: HashMap::new(),
        chat_message_editors,
        chat_special_text_editors: HashMap::new(),
        chat_tool_text_editors: HashMap::new(),
        chat_think_editors: HashMap::new(),
        chat_think_expanded: std::collections::HashSet::new(),
        chat_think_collapsed: std::collections::HashSet::new(),
        chat_think_hovered_idx: None,
        chat_tool_file_expanded: std::collections::HashSet::new(),
        chat_tool_file_hovered: None,
        chat_tool_expanded: std::collections::HashSet::new(),
        chat_tool_hovered_idx: None,
        chat_explore_expanded: std::collections::HashSet::new(),
        chat_explore_summary_animations: HashMap::new(),
        tool_detail_dialog: None,
        chat_think_scroll_ids: HashMap::new(),
        chat_stream_think_msg_idx: None,
        chat_stream_think_count: 0,
        chat_stream_think_open_idx: None,
        chat_context_menu_target: None,
        chat_context_menu_pos: None,
        chat_context_menu_text: String::new(),
        input_context_menu_open: false,
        input_context_menu_pos: None,
        chat_reset_menu_idx: None,
        chat_todo_expanded: true,
        chat_todo_anim: 1.0,
        chat_todo_placement: TodoPanelPlacement::ChatTopRight,
        chat_todo_session_id: None,
        chat_todo_items: vec![],
        usage,
        active_session_view_state: crate::app::state::ActiveSessionViewState::default(),
        usage_model_info: None,
        usage_session_file_path: None,
        usage_step_expanded: std::collections::HashSet::new(),
        merge_view: true,
        expanded_files: vec![],
        expanded_files_set: HashSet::new(),
        context_expansions: HashMap::new(),
        branches: vec![],
        selected_branch: None,
        project_updated_at_ms: None,
        recent_projects: load_recent_projects(),
        is_requesting: false,
        submit_anim: 0,
        active_agent_request: None,
        agent_stream_id: 0,
        queue: vec![],
        split_ratio: 0.6,
        dragging_split: false,
        split_drag_anchor_x: None,
        split_drag_start_ratio: 0.0,
        window_size: (1200.0, 800.0),
        fullscreen_layout_settling: false,
        startup_resize_checked: false,
        show_settings: cfg_show_settings,
        settings_sidebar_collapsed: false,
        show_system_settings: false,
        show_about_modal: false,
        show_cli_install_modal: false,
        cli_install_modal_title: String::new(),
        cli_install_modal_message: String::new(),
        cli_install_modal_current_version: String::new(),
        cli_install_modal_server_version: String::new(),
        cli_install_modal_show_update_action: false,
        cli_install_modal_show_install_action: false,
        cli_install_modal_use_app_update_action: false,
        cli_install_modal_is_checking_update: false,
        question_modal_request_id: None,
        question_modal_request: None,
        question_modal_answers: vec![],
        question_modal_custom: vec![],
        permission_modal_request_id: None,
        permission_modal_request: None,
        permission_modal_requests: vec![],
        active_menu: None,
        open_external_app,
        open_external_platform,
        open_external_exists,
        settings_panel_width: 476.0,
        dragging_settings: false,
        settings_drag_anchor_x: None,
        settings_drag_start_width: 0.0,
        file_manager_width: cfg_file_manager_width,
        dragging_file_manager: false,
        file_manager_drag_anchor_x: None,
        file_manager_start_width: cfg_file_manager_width,
        file_manager_show_changes: cfg_file_manager_show_changes,
        file_manager_refresh_frame: 0,
        status_animation_frame: 0,
        file_manager_changes_refreshing: false,
        file_manager_file_tree_refreshing: false,
        show_file_manager: cfg_show_file_manager,
        show_model_popover: false,
        show_mode_popover: false,
        show_file_popover: false,
        show_acp_popover: false,
        show_usage_popover: false,
        show_session_tool_selector_popover: false,
        show_session_actions_popover: false,
        show_executor_popover: false,
        session_title_last_click: None,
        model_popover_hover: None,
        auto_max_mode: cfg_auto_max_mode,
        last_call_log_path: None,
        last_session_snapshot_path: None,
        search_text: String::new(),
        show_search_overlay: false,
        file_index_cache: HashMap::new(),
        file_index_revision: 0,
        file_tree_model_cache: HashMap::new(),
        search_panel_file_cache_query: String::new(),
        search_panel_file_cache_project_path: None,
        search_panel_file_cache_revision: 0,
        search_panel_file_cache_results: Vec::new(),
        file_search_cache_query: String::new(),
        file_search_cache_project_path: None,
        file_search_cache_revision: 0,
        file_search_cache_entries: Vec::new(),
        input_editor: iced::widget::text_editor::Content::new(),
        last_copied_code_hash: None,
        last_copy_time: None,
        chat_auto_scroll: true,
        chat_scroll_id: iced::widget::Id::new("chat"),
        chat_scroll_offset_y: 1.0,
        chat_scroll_viewport_h: 0.0,
        task_pet_scroll_id: iced::widget::Id::new("task_pet"),
        task_pet_position: iced::Point::new(620.0, 96.0),
        task_pet_collapsed: true,
        task_pet_expand_progress: 0.0,
        task_pet_expand_target: None,
        task_pet_dragging: false,
        task_pet_drag_anchor: None,
        task_pet_drag_start: iced::Point::ORIGIN,
        task_pet_drag_direction: 1,
        task_pet_walk_until_ms: 0,
        task_pet_avatar_kind: state::TaskPetAvatarKind::Robot,
        task_pet_robot_hovered: false,
        task_pet_items: Vec::new(),
        task_pet_hovered_request_id: None,
        task_pet_reply_request_id: None,
        task_pet_reply_input: String::new(),
        task_pet_dismissed_request_ids: HashSet::new(),
        chat_stream_autoscroll_last_ms: 0,
        chat_autoscroll_hold_until_ms: 0,
        chat_panel_fullscreen: false,
        chat_panel_half_fullscreen: false,
        show_chat_fullscreen_overlay: false,
        input_editor_id: iced::widget::Id::new("input_editor"),
        json_tool_editor_id: iced::widget::Id::new("json_tool_editor"),
        sql_tool_editor_id: iced::widget::Id::new("sql_tool_editor"),
        html_tool_editor_id: iced::widget::Id::new("html_tool_editor"),
        json_diff_left_editor_id: iced::widget::Id::new("json_diff_left_editor"),
        json_diff_right_editor_id: iced::widget::Id::new("json_diff_right_editor"),
        json_yaml_left_editor_id: iced::widget::Id::new("json_yaml_left_editor"),
        json_yaml_right_editor_id: iced::widget::Id::new("json_yaml_right_editor"),
        markdown_tool_editor_id: iced::widget::Id::new("markdown_tool_editor"),
        pwd_editor_id: iced::widget::Id::new("pwd_editor"),
        file_search_scroll_id: iced::widget::Id::new("file_search"),
        preview_scroll_id: iced::widget::Id::new("preview"),
        preview_tabs_scroll_id: iced::widget::Id::new("preview_tabs"),
        git_diff_scroll_id: iced::widget::Id::new("git_diff"),
        git_diff_fullscreen: false,
        git_diff_half_fullscreen: false,
        home_apps_bar_scroll_x: 0.0,
        home_apps_bar_scroll_id: iced::widget::Id::new("home_apps_bar"),
        cursor_position: iced::Point::ORIGIN,
        window_position: (0.0, 0.0),
        main_window_id: None,
        task_pet_window_id: None,
        recent_projects_edits: {
            let recent = load_recent_projects();
            recent.iter().map(|path| display_name_for_path(&recent_meta, path)).collect()
        },
        recent_project_delete_confirm_idx: None,
        dialogue_flow_permission_editor: iced::widget::text_editor::Content::with_text("{}"),
        dialogue_flow_show_reasoning_summary: system_settings_cfg.dialogue_flow_show_reasoning_summary,
        dialogue_flow_expand_shell_tool_section: system_settings_cfg.dialogue_flow_expand_shell_tool_section,
        dialogue_flow_expand_edit_tool_section: system_settings_cfg.dialogue_flow_expand_edit_tool_section,
        chat_send_behavior: cfg_chat_send_behavior,
        dialogue_flow_settings_save_message: None,
        recent_projects_meta: recent_meta,
        spinner_frame: 0,
        file_to_discard: None,
        sessions,
        active_session_id,
        archived_session_ids: cfg_archived_session_ids,
        session_chat_cache: HashMap::new(),
        session_chat_message_id_cache: HashMap::new(),
        session_previews: HashMap::new(),
        session_runtime_states: std::collections::HashMap::from([(
            "__empty__".to_string(),
            crate::app::state::SessionRuntimeState::with_defaults(cfg_model.clone(), cfg_auto_model),
        )]),
        project_sessions: std::collections::HashMap::new(),
        project_session_load_counts: std::collections::HashMap::new(),
        project_sessions_loading: std::collections::HashSet::new(),
        project_session_has_vertical_scrollbar: std::collections::HashMap::new(),
        project_sessions_last_refresh_at: std::collections::HashMap::new(),
        session_menu_id: None,
        session_menu_anchor: None,
        project_tools_menu_path: None,
        new_session_picker_project: None,
        new_session_picker_options: Vec::new(),
        project_worktree_enabled: cfg_project_worktree_enabled,
        new_session_last_directory: cfg_new_session_last_directory,
        new_session_worktree_name: String::new(),
        new_session_confirm_delete_directory: None,
        new_session_force_delete_directory: None,
        new_session_delete_error: None,
        new_session_confirm_reset_directory: None,
        new_session_reset_error: None,
        session_rename_id: None,
        session_rename_value: String::new(),
        project_edit_path: None,
        project_edit_tab: crate::app::state::ProjectEditTab::General,
        project_edit_name: String::new(),
        project_edit_icon: String::new(),
        project_edit_icon_hovered: false,
        project_edit_icon_color: String::new(),
        project_edit_icon_color_picker_open: false,
        project_edit_icon_color_format: crate::app::views::design::models::ColorFormat::Hex,
        project_edit_start_script: String::new(),
        project_edit_start_script_editor: iced::widget::text_editor::Content::new(),
        project_edit_worktree_enabled: false,
        project_edit_task_board_settings: crate::app::task::TaskBoardSettings::new(),
        project_edit_max_concurrent_input: String::new(),
        project_edit_task_board_auto_refresh: true,
        project_edit_session_auto_refresh: crate::app::state::default_recent_project_session_auto_refresh(),
        project_edit_session_refresh_interval_seconds_input: String::new(),
        project_edit_task_board_refresh_interval_seconds_input: String::new(),
        project_edit_task_board_scheduler_tick_interval_seconds_input: String::new(),
        project_edit_task_board_auto_promote_tick_interval_seconds_input: String::new(),
        project_edit_failed_retry_minutes_input: String::new(),
        project_edit_running_timeout_minutes_input: String::new(),
        project_edit_pr_submitted_stall_timeout_seconds_input: String::new(),
        show_diff: true,
        show_git_diff_summary: cfg_show_git_diff_summary,
        show_git_diff_highlight: cfg_show_git_diff_highlight,
        terminal: TerminalState::new(
            cfg_show_terminal,
            cfg_shell,
            cfg_term_theme,
            cfg_term_font_family,
            cfg_term_font_size,
            None,
        ),
        terminals_by_project: HashMap::new(),
        diff_theme: DiffTheme::GitHub,
        app_theme: effective_app_theme,
        editor_follow_system_theme: cfg_editor_follow_system_theme,
        editor_theme: cfg_editor_theme,
        expanded_hunks: vec![],
        git_commit_message: String::new(),
        git_commit_type: Some(ConventionalCommitType::Feat),
        git_commit_scope: String::new(),
        git_commit_description: String::new(),
        git_commit_in_progress: false,
        show_git_commit_help_modal: false,
        show_git_filter_help_modal: false,
        staged_files_selected: vec![],
        staged_hunks_selected: vec![],
        staged_lines_selected: vec![],
        staged_old_lines_selected: vec![],
        git_diff_selected_lines: vec![],
        git_diff_drag_range: None,
        git_diff_selected_range: None,
        git_diff_dragging: false,
        git_diff_drag_start_text: None,
        git_diff_last_click: None,
        git_diff_hovered_line: None,
        git_diff_comment_draft: None,
        git_diff_context_menu: None,
        git_diff_file_menu: None,
        show_git_copy_modal: false,
        git_copy_modal_editor: iced::widget::text_editor::Content::new(),
        git_copy_modal_use_color: true,
        git_copy_modal_code_editor: iced_code_editor::CodeEditor::new("", "diff"),
        show_git_custom_diff_modal: false,
        git_custom_diff_hide_inputs: false,
        git_custom_diff_title: String::new(),
        git_custom_diff_before_editor: iced::widget::text_editor::Content::new(),
        git_custom_diff_after_editor: iced::widget::text_editor::Content::new(),
        chat_text_diff: None,
        git_commit_description_editor: iced::widget::text_editor::Content::new(),
        show_git_filter_options: false,
        git_filter_query: String::new(),
        git_filter_included: false,
        git_filter_excluded: false,
        git_filter_new: false,
        git_filter_modified: false,
        git_filter_deleted: false,
        git_focused_file: None,
        git_hovered_file_header: None,
        git_panel_header_hovered: false,
        git_changed_files: vec![],
        git_changed_files_loading: false,
        git_diff_file_metas: vec![],
        git_diff_file_metas_loading: false,
        git_diff_file_metas_repo_path: None,
        git_diff_contents: HashMap::new(),
        git_diff_contents_loading: HashSet::new(),
        git_diff_scroll_offset_y: 0.0,
        git_diff_scroll_viewport_h: 0.0,
        settings_tab: SettingsTab::Sessions,
        system_settings_tab: components::system_settings::SystemTab::General,
        system_settings_query: String::new(),
        system_settings_help_tab: None,
        provider_settings: crate::app::state::ProviderSettingsState::default(),
        model_settings: crate::app::state::ModelSettingsState::default(),
        embedding_routes_settings: settings::build_embedding_routes_settings(&full_agent_cfg, &gateway_cfg_result),
        model_routes_settings: settings::build_model_routes_settings(&full_agent_cfg),
        query_classification_settings: settings::build_query_classification_settings(&full_agent_cfg),
        goal_loop_settings: settings::build_goal_loop_settings(&full_agent_cfg),
        heartbeat_settings: settings::build_heartbeat_settings(&full_agent_cfg),
        cron_settings: settings::build_cron_settings(&full_agent_cfg),
        sop_settings: settings::build_sop_settings(&full_agent_cfg),
        scheduler_settings: settings::build_scheduler_settings(&full_agent_cfg),
        hooks_settings: settings::build_hooks_settings(&full_agent_cfg),
        runtime_settings: settings::build_runtime_settings(&full_agent_cfg),
        skills_settings: settings::build_skills_settings(&full_agent_cfg),
        research_settings: settings::build_research_settings(&full_agent_cfg),
        web_search_settings: settings::build_web_search_settings(&full_agent_cfg),
        browser_settings: settings::build_browser_settings(&full_agent_cfg),
        http_request_settings: settings::build_http_request_settings(&full_agent_cfg),
        gateway_settings: settings::build_gateway_settings(&gateway_cfg_result),
        gateway_client_settings: settings::build_gateway_client_settings(&gateway_client_cfg),
        agents_ipc_settings: settings::build_agents_ipc_settings(&full_agent_cfg),
        agents_settings: settings::build_agents_settings(&cfg, &full_agent_cfg),
        coordination_settings: settings::build_coordination_settings(&full_agent_cfg),
        cost_settings: crate::app::state::CostSettingsState::default(),
        memory_settings: settings::build_memory_settings(&full_agent_cfg),
        channels_settings,
        reliability_settings: settings::build_reliability_settings(&full_agent_cfg),
        multimodal_settings: settings::build_multimodal_settings(&full_agent_cfg),
        security_settings: settings::build_security_settings(&full_agent_cfg),
        autonomy_settings: settings::build_autonomy_settings(&full_agent_cfg),
        observability_settings: settings::build_observability_settings(&full_agent_cfg),
        storage_settings: settings::build_storage_settings(&full_agent_cfg),
        proxy_settings: settings::build_proxy_settings(&full_agent_cfg),
        tunnel_settings: settings::build_tunnel_settings(&full_agent_cfg),
        composio_settings: settings::build_composio_settings(&full_agent_cfg),
        transcription_settings: settings::build_transcription_settings(&full_agent_cfg),
        file_tree_expanded: cfg_file_tree_expanded.clone(),
        file_tree_expanded_set: cfg_file_tree_expanded.into_iter().collect(),
        file_tree_menu_path,
        file_tree_menu_anchor,
        file_tree_menu_source,
        file_tree_clipboard: None,
        dragging_file_paths: Vec::new(),
        dragging_file_position: None,
        pending_drop_file_paths: Vec::new(),
        pending_drop_file_position: None,
        input_drop_hovered: false,
        file_tree_rename_path: None,
        file_tree_rename_value: String::new(),
        file_search_query: String::new(),
        show_file_search: false,
        file_search_anchor: None,
        file_search_selected_index: 0,
        find_results_tabs: Vec::new(),
        active_find_results_tab_id: None,
        tool_files_filter: String::new(),
        file_ref_hovered_index: None,
        preview_tabs: Vec::new(),
        active_preview_path: None,
        project_preview_tabs: HashMap::new(),
        project_preview_active_path: HashMap::new(),
        preview_tab_menu_path: None,
        preview_tab_menu_pos: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_events,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_event_sender,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_manager,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_overlay: iced_code_editor::LspOverlayState::new(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_disabled: false,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_applying_completion: false,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_hover_anchor: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_overlay_path: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_hover_pending: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_hover_hide_deadline: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_progress: HashMap::new(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_status: None,
        pending_preview_goto: None,
        preview_trace_back: Vec::new(),
        preview_trace_forward: Vec::new(),
        preview_trace_navigating: false,
        previous_preview_path: None,
        focus_area: FocusArea::None,
        show_preview_context_menu: false,
        preview_context_target: None,
        preview_context_menu_pos: None,
        preview_nav_popup: None,
        design_states: HashMap::new(),
        mouse_wheel_zoom_enabled: false,
        show_slot_content: cfg
            .get("show_slot_content")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(false),
        show_slot_overflow: cfg
            .get("show_slot_overflow")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(false),
        show_design_settings: false,
        design_settings_active_tab: DesignSettingsTab::General,
        show_design_shortcuts: false,
        show_design_variables: false,
        show_zoom_menu: false,
        show_layer_panel: cfg_show_layer_panel,
        layer_panel_width: cfg_layer_panel_width,
        dragging_layer_panel: false,
        layer_panel_drag_anchor_x: None,
        layer_panel_start_width: cfg_layer_panel_width,
        show_design_planner_panel: cfg_show_design_planner_panel,
        design_planner_panel_width: cfg_design_planner_panel_width,
        design_planner_corner: cfg_design_planner_corner,
        dragging_design_planner_panel: false,
        design_planner_panel_drag_anchor_x: None,
        design_planner_panel_start_width: cfg_design_planner_panel_width,
        show_properties_panel: cfg_show_properties_panel,
        properties_panel_width: cfg_properties_panel_width,
        dragging_properties_panel: false,
        properties_panel_drag_anchor_x: None,
        properties_panel_start_width: cfg_properties_panel_width,
        active_layer_menu: None,
        layer_menu_anchor: None,
        dragging_layer: None,
        drag_target_layer: None,
        hovered_layer_id: None,
        error_message: None,
        active_color_picker: None,
        active_fill_picker: None,
        active_effect_picker: None,
        active_font_picker: None,
        active_icon_picker: None,
        active_tailwind_class_picker: None,
        show_element_html_preview: false,
        element_html_preview_editor: iced::widget::text_editor::Content::new(),
        design_help_text: None,
        tailwind_filter_query: String::new(),
        font_filter_query: String::new(),
        icon_picker_filter_query: String::new(),
        icon_picker_family_tab: "lucide".to_string(),
        show_preview_settings: false,
        show_preview_fullscreen_overlay: false,
        current_font_size: system_settings_cfg.editor_font_size.max(1.0),
        current_line_height: system_settings_cfg.editor_line_height.max(1.0),
        auto_adjust_line_height: system_settings_cfg.editor_auto_line_height,
        preview_auto_save_mode: system_settings_cfg.preview_auto_save,
        current_language: iced_code_editor::i18n::Language::default(),
        json_tool_editor: {
            let remember = cfg
                .get("json_tool_remember")
                .and_then(|value: &serde_json::Value| value.as_bool())
                .unwrap_or(false);
            if remember {
                iced::widget::text_editor::Content::with_text(&config::load_json_tool_content())
            } else {
                iced::widget::text_editor::Content::new()
            }
        },
        json_tool_remember: cfg
            .get("json_tool_remember")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(false),
        json_tool_loading: false,
        json_tool_notification: None,
        json_tool_context_menu_open: false,
        json_tool_context_menu_pos: None,
        json_tool_scroll_top_line: 0.0,
        json_tool_scroll_remainder: 0.0,
        json_tool_viewport_height: 0.0,
        json_yaml_left_editor: iced::widget::text_editor::Content::new(),
        json_yaml_right_editor: iced::widget::text_editor::Content::new(),
        json_yaml_loading: false,
        json_yaml_notification: None,
        json_yaml_left_context_menu_open: false,
        json_yaml_left_context_menu_pos: None,
        json_yaml_left_scroll_top_line: 0.0,
        json_yaml_left_scroll_remainder: 0.0,
        json_yaml_left_viewport_height: 0.0,
        json_yaml_right_context_menu_open: false,
        json_yaml_right_context_menu_pos: None,
        json_yaml_right_scroll_top_line: 0.0,
        json_yaml_right_scroll_remainder: 0.0,
        json_yaml_right_viewport_height: 0.0,
        sql_tool_editor: {
            let remember = cfg
                .get("sql_tool_remember")
                .and_then(|value: &serde_json::Value| value.as_bool())
                .unwrap_or(false);
            if remember {
                iced::widget::text_editor::Content::with_text(&config::load_sql_tool_content())
            } else {
                iced::widget::text_editor::Content::new()
            }
        },
        sql_tool_remember: cfg
            .get("sql_tool_remember")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(false),
        sql_tool_loading: false,
        sql_tool_notification: None,
        sql_tool_context_menu_open: false,
        sql_tool_context_menu_pos: None,
        sql_tool_scroll_top_line: 0.0,
        sql_tool_scroll_remainder: 0.0,
        sql_tool_viewport_height: 0.0,
        redis_tool: crate::app::state::RedisToolUiState::from_persisted(redis_tool_persisted),
        html_tool_editor: {
            let remember = cfg
                .get("html_tool_remember")
                .and_then(|value: &serde_json::Value| value.as_bool())
                .unwrap_or(false);
            if remember {
                iced::widget::text_editor::Content::with_text(&config::load_html_tool_content())
            } else {
                iced::widget::text_editor::Content::new()
            }
        },
        html_tool_remember: cfg
            .get("html_tool_remember")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(false),
        html_tool_loading: false,
        html_tool_notification: None,
        html_tool_context_menu_open: false,
        html_tool_context_menu_pos: None,
        html_tool_scroll_top_line: 0.0,
        html_tool_scroll_remainder: 0.0,
        html_tool_viewport_height: 0.0,
        json_diff_left_editor: iced::widget::text_editor::Content::new(),
        json_diff_right_editor: iced::widget::text_editor::Content::new(),
        json_diff_results: Vec::new(),
        json_diff_notification: None,
        json_diff_notification_is_error: false,
        json_diff_loading: false,
        json_diff_left_context_menu_open: false,
        json_diff_left_context_menu_pos: None,
        json_diff_left_scroll_top_line: 0.0,
        json_diff_left_scroll_remainder: 0.0,
        json_diff_left_viewport_height: 0.0,
        json_diff_right_context_menu_open: false,
        json_diff_right_context_menu_pos: None,
        json_diff_right_scroll_top_line: 0.0,
        json_diff_right_scroll_remainder: 0.0,
        json_diff_right_viewport_height: 0.0,
        markdown_tool_editor: {
            const INITIAL_CONTENT: &str = "";
            iced::widget::text_editor::Content::with_text(INITIAL_CONTENT)
        },
        markdown_tool_context_menu_open: false,
        markdown_tool_context_menu_pos: None,
        markdown_tool_scroll_top_line: 0.0,
        markdown_tool_scroll_remainder: 0.0,
        markdown_tool_viewport_height: 0.0,
        markdown_tool_content: {
            const INITIAL_CONTENT: &str = "";
            iced::widget::markdown::Content::parse(INITIAL_CONTENT)
        },
        markdown_tool_view_mode: crate::app::components::markdown_editor::MarkdownViewMode::Split,
        markdown_tool_notification: None,
        markdown_tool_show_html2md: false,
        markdown_tool_html_editor: iced::widget::text_editor::Content::new(),
        markdown_tool_show_image: false,
        markdown_tool_image_url_input: String::new(),
        markdown_tool_remote_images: std::collections::HashMap::new(),
        markdown_tool_remote_images_loading: std::collections::HashSet::new(),
        markdown_tool_stream_enabled: false,
        markdown_tool_stream_chars: usize::MAX,
        mindmap_tabs: Vec::new(),
        mindmap_active_tab_id: None,
        workflow_state: crate::apps::workflow::state::WorkflowState::default(),
        pwd_digits: true,
        pwd_lowercase: true,
        pwd_uppercase: true,
        pwd_special: true,
        pwd_length_input: "12".to_string(),
        pwd_count_input: "1".to_string(),
        pwd_output_editor: iced::widget::text_editor::Content::new(),
        pwd_notification: None,
        pwd_notification_is_error: false,
        pwd_context_menu_open: false,
        pwd_context_menu_pos: None,
        pwd_scroll_top_line: 0.0,
        pwd_scroll_remainder: 0.0,
        pwd_viewport_height: 0.0,
        base_from: 10,
        base_to: 2,
        base_input: String::new(),
        base_output: String::new(),
        base_notification: None,
        ts_auto: true,
        ts_now_unix_sec: init_secs.to_string(),
        ts_now_unix_ms: init_ms.to_string(),
        ts_now_utc_str: init_utc,
        ts_input_ts: String::new(),
        ts_unit: crate::app::message::timestamp_tool::TsUnit::Seconds,
        ts_time_output: String::new(),
        ts_time_input: String::new(),
        ts_ts_output_sec: String::new(),
        ts_ts_output_ms: String::new(),
        ts_notification: None,
        qr_input: String::new(),
        qr_size: 256,
        qr_size_input: "256".to_string(),
        qr_level: crate::app::message::qr_tool::QrEcLevel::M,
        qr_image: None,
        qr_loading: false,
        qr_notification: None,
        qr_notification_is_error: false,
        qr_color_hex: "#000000".to_string(),
        qr_color_format: crate::app::views::design::models::ColorFormat::Hex,
        qr_icon_mode: crate::app::message::qr_tool::QrIconMode::None,
        qr_icon_bytes: None,
        qr_editor: iced::widget::text_editor::Content::new(),
        qr_scroll_top_line: 0.0,
        qr_scroll_remainder: 0.0,
        qr_viewport_height: 0.0,
        show_qr_color_picker: false,
        color_tool_color: iced::Color::from_rgb(0.0, 0.0, 0.0),
        color_tool_format: crate::app::views::design::models::ColorFormat::Hex,
        color_hex_input: "#000000ff".to_string(),
        color_rgb_input: "rgba(0, 0, 0, 1.00)".to_string(),
        color_hsl_input: "hsla(0, 0%, 0%, 1.00)".to_string(),
        color_hsv_input: "hsva(0, 0%, 0%, 1.00)".to_string(),
        color_notification: None,
        cleaner_clear_system_temp: true,
        cleaner_clear_app_cache: true,
        cleaner_clear_logs: true,
        cleaner_clear_package_cache: true,
        cleaner_clear_downloads: false,
        cleaner_empty_trash: false,
        cleaner_clear_installers: false,
        cleaner_clear_other_apps: true,
        cleaner_clear_wechat_work: false,
        cleaner_clear_wechat: false,
        cleaner_clear_qq: false,
        cleaner_clear_dingtalk: false,
        cleaner_clear_feishu: false,
        cleaner_clear_safari: false,
        cleaner_clear_chrome: false,
        cleaner_clear_edge: false,
        cleaner_clear_firefox: false,
        cleaner_clear_mail: false,
        cleaner_running: false,
        cleaner_cancelling: false,
        cleaner_scanning: false,
        cleaner_scanned: false,
        cleaner_animation_frame: 0,
        cleaner_scan_report: None,
        cleaner_tree_expanded: std::collections::HashSet::new(),
        cleaner_preview_mode: false,
        cleaner_output_editor: iced::widget::text_editor::Content::new(),
        cleaner_notification: None,
        cleaner_last_run_completed: false,
        cleaner_cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        large_file_root: std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
        large_file_scanning: false,
        large_file_scanned: false,
        large_file_animation_frame: 0,
        large_file_active_filter: "all".to_string(),
        large_file_report: None,
        large_file_notification: None,
        large_file_progress_label: "等待扫描".to_string(),
        large_file_current_path: String::new(),
        large_file_progress_value: 0.0,
        large_file_processed_files: 0,
        large_file_total_files: 0,
        large_file_selected_entries: std::collections::HashSet::new(),
        large_file_deleting: false,
        large_file_cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        large_file_progress_state: std::sync::Arc::new(std::sync::Mutex::new(
            crate::app::message::large_file_tool::LargeFileScanProgress::default(),
        )),
        active_tab_id: Some("home".to_string()),
        hovered_tab_id: None,
        hovered_recent_project: None,
        open_tabs: vec![AppTab {
            id: "home".to_string(),
            title: "首页".to_string(),
            screen: Screen::Home,
            project_path: None,
        }],
        apps_search_query: String::new(),
        web_bookmarks: cfg_web_bookmarks,
        show_web_links_menu: false,
        web_bookmark_title_input: String::new(),
        web_bookmark_url_input: String::new(),
        web_bookmark_width_input: String::new(),
        web_bookmark_height_input: String::new(),
        editing_web_bookmark: None,
        edit_web_bookmark_title_input: String::new(),
        edit_web_bookmark_url_input: String::new(),
        edit_web_bookmark_width_input: String::new(),
        edit_web_bookmark_height_input: String::new(),
        edit_web_bookmark_cookie_configs_editor: iced::widget::text_editor::Content::new(),
        notifications: Vec::new(),
        notifications_expanded: false,
        next_notification_id: 0,
        notifications_scroll_id: iced::widget::Id::new("notifications_scroll"),
        notification_editors: std::collections::HashMap::new(),
        copied_notification_id: None,
        active_toast: None,
        next_toast_id: 0,
        show_task_board: false,
        task_board_loading: false,
        task_board_tasks: Vec::new(),
        task_board_create_modal_open: false,
        task_board_settings_modal_open: false,
        task_board_settings_modal_tab: crate::app::state::TaskBoardSettingsModalTab::default(),
        task_board_is_import_mode: false,
        task_board_import_editor: iced::widget::text_editor::Content::new(),
        task_board_import_prompt_format: crate::app::task::TaskImportPromptFormat::Json,
        task_board_import_prompt_collapsed: true,
        task_board_column_has_vertical_scrollbar: std::collections::HashMap::new(),
        task_board_draft: crate::app::task::TaskDraft::default(),
        task_board_last_model: "auto".to_string(),
        task_board_last_acp_agent: cfg_acp_agent.clone(),
        task_board_selected_tasks: std::collections::HashSet::new(),
        task_board_bulk_active_status: None,
        task_board_bulk_priority_input: "999".to_string(),
        task_board_bulk_model_input: "auto".to_string(),
        task_board_bulk_acp_agent: cfg_acp_agent.clone(),
        task_board_selected_task: None,
        task_board_viewing_logs: None,
        task_board_log_cache: std::collections::HashMap::new(),
        task_board_editing_task_id: None,
        task_board_dragging: None,
        task_board_drag_pending: None,
        task_board_filter_status: None,
        task_board_filter_priority: None,
        task_board_sort_by_priority: true,
        task_board_sort_ascending: false,
        task_board_settings: settings::build_task_board_settings(&cfg),
        task_board_executor: crate::app::task::TaskExecutorState::new(),
        task_board_executor_running: false,
        task_board_next_refresh_at_ms: (init_ms as u64).saturating_add(60_000),
        task_board_next_scheduler_tick_at_ms: (init_ms as u64).saturating_add(1_000),
        task_board_next_auto_review_tick_at_ms: (init_ms as u64).saturating_add(30_000),
        task_board_next_auto_promote_tick_at_ms: (init_ms as u64).saturating_add(30_000),
        task_board_last_log_flush_at_ms: 0,
        task_board_log_scan_cursor: 0,
        task_board_timeout_scan_cursor: 0,
        task_board_schedule_scan_cursor: 0,
        task_board_executor_popover: false,
        task_board_bulk_executor_popover: false,
        task_board_bulk_model_popover: false,
        task_board_worktree_maintenance_in_flight: false,
        task_board_worktree_snapshot_loading: false,
        task_board_worktree_manual_action_kind: None,
        task_board_worktree_manual_confirm_kind: None,
        task_board_worktree_action_logs: Vec::new(),
        task_board_worktree_action_logs_visible_until_ms: None,
        task_board_worktree_action_log_rx: None,
        task_board_worktree_snapshot: None,
        task_board_last_worktree_snapshot_at_ms: 0,
        task_board_worktree_panel_expanded: false,
        task_board_worktree_pixel_office: cfg
            .get("task_board_worktree_pixel_office")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(false),
        task_board_context_menu: None,
        task_board_new_subtask_content: String::new(),
        task_board_desc_editor: iced::widget::text_editor::Content::new(),
        task_board_prompt_editor: iced::widget::text_editor::Content::new(),
        task_board_logs_editor: iced::widget::text_editor::Content::new(),
        task_board_logs_editor_id: iced::widget::Id::new("task_board_logs_editor"),
        task_board_logs_auto_scroll: true,
        task_board_logs_context_menu_open: false,
        task_board_logs_context_menu_pos: None,
        task_board_logs_scroll_top_line: 0.0,
        task_board_logs_scroll_remainder: 0.0,
        task_board_logs_viewport_height: 0.0,
        task_board_model_popover: false,
        task_board_clear_prompt_after_create: cfg
            .get("task_board_clear_prompt_after_create")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(true),
        task_board_close_after_create: cfg
            .get("task_board_close_after_create")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(true),
        task_board_close_after_edit: cfg
            .get("task_board_close_after_edit")
            .and_then(|value: &serde_json::Value| value.as_bool())
            .unwrap_or(true),
        task_board_create_submit_success: false,
        task_board_edit_submit_success: false,
        #[cfg(not(target_arch = "wasm32"))]
        independent_webview_children: Vec::new(),
    }
}
#[cfg(test)]
#[path = "build_tests.rs"]
mod build_tests;
