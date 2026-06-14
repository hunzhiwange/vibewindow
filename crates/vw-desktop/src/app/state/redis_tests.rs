#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("redis_tests"));
}

use super::{
    RedisCommandOutputEntry, RedisConnectionConfig, RedisConnectionTab, RedisDetailTab,
    RedisHistoryRecord, RedisKeyAnalysis, RedisKeyPage, RedisKeyValueKind, RedisRuntimeOverview,
    RedisToolPersistedState, RedisToolUiState,
};
use crate::app::config::RedisToolGatewaySnapshot;

fn connection(id: &str, name: &str) -> RedisConnectionConfig {
    RedisConnectionConfig {
        id: id.to_string(),
        name: name.to_string(),
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
        updated_at_ms: 0,
    }
}

#[test]
fn redis_defaults_match_persisted_schema_and_connection_draft() {
    let persisted = RedisToolPersistedState::default();
    let ui = RedisToolUiState::from_persisted(persisted);

    assert_eq!(ui.default_load_count_input, "500");
    assert!(ui.connections.is_empty());
    assert!(ui.selected_connection_id.is_none());
    assert!(ui.draft_is_new);
    assert_eq!(ui.draft.host, "127.0.0.1");
    assert_eq!(ui.draft.port, "6379");
}

#[test]
fn redis_tab_titles_and_requirements_are_stable() {
    assert_eq!(RedisConnectionTab::Basic.title(), "基础");
    assert_eq!(RedisConnectionTab::Ssh.title(), "SSH");
    assert_eq!(RedisDetailTab::Overview.title(), "概览");
    assert!(RedisDetailTab::Overview.requires_runtime());
    assert!(RedisDetailTab::Keys.requires_keys());
    assert!(RedisDetailTab::Analysis.requires_key_analysis());
    assert!(!RedisDetailTab::Command.requires_runtime());
}

#[test]
fn redis_key_value_kinds_match_gateway_values_and_display() {
    let values = RedisKeyValueKind::ALL
        .into_iter()
        .map(|kind| kind.gateway_value())
        .collect::<Vec<_>>();

    assert_eq!(values, vec!["String", "Hash", "List", "Set", "Zset", "Stream", "ReJSON"]);
    assert_eq!(RedisKeyValueKind::ReJson.to_string(), "ReJSON");
}

#[test]
fn redis_from_persisted_loads_selected_connection_into_draft() {
    let persisted = RedisToolPersistedState {
        connections: vec![connection("one", "Local")],
        selected_connection_id: Some("one".to_string()),
        default_load_count: 0,
        history: vec![RedisHistoryRecord {
            time_ms: 1,
            connection_id: Some("one".to_string()),
            connection_label: "Local".to_string(),
            command: "PING".to_string(),
            args: String::new(),
            cost_ms: 2,
            is_write: false,
        }],
        ..Default::default()
    };

    let ui = RedisToolUiState::from_persisted(persisted);

    assert_eq!(ui.selected_connection_id.as_deref(), Some("one"));
    assert_eq!(ui.draft.name, "Local");
    assert_eq!(ui.key_browser_pattern, "app:*");
    assert!(!ui.draft_is_new);
    assert_eq!(ui.history_total, 1);
    assert_eq!(ui.default_load_count_input, "1");
}

#[test]
fn redis_gateway_request_helpers_track_loading_and_errors() {
    let mut ui = RedisToolUiState::from_persisted(Default::default());

    ui.begin_gateway_request("加载");
    assert!(ui.is_gateway_loading());
    assert!(ui.gateway_error.is_none());

    ui.fail_gateway_request("boom".to_string());
    assert!(!ui.is_gateway_loading());
    assert_eq!(ui.gateway_error.as_deref(), Some("boom"));

    ui.clear_gateway_error();
    assert!(ui.gateway_error.is_none());
}

#[test]
fn redis_apply_key_page_sorts_dedups_and_ignores_other_connections() {
    let mut ui = RedisToolUiState::from_persisted(RedisToolPersistedState {
        connections: vec![connection("one", "Local")],
        selected_connection_id: Some("one".to_string()),
        ..Default::default()
    });

    ui.apply_key_page(
        RedisKeyPage {
            connection_id: "other".to_string(),
            pattern: "*".to_string(),
            keys: vec!["ignored".to_string()],
            next_cursor: 1,
            has_more: true,
        },
        false,
    );
    assert!(ui.key_browser_items.is_empty());

    ui.apply_key_page(
        RedisKeyPage {
            connection_id: "one".to_string(),
            pattern: "app:*".to_string(),
            keys: vec!["app:b".to_string(), "app:a".to_string(), "app:a".to_string()],
            next_cursor: 42,
            has_more: true,
        },
        false,
    );

    assert_eq!(ui.key_browser_items, vec!["app:a", "app:b"]);
    assert_eq!(ui.key_browser_cursor, 42);
    assert!(ui.key_browser_has_more);
    assert!(ui.has_key_page_for_selected());
}

#[test]
fn redis_command_output_keeps_recent_sixty_entries() {
    let mut ui = RedisToolUiState::from_persisted(Default::default());

    for index in 0..61 {
        ui.push_command_output(RedisCommandOutputEntry {
            command: format!("GET {index}"),
            output: index.to_string(),
            cost_ms: index,
            is_error: false,
            time_ms: index,
        });
    }

    assert_eq!(ui.command_output.len(), 60);
    assert_eq!(ui.command_output[0].command, "GET 1");
}

#[test]
fn redis_key_analysis_matches_selected_connection_and_key() {
    let mut ui = RedisToolUiState::from_persisted(RedisToolPersistedState {
        connections: vec![connection("one", "Local")],
        selected_connection_id: Some("one".to_string()),
        ..Default::default()
    });

    ui.apply_key_analysis(RedisKeyAnalysis {
        connection_id: "one".to_string(),
        key: "app:a".to_string(),
        key_type: "string".to_string(),
        ttl_secs: -1,
        memory_usage_bytes: Some(12),
        preview_command: "GET app:a".to_string(),
        preview_output: "value".to_string(),
    });

    assert!(ui.has_key_analysis_for_selected());
    ui.select_key("app:b".to_string());
    assert!(!ui.has_key_analysis_for_selected());
    assert!(ui.key_analysis.is_none());
}

#[test]
fn redis_include_key_browser_item_expands_parent_paths() {
    let mut ui = RedisToolUiState::from_persisted(Default::default());

    ui.include_key_browser_item("app:user:1".to_string());
    ui.include_key_browser_item("app:user:1".to_string());

    assert_eq!(ui.key_browser_items, vec!["app:user:1"]);
    assert!(ui.key_tree_expanded_paths.contains("app"));
    assert!(ui.key_tree_expanded_paths.contains("app:user"));
    assert!(ui.key_tree_expanded_paths.contains("app:user:1"));
}

#[test]
fn redis_clear_runtime_state_resets_runtime_command_and_key_data() {
    let mut ui = RedisToolUiState::from_persisted(Default::default());
    ui.runtime_overview = Some(RedisRuntimeOverview { connection_id: "one".to_string(), ..Default::default() });
    ui.command_input = "PING".to_string();
    ui.info_filter = "server".to_string();
    ui.key_browser_items = vec!["a".to_string()];
    ui.selected_key = Some("a".to_string());
    ui.detail_tab = RedisDetailTab::Info;

    ui.clear_runtime_state();

    assert!(ui.runtime_overview.is_none());
    assert_eq!(ui.detail_tab, RedisDetailTab::Connection);
    assert!(ui.command_input.is_empty());
    assert!(ui.key_browser_items.is_empty());
    assert!(ui.selected_key.is_none());
}

#[test]
fn redis_apply_gateway_snapshot_replaces_persisted_state_and_pagination() {
    let mut ui = RedisToolUiState::from_persisted(Default::default());
    let snapshot = RedisToolGatewaySnapshot {
        persisted_state: RedisToolPersistedState {
            default_load_count: 25,
            connections: vec![connection("one", "Local")],
            selected_connection_id: Some("one".to_string()),
            history: Vec::new(),
            ..Default::default()
        },
        history_offset: 50,
        history_limit: 0,
        history_total: 75,
        history_has_more: true,
    };

    ui.apply_gateway_snapshot(snapshot);

    assert_eq!(ui.selected_connection_id.as_deref(), Some("one"));
    assert_eq!(ui.default_load_count_input, "25");
    assert_eq!(ui.history_page_offset, 50);
    assert_eq!(ui.history_page_limit, 1);
    assert_eq!(ui.history_total, 75);
    assert!(ui.history_has_more);
    assert_eq!(ui.draft.name, "Local");
}
