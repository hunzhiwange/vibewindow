#![allow(unused_must_use)]
use super::*;
use crate::app::config::RedisToolGatewaySnapshot;
use crate::app::state::{
    RedisCommandOutputEntry, RedisConnectionConfig, RedisDetailTab, RedisKeyAnalysis,
    RedisKeyValueKind, RedisToolPersistedState,
};
use vw_gateway_client::GatewayRedisConnectionTestResponse;

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
        key_pattern: "*".to_string(),
        last_used_ms: None,
        updated_at_ms: 1,
    }
}

fn snapshot(selected: Option<&str>) -> RedisToolGatewaySnapshot {
    RedisToolGatewaySnapshot {
        persisted_state: RedisToolPersistedState {
            default_load_count: 750,
            connections: vec![connection("conn-1")],
            selected_connection_id: selected.map(str::to_string),
            ..Default::default()
        },
        history_offset: 0,
        history_limit: 50,
        history_total: 0,
        history_has_more: false,
    }
}

fn valid_draft(app: &mut App) {
    app.redis_tool.draft.name = "Primary".to_string();
    app.redis_tool.draft.host = "127.0.0.1".to_string();
    app.redis_tool.draft.port = "6379".to_string();
    app.redis_tool.draft.db = "0".to_string();
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
fn save_draft_validates_before_starting_gateway_request() {
    let mut app = app();
    app.redis_tool.draft.name.clear();

    save_draft(&mut app);

    assert!(app.redis_tool.gateway_error.as_deref().is_some_and(|error| error.contains("名称")));
    assert_eq!(app.redis_tool.gateway_loading_label, None);
}

#[test]
fn save_draft_starts_create_or_update_request_when_valid() {
    let mut app = app();
    valid_draft(&mut app);

    save_draft(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("保存连接"));

    app.redis_tool.finish_gateway_request();
    app.redis_tool.draft_is_new = false;
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    save_draft(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("保存连接"));
}

#[test]
fn save_draft_completed_applies_snapshot_and_closes_modal_or_records_error() {
    let mut app = app();
    app.redis_tool.show_connection_modal = true;
    app.redis_tool.connection_search_query = "prod".to_string();
    app.redis_tool.begin_gateway_request("保存连接");

    save_draft_completed(&mut app, Ok(snapshot(Some("conn-1"))));

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert_eq!(app.redis_tool.selected_connection_id.as_deref(), Some("conn-1"));
    assert!(!app.redis_tool.show_connection_modal);
    assert!(app.redis_tool.connection_search_query.is_empty());
    assert_eq!(app.redis_tool.notification.as_deref(), Some("连接配置已保存"));

    save_draft_completed(&mut app, Err("save failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("save failed"));
}

#[test]
fn delete_and_test_selected_require_selection_and_respect_loading() {
    let mut app = app();

    delete_selected(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label, None);

    test_selected(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.begin_gateway_request("busy");
    delete_selected(&mut app);
    test_selected(&mut app);

    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("busy"));
    assert_eq!(app.redis_tool.gateway_error, None);
}

#[test]
fn selected_operations_start_gateway_request_when_selected() {
    let mut app = app();
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());

    delete_selected(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("删除连接"));

    app.redis_tool.finish_gateway_request();
    test_selected(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("测试连接"));
}

#[test]
fn test_selected_completed_notifies_or_records_error() {
    let mut app = app();
    app.redis_tool.begin_gateway_request("测试连接");

    test_selected_completed(
        &mut app,
        Ok((
            GatewayRedisConnectionTestResponse {
                ok: true,
                message: "PONG".to_string(),
                latency_ms: 12,
            },
            snapshot(Some("conn-1")),
        )),
    );

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert_eq!(app.redis_tool.notification.as_deref(), Some("连接测试成功：PONG（12 ms）"));

    test_selected_completed(&mut app, Err("test failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("test failed"));
}

#[test]
fn copy_selected_uri_notifies_on_valid_draft_and_errors_on_invalid_draft() {
    let mut app = app();
    valid_draft(&mut app);

    copy_selected_uri(&mut app);
    assert_eq!(app.redis_tool.notification.as_deref(), Some("连接 URI 已复制"));

    app.redis_tool.draft.port = "bad".to_string();
    copy_selected_uri(&mut app);
    assert!(app.redis_tool.gateway_error.is_some());
}

#[test]
fn export_and_import_start_loading_and_complete_all_result_shapes() {
    let mut app = app();

    export_configs(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("导出配置"));
    export_completed(&mut app, Ok(None));
    assert_eq!(app.redis_tool.gateway_loading_label, None);

    export_completed(&mut app, Ok(Some(snapshot(Some("conn-1")))));
    assert_eq!(app.redis_tool.notification.as_deref(), Some("连接配置已导出"));
    export_completed(&mut app, Err("export failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("export failed"));

    import_configs(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("导入配置"));
    import_completed(&mut app, Ok(None));
    assert_eq!(app.redis_tool.gateway_loading_label, None);

    import_completed(&mut app, Ok(Some(snapshot(Some("conn-1")))));
    assert_eq!(app.redis_tool.notification.as_deref(), Some("连接配置已导入"));
    import_completed(&mut app, Err("import failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("import failed"));
}

#[test]
fn default_load_count_controls_update_and_clamp_values() {
    let mut app = app();

    default_load_count_changed(&mut app, "25".to_string());
    assert_eq!(app.redis_tool.default_load_count_input, "25");

    increase_default_load_count(&mut app);
    assert_eq!(app.redis_tool.default_load_count_input, "125");

    decrease_default_load_count(&mut app);
    assert_eq!(app.redis_tool.default_load_count_input, "25");

    app.redis_tool.default_load_count_input = "bad".to_string();
    save_default_load_count(&mut app);
    assert_eq!(
        app.redis_tool.gateway_error.as_deref(),
        Some("默认加载数量必须是 1-10000 的整数")
    );

    app.redis_tool.gateway_error = None;
    app.redis_tool.default_load_count_input = "20000".to_string();
    save_default_load_count(&mut app);
    assert_eq!(app.redis_tool.default_load_count_input, "10000");
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("保存默认加载数量"));
}

#[test]
fn create_key_inputs_validate_selection_and_name() {
    let mut app = app();

    create_key_name_changed(&mut app, "  app:key  ".to_string());
    create_key_type_changed(&mut app, RedisKeyValueKind::Hash);

    assert_eq!(app.redis_tool.create_key_draft.name, "  app:key  ");
    assert_eq!(app.redis_tool.create_key_draft.key_type, RedisKeyValueKind::Hash);

    confirm_create_key(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.create_key_draft.name = "  ".to_string();
    confirm_create_key(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请输入 Key 名称"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.create_key_draft.name = "  app:key  ".to_string();
    confirm_create_key(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("创建 Key"));
}

#[test]
fn create_key_completed_updates_matching_connection_and_notifies() {
    let mut app = app();
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.show_create_key_modal = true;
    app.redis_tool.begin_gateway_request("创建 Key");

    create_key_completed(
        &mut app,
        "conn-1".to_string(),
        "app:key".to_string(),
        Ok(analysis("conn-1", "app:key")),
    );

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert!(!app.redis_tool.show_create_key_modal);
    assert!(app.redis_tool.key_browser_items.contains(&"app:key".to_string()));
    assert_eq!(app.redis_tool.detail_tab, RedisDetailTab::Analysis);
    assert_eq!(app.redis_tool.notification.as_deref(), Some("Key 已创建并进入内容分析"));

    create_key_completed(
        &mut app,
        "conn-1".to_string(),
        "bad".to_string(),
        Err("create failed".to_string()),
    );
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("create failed"));
}

#[test]
fn command_input_and_run_command_validate_preconditions() {
    let mut app = app();

    command_input_changed(&mut app, " PING ".to_string());
    assert_eq!(app.redis_tool.command_input, " PING ");

    run_command(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请先选择已保存的连接"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.command_input = "   ".to_string();
    run_command(&mut app);
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("请输入 Redis 命令"));

    app.redis_tool.gateway_error = None;
    app.redis_tool.command_input = "PING".to_string();
    run_command(&mut app);
    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("执行 Redis 命令"));
}

#[test]
fn command_completed_appends_matching_output_or_records_error() {
    let mut app = app();
    app.redis_tool.selected_connection_id = Some("conn-1".to_string());
    app.redis_tool.begin_gateway_request("执行 Redis 命令");

    command_completed(
        &mut app,
        "conn-1".to_string(),
        Ok(RedisCommandOutputEntry {
            command: "PING".to_string(),
            output: "PONG".to_string(),
            cost_ms: 3,
            is_error: false,
            time_ms: 4,
        }),
    );

    assert_eq!(app.redis_tool.gateway_loading_label, None);
    assert_eq!(app.redis_tool.command_output.len(), 1);
    assert_eq!(app.redis_tool.command_output[0].output, "PONG");

    command_completed(&mut app, "conn-1".to_string(), Err("command failed".to_string()));
    assert_eq!(app.redis_tool.gateway_error.as_deref(), Some("command failed"));
}
