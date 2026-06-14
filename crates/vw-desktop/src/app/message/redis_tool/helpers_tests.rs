use super::*;

fn app() -> App {
    App::new().0
}

#[test]
fn current_default_load_count_uses_default_for_invalid_or_blank_input() {
    let mut app = app();

    app.redis_tool.default_load_count_input = "not-a-number".to_string();
    assert_eq!(current_default_load_count(&app), 500);

    app.redis_tool.default_load_count_input = "   ".to_string();
    assert_eq!(current_default_load_count(&app), 500);
}

#[test]
fn current_default_load_count_clamps_to_supported_range() {
    let mut app = app();

    app.redis_tool.default_load_count_input = "0".to_string();
    assert_eq!(current_default_load_count(&app), 1);

    app.redis_tool.default_load_count_input = "42".to_string();
    assert_eq!(current_default_load_count(&app), 42);

    app.redis_tool.default_load_count_input = "20000".to_string();
    assert_eq!(current_default_load_count(&app), 10_000);
}

#[test]
fn current_history_query_uses_state_and_trims_filter() {
    let mut app = app();
    app.redis_tool.history_page_offset = 25;
    app.redis_tool.history_page_limit = 10;
    app.redis_tool.history_filter = "  SET  ".to_string();
    app.redis_tool.history_only_write = true;

    let query = current_history_query(&app, None);

    assert_eq!(query.offset, Some(25));
    assert_eq!(query.limit, Some(REDIS_HISTORY_PAGE_SIZE));
    assert_eq!(query.connection_id, None);
    assert_eq!(query.query.as_deref(), Some("SET"));
    assert_eq!(query.only_write, Some(true));
}

#[test]
fn current_history_query_accepts_explicit_offset_and_empty_filter() {
    let mut app = app();
    app.redis_tool.history_page_offset = 25;
    app.redis_tool.history_page_limit = 75;
    app.redis_tool.history_filter = "   ".to_string();

    let query = current_history_query(&app, Some(100));

    assert_eq!(query.offset, Some(100));
    assert_eq!(query.limit, Some(75));
    assert_eq!(query.query, None);
    assert_eq!(query.only_write, Some(false));
}

#[test]
fn notify_success_sets_notification_and_clears_error() {
    let mut app = app();
    app.redis_tool.gateway_error = Some("old error".to_string());

    notify_success(&mut app, "done");

    assert_eq!(app.redis_tool.notification.as_deref(), Some("done"));
    assert_eq!(app.redis_tool.gateway_error, None);
}
