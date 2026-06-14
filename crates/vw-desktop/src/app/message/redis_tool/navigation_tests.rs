#![allow(unused_must_use)]
use super::*;
use crate::app::config::RedisToolGatewaySnapshot;
use crate::app::state::{
    RedisConnectionConfig, RedisHistoryRecord, RedisInfoEntry, RedisKeyAnalysis, RedisKeyPage,
    RedisRuntimeOverview, RedisToolPersistedState,
};

fn app() -> App {
    App::new().0
}

fn connection(id: &str) -> RedisConnectionConfig {
    RedisConnectionConfig {
        id: id.to_string(),
        name: "Primary".to_string(),
        host: "127.0.0.1".to_string(),
        port: 6379,
        db: 0,
        username: String::new(),
        password: String::new(),
        use_tls: false,
        tls_cert: Default::default(),
        ssh_tunnel: Default::default(),
        sentinel: Default::default(),
        use_cluster: false,
        read_only: false,
        key_pattern: "app:*".to_string(),
        last_used_ms: None,
        updated_at_ms: 1,
    }
}

fn snapshot(selected: Option<&str>) -> RedisToolGatewaySnapshot {
    RedisToolGatewaySnapshot {
        persisted_state: RedisToolPersistedState {
            default_load_count: 250,
            connections: vec![connection("conn-1")],
            history: vec![RedisHistoryRecord {
                time_ms: 1,
                connection_id: Some("conn-1".to_string()),
                connection_label: "Primary".to_string(),
                command: "GET".to_string(),
                args: "app:key".to_string(),
                cost_ms: 2,
                is_write: false,
            }],
            selected_connection_id: selected.map(str::to_string),
            ..Default::default()
        },
        history_offset: 50,
        history_limit: 25,
        history_total: 75,
        history_has_more: true,
    }
}

fn runtime(connection_id: &str) -> RedisRuntimeOverview {
    RedisRuntimeOverview {
        connection_id: connection_id.to_string(),
        connection_label: "Primary".to_string(),
        server_version: "7.2".to_string(),
        info_entries: vec![RedisInfoEntry {
            key: "redis_version".to_string(),
            value: "7.2".to_string(),
        }],
        ..Default::default()
    }
}

fn key_page(connection_id: &str) -> RedisKeyPage {
    RedisKeyPage {
        connection_id: connection_id.to_string(),
        pattern: "app:*".to_string(),
        keys: vec!["app:b".to_string(), "app:a".to_string()],
        next_cursor: 10,
        has_more: true,
    }
}

fn analysis(connection_id: &str, key: &str) -> RedisKeyAnalysis {
    RedisKeyAnalysis {
        connection_id: connection_id.to_string(),
        key: key.to_string(),
        key_type: "string".to_string(),
        preview_output: "value".to_string(),
        ..Default::default()
    }
}

#[test]
fn modal_openers_close_competing_modals() {
    let mut app = app();
    app.redis_tool.show_history_modal = true;
    app.redis_tool.show_connection_modal = true;
    app.redis_tool.show_create_key_modal = true;

    open_settings_modal(&mut app);

    assert!(app.redis_tool.show_settings_modal);
    assert!(!app.redis_tool.show_history_modal);
    assert!(!app.redis_tool.show_connection_modal);
    assert!(!app.redis_tool.show_create_key_modal);

    open_connection_modal(&mut app);
    assert!(app.redis_tool.show_connection_modal);
    assert!(!app.redis_tool.show_settings_modal);

    close_connection_modal(&mut app);
    close_settings_modal(&mut app);
    close_history_modal(&mut app);
    close_create_key_modal(&mut app);

    assert!(!app.redis_tool.show_connection_modal);
    assert!(!app.redis_tool.show_settings_modal);
    assert!(!app.redis_tool.show_history_modal);
    assert!(!app.redis_tool.show_create_key_modal);
}

#[test]
fn open_create_key_modal_requires_selected_connection() {
    let mut app = app();

    open_create_key_modal(&mut app);

    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));
    assert!(!app.redis_tool.show_create_key_modal);

    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.gateway_error = None;
    open_create_key_modal(&mut app);

    assert!(app.redis_tool.show_create_key_modal);
    assert_eq!(app.redis_tool.gateway_error, None);
}

#[test]
fn new_connection_resets_selection_runtime_and_opens_modal() {
    let mut app = app();
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.command_input = "PING".to_string();
    app.redis_tool.key_browser_items = vec!["app:key".to_string()];
    app.redis_tool.connection_search_query = "old".to_string();

    new_connection(&mut app);

    assert_eq!(app.redis_tool.selected_connection_id, None);
    assert!(app.redis_tool.draft_is_new);
    assert!(app.redis_tool.show_connection_modal);
    assert!(app.redis_tool.command_input.is_empty());
    assert!(app.redis_tool.key_browser_items.is_empty());
    assert!(app.redis_tool.connection_search_query.is_empty());
    assert_eq!(app.redis_tool.notification.as_deref(), Some("已打开新建连接"));
}

#[test]
fn select_connection_loads_existing_draft_and_starts_request() {
    let mut app = app();
    app.redis_tool.connections = vec![connection("conn-1")];

    select_connection(&mut app, "conn-1".to_string());

    assert_eq!(app.redis_tool.selected_connection_id.as_deref(), Some("conn-1"));
    assert_eq!(app.redis_tool.draft.name, "Primary");
    assert!(!app.redis_tool.draft_is_new);
    assert_eq!(app.redis_tool.detail_tab, RedisDetailTab::Connection);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("切换连接"));
}

#[test]
fn select_connection_ignores_when_loading() {
    let mut app = app();
    app.redis_tool.connections = vec![connection("conn-1")];
    app.redis_tool.begin_gateway_request("busy");

    select_connection(&mut app, "conn-1".to_string());

    assert_eq!(app.redis_tool.selected_connection_id, None);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("busy"));
}

#[test]
fn select_connection_completed_applies_snapshot_runtime_and_keys() {
    let mut app = app();
    app.redis_tool.detail_tab = RedisDetailTab::Keys;

    select_connection_completed(
        &mut app,
        Ok((snapshot(Some("conn-1")), Some(Ok(runtime("conn-1"))), Some(Ok(key_page("conn-1"))))),
    );

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert_eq!(app.redis_tool.selected_connection_id.as_deref(), Some("conn-1"));
    assert!(app.redis_tool.runtime_overview.is_some());
    assert_eq!(app.redis_tool.key_browser_items, vec!["app:a".to_string(), "app:b".to_string()]);
    assert_eq!(app.redis_tool.notification.as_deref(), Some("已展开连接并加载 键树 标签"));
}

#[test]
fn select_connection_completed_keeps_first_nested_error() {
    let mut app = app();

    select_connection_completed(
        &mut app,
        Ok((
            snapshot(Some("conn-1")),
            Some(Err("runtime failed".to_string())),
            Some(Err("keys failed".to_string())),
        )),
    );

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("runtime failed"));

    select_connection_completed(&mut app, Err("activate failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("activate failed"));
}

#[test]
fn detail_tab_changed_loads_only_missing_selected_data() {
    let mut app = app();
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());

    detail_tab_changed(&mut app, RedisDetailTab::Overview);

    assert_eq!(app.redis_tool.detail_tab, RedisDetailTab::Overview);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载 Redis 信息"));

    app.redis_tool.finish_gateway_request();
    app.redis_tool.apply_runtime_overview(runtime("conn-1"));
    detail_tab_changed(&mut app, RedisDetailTab::Info);

    assert_eq!(app.redis_tool.detail_tab, RedisDetailTab::Info);
    assert_eq!(app.redis_tool.gateway_loading_label, None);
}

#[test]
fn refresh_and_key_loading_require_selected_connection() {
    let mut app = app();

    refresh_selected_runtime(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));

    app.redis_tool.gateway_error = None;
    reload_selected_keys(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.key_browser_has_more = true;
    load_more_keys(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));
}

#[test]
fn select_and_refresh_key_analysis_validate_selection() {
    let mut app = app();

    select_key(&mut app, "app:key".to_string());
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    refresh_selected_key_analysis(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先在键树中选择一个 Key"));

    app.redis_tool.gateway_error = None;
    select_key(&mut app, "app:key".to_string());
    assert_eq!(app.redis_tool.selected_key.as_deref(), Some("app:key"));
    assert_eq!(app.redis_tool.detail_tab, RedisDetailTab::Analysis);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载 Key 内容"));
}

#[test]
fn key_analysis_runtime_and_key_page_loaded_apply_only_matching_connection() {
    let mut app = app();
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.selected_key = Some("app:key".to_string());
    app.redis_tool.begin_gateway_request("busy");

    key_analysis_loaded(
        &mut app,
        "conn-1".to_string(),
        "app:key".to_string(),
        Ok(analysis("conn-1", "app:key")),
    );
    assert!(app.redis_tool.key_analysis.is_some());
    assert_eq!(app.redis_tool.gateway_loading_label, None);

    app.redis_tool.begin_gateway_request("busy");
    runtime_loaded(
        &mut app,
        "other".to_string(),
        Some("refreshed".to_string()),
        Ok(runtime("other")),
    );
    assert!(app.redis_tool.runtime_overview.is_none());
    assert_eq!(app.redis_tool.notification.as_deref(), Some("refreshed"));

    key_page_loaded(&mut app, "conn-1".to_string(), false, Ok(key_page("conn-1")));
    assert_eq!(app.redis_tool.key_browser_items, vec!["app:a".to_string(), "app:b".to_string()]);
}

#[test]
fn loaded_error_results_fail_gateway_request() {
    let mut app = app();

    key_analysis_loaded(&mut app, "conn-1".to_string(), "k".to_string(), Err("bad key".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("bad key"));

    runtime_loaded(&mut app, "conn-1".to_string(), None, Err("bad runtime".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("bad runtime"));

    key_page_loaded(&mut app, "conn-1".to_string(), false, Err("bad page".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("bad page"));
}

#[test]
fn simple_filters_and_toggles_update_state() {
    let mut app = app();

    search_connections_changed(&mut app, "prod".to_string());
    key_browser_pattern_changed(&mut app, "app:*".to_string());
    toggle_key_tree_path(&mut app, "app".to_string());
    info_filter_changed(&mut app, "memory".to_string());

    assert_eq!(app.redis_tool.connection_search_query, "prod");
    assert_eq!(app.redis_tool.key_browser_pattern, "app:*");
    assert!(app.redis_tool.key_tree_expanded_paths.contains("app"));
    assert_eq!(app.redis_tool.info_filter, "memory");

    toggle_key_tree_path(&mut app, "app".to_string());
    assert!(!app.redis_tool.key_tree_expanded_paths.contains("app"));
}

#[test]
fn history_pagination_and_filters_start_snapshot_reload_when_allowed() {
    let mut app = app();
    app.redis_tool.history_page_offset = 100;
    app.redis_tool.history_page_limit = 25;

    history_previous_page(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载历史"));

    app.redis_tool.finish_gateway_request();
    app.redis_tool.history_has_more = true;
    history_next_page(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载历史"));

    app.redis_tool.finish_gateway_request();
    history_filter_changed(&mut app, "SET".to_string());
    assert_eq!(app.redis_tool.history_filter, "SET");
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("筛选历史"));

    app.redis_tool.finish_gateway_request();
    history_only_write_toggled(&mut app, true);
    assert!(app.redis_tool.history_only_write);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("筛选历史"));
}

#[test]
fn history_pagination_guards_when_loading_or_unavailable() {
    let mut app = app();

    history_previous_page(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label, None);

    app.redis_tool.history_page_offset = 10;
    app.redis_tool.begin_gateway_request("busy");
    history_previous_page(&mut app);
    history_next_page(&mut app);
    history_filter_changed(&mut app, "GET".to_string());
    history_only_write_toggled(&mut app, true);

    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("busy"));
    assert_eq!(app.redis_tool.history_filter, "GET");
    assert!(app.redis_tool.history_only_write);
}

#[test]
fn snapshot_loaded_applies_state_or_records_error() {
    let mut app = app();
    app.redis_tool.begin_gateway_request("loading");

    snapshot_loaded(&mut app, Some("loaded".to_string()), Ok(snapshot(Some("conn-1"))));

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert_eq!(app.redis_tool.selected_connection_id.as_deref(), Some("conn-1"));
    assert_eq!(app.redis_tool.notification.as_deref(), Some("loaded"));

    snapshot_loaded(&mut app, None, Err("load failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("load failed"));
}

#[test]
fn clear_notification_and_gateway_error_reset_transient_state() {
    let mut app = app();
    app.redis_tool.notification = Some("done".to_string());
    app.redis_tool.gateway_error = Some("bad".to_string());

    clear_notification(&mut app);
    clear_gateway_error(&mut app);

    assert_eq!(app.redis_tool.notification, None);
    assert_eq!(app.redis_tool.gateway_error, None);
}
