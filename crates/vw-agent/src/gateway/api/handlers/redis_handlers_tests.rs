use super::*;

#[test]
fn redis_handler_functions_are_available() {
    let _ = redis_settings_get;
    let _ = redis_connections_list;
    let _ = redis_connection_get;
    let _ = redis_history_list;
}
