use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn router_builds_with_string_state() {
    let _: axum::Router<String> = router();
}
