use super::*;

#[test]
fn not_found_error_display_uses_message_only() {
    let err = NotFoundError { message: "missing thing".to_string() };
    assert_eq!(err.to_string(), "missing thing");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn now_ms_returns_epoch_milliseconds() {
    assert!(now_ms() > 1_000_000_000_000);
}
