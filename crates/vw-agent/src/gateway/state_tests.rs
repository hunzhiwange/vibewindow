use super::*;

#[test]
fn app_state_type_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AppState>();
}
