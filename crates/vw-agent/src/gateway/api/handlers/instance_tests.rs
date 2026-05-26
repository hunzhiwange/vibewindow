use super::*;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}
