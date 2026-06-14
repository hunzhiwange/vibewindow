use super::*;

fn app() -> App {
    App::new().0
}

#[test]
fn start_snapshot_reload_marks_gateway_loading() {
    let mut app = app();
    app.redis_tool.history_filter = " GET ".to_string();

    let _ = start_snapshot_reload(&mut app, 10, "加载历史", Some("ok".to_string()));

    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载历史"));
    assert_eq!(app.redis_tool.gateway_error, None);
}

#[test]
fn start_runtime_reload_marks_gateway_loading() {
    let mut app = app();

    let _ = start_runtime_reload(
        &mut app,
        "conn-1".to_string(),
        "加载 Redis 信息",
        Some("done".to_string()),
    );

    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载 Redis 信息"));
}

#[test]
fn start_key_page_reload_uses_loading_label_and_existing_cursor_for_append() {
    let mut app = app();
    app.redis_tool.key_browser_pattern = "  app:*  ".to_string();
    app.redis_tool.key_browser_cursor = 99;
    app.redis_tool.default_load_count_input = "25".to_string();

    let _ = start_key_page_reload(&mut app, "conn-1".to_string(), true, "加载更多键");

    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载更多键"));
}

#[test]
fn start_key_analysis_reload_marks_gateway_loading() {
    let mut app = app();

    let _ = start_key_analysis_reload(
        &mut app,
        "conn-1".to_string(),
        "app:key".to_string(),
        "加载 Key 内容",
    );

    assert_eq!(app.redis_tool.gateway_loading_label.as_deref(), Some("加载 Key 内容"));
}
