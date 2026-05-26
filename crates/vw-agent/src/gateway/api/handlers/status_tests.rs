use super::*;

#[test]
fn status_handlers_are_available() {
    let _ = handle_api_status;
    let _ = handle_api_health;
}
