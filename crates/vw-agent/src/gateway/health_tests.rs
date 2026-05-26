use super::*;

#[test]
fn health_handlers_are_available() {
    let _ = handle_health;
    let _ = handle_metrics;
}
