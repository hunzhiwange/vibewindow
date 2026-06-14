use super::*;
use crate::app::state::{RedisInfoEntry, RedisKeyAnalysis, RedisKeyspaceStat};

fn test_app() -> App {
    let (mut app, _task) = App::new();
    app.redis_tool.default_load_count_input = "25".to_string();
    app.redis_tool.draft.name = "cache-main".to_string();
    app.redis_tool.draft.host = "redis.example.com".to_string();
    app.redis_tool.draft.port = "6380".to_string();
    app.redis_tool.draft.db = "2".to_string();
    app.redis_tool.draft.username = "svc".to_string();
    app.redis_tool.draft.password = "secret".to_string();
    app.redis_tool.draft.key_pattern = "svc:*".to_string();
    app
}

fn selected_app() -> App {
    let mut app = test_app();
    app.redis_tool.selected_connection_id = Some("redis-main".to_string());
    app.redis_tool.draft_is_new = false;
    app
}

fn runtime(connection_id: &str) -> RedisRuntimeOverview {
    RedisRuntimeOverview {
        connection_id: connection_id.to_string(),
        connection_label: "main runtime".to_string(),
        server_version: "7.2.4".to_string(),
        os: "Darwin".to_string(),
        process_id: "4242".to_string(),
        used_memory_human: "8M".to_string(),
        used_memory_peak_human: "12M".to_string(),
        used_memory_lua_human: "64K".to_string(),
        connected_clients: 4,
        total_connections_received: 32,
        total_commands_processed: 128,
        keyspace: vec![RedisKeyspaceStat {
            db: "db2".to_string(),
            keys: 12,
            expires: 3,
            avg_ttl: 600,
        }],
        info_entries: vec![
            RedisInfoEntry { key: "redis_version".to_string(), value: "7.2.4".to_string() },
            RedisInfoEntry { key: "connected_clients".to_string(), value: "4".to_string() },
        ],
    }
}

fn key_analysis(connection_id: &str, key: &str) -> RedisKeyAnalysis {
    RedisKeyAnalysis {
        connection_id: connection_id.to_string(),
        key: key.to_string(),
        key_type: "String".to_string(),
        ttl_secs: 120,
        memory_usage_bytes: Some(2048),
        preview_command: format!("GET {key}"),
        preview_output: "cached-value".to_string(),
    }
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("mod_tests"));
}

#[test]
fn detail_tab_titles_are_stable() {
    assert_eq!(RedisDetailTab::Connection.title(), "连接配置");
    assert_eq!(RedisDetailTab::Keys.title(), "键树");
    assert_eq!(RedisDetailTab::Analysis.title(), "内容分析");
    assert_eq!(RedisDetailTab::Command.title(), "命令");
    assert_eq!(RedisDetailTab::Overview.title(), "概览");
    assert_eq!(RedisDetailTab::Info.title(), "INFO");
}

#[test]
fn detail_panel_builds_unselected_selected_and_busy_headers() {
    let mut app = test_app();
    keep_element(build_detail_panel(&app, false));

    app.redis_tool.draft.name.clear();
    app.redis_tool.selected_connection_id = Some("redis-main".to_string());
    keep_element(build_detail_panel(&app, true));

    app.redis_tool.draft.name = "cache-main".to_string();
    app.redis_tool.begin_gateway_request("刷新信息");
    keep_element(build_detail_panel(&app, false));
}

#[test]
fn connection_workspace_builds_new_saved_runtime_and_advanced_notes() {
    let mut app = test_app();
    keep_element(build_connection_workspace(&app, false, false));
    keep_element(build_connection_workspace(&app, true, true));

    app.redis_tool.selected_connection_id = Some("redis-main".to_string());
    app.redis_tool.draft_is_new = false;
    keep_element(build_connection_workspace(&app, false, false));

    app.redis_tool.runtime_overview = Some(runtime("redis-main"));
    keep_element(build_connection_workspace(&app, true, false));

    app.redis_tool.draft.ssh_tunnel.enabled = true;
    app.redis_tool.draft.sentinel.enabled = true;
    app.redis_tool.draft.use_cluster = true;
    app.redis_tool.draft.read_only = true;
    keep_element(build_connection_workspace(&app, false, true));
}

#[test]
fn connection_form_panel_builds_compact_regular_and_busy_states() {
    let mut app = selected_app();
    keep_element(build_connection_form_panel(&app, false, false));
    keep_element(build_connection_form_panel(&app, true, false));

    app.redis_tool.begin_gateway_request("测试连接");
    keep_element(build_connection_form_panel(&app, true, true));
}

#[test]
fn detail_workspace_uses_runtime_label_draft_label_and_masked_fallback() {
    let mut app = selected_app();
    app.redis_tool.draft.name.clear();
    keep_element(build_detail_workspace(&app, false, false));

    app.redis_tool.draft.name = "cache-main".to_string();
    keep_element(build_detail_workspace(&app, true, false));

    app.redis_tool.runtime_overview = Some(runtime("other-connection"));
    keep_element(build_detail_workspace(&app, false, false));

    app.redis_tool.runtime_overview = Some(runtime("redis-main"));
    keep_element(build_detail_workspace(&app, true, true));

    app.redis_tool.selected_connection_id = None;
    keep_element(build_detail_workspace(&app, false, false));
}

#[test]
fn detail_tab_bar_builds_all_tabs_when_idle_and_busy() {
    for tab in [
        RedisDetailTab::Keys,
        RedisDetailTab::Analysis,
        RedisDetailTab::Command,
        RedisDetailTab::Connection,
        RedisDetailTab::Overview,
        RedisDetailTab::Info,
    ] {
        let mut app = selected_app();
        app.redis_tool.detail_tab = tab;

        keep_element(build_detail_tab_bar(&app, false));
        keep_element(build_detail_tab_bar(&app, true));
    }
}

#[test]
fn active_detail_tab_builds_connection_keys_command_overview_and_info() {
    let mut app = selected_app();
    app.redis_tool.key_browser_pattern = "svc:*".to_string();
    app.redis_tool.key_browser_items =
        vec!["svc:user:1".to_string(), "svc:user:2".to_string(), "svc:session".to_string()];
    app.redis_tool.key_browser_has_more = true;
    app.redis_tool.key_tree_expanded_paths.insert("svc".to_string());
    app.redis_tool.selected_key = Some("svc:user:1".to_string());
    app.redis_tool.command_input = "PING".to_string();
    let current_runtime = runtime("redis-main");

    for tab in [
        RedisDetailTab::Connection,
        RedisDetailTab::Keys,
        RedisDetailTab::Command,
        RedisDetailTab::Overview,
        RedisDetailTab::Info,
    ] {
        app.redis_tool.detail_tab = tab;
        keep_element(build_active_detail_tab(&app, false, false, Some(&current_runtime)));
        keep_element(build_active_detail_tab(&app, true, true, Some(&current_runtime)));
    }

    app.redis_tool.info_filter = "missing".to_string();
    app.redis_tool.detail_tab = RedisDetailTab::Info;
    keep_element(build_active_detail_tab(&app, false, false, Some(&current_runtime)));

    app.redis_tool.selected_connection_id = None;
    app.redis_tool.detail_tab = RedisDetailTab::Connection;
    keep_element(build_active_detail_tab(&app, true, false, None));
    app.redis_tool.detail_tab = RedisDetailTab::Keys;
    keep_element(build_active_detail_tab(&app, true, false, None));
    app.redis_tool.detail_tab = RedisDetailTab::Command;
    keep_element(build_active_detail_tab(&app, true, false, None));
}

#[test]
fn active_detail_tab_builds_analysis_loaded_empty_and_hint_states() {
    let mut app = selected_app();
    app.redis_tool.detail_tab = RedisDetailTab::Analysis;
    app.redis_tool.selected_key = Some("svc:user:1".to_string());
    app.redis_tool.key_analysis = Some(key_analysis("redis-main", "svc:user:1"));
    keep_element(build_active_detail_tab(&app, false, false, None));

    app.redis_tool.key_analysis = Some(key_analysis("other-connection", "svc:user:1"));
    keep_element(build_active_detail_tab(&app, true, false, None));

    app.redis_tool.selected_key = None;
    keep_element(build_active_detail_tab(&app, false, true, None));

    app.redis_tool.selected_key = Some("svc:user:1".to_string());
    app.redis_tool.selected_connection_id = None;
    keep_element(build_active_detail_tab(&app, true, false, None));
}

#[test]
fn active_detail_tab_builds_runtime_empty_and_unselected_hints() {
    let mut app = selected_app();
    app.redis_tool.detail_tab = RedisDetailTab::Overview;
    keep_element(build_active_detail_tab(&app, false, false, None));

    app.redis_tool.draft.ssh_tunnel.enabled = true;
    keep_element(build_active_detail_tab(&app, true, true, None));

    app.redis_tool.detail_tab = RedisDetailTab::Info;
    keep_element(build_active_detail_tab(&app, false, false, None));

    app.redis_tool.selected_connection_id = None;
    app.redis_tool.detail_tab = RedisDetailTab::Overview;
    keep_element(build_active_detail_tab(&app, true, false, None));

    app.redis_tool.detail_tab = RedisDetailTab::Info;
    keep_element(build_active_detail_tab(&app, false, false, None));
}

#[test]
fn detail_hint_state_builds_static_empty_panel() {
    keep_element(build_detail_hint_state("标题", "说明"));
}
