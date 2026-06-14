use super::*;
use crate::app::agent::channels::traits::Channel;
use std::sync::atomic::Ordering;

fn channel(room_id: &str) -> MatrixChannel {
    MatrixChannel::new(
        "http://127.0.0.1:9".to_string(),
        "token".to_string(),
        room_id.to_string(),
        Vec::new(),
    )
}

#[test]
fn channel_name_is_matrix() {
    assert_eq!(channel("!room:server").name(), "matrix");
}

#[tokio::test]
async fn health_check_fails_when_otk_conflict_was_detected() {
    let ch = channel("!room:server");
    ch.otk_conflict_detected.store(true, Ordering::Relaxed);

    assert!(!ch.health_check().await);
}

#[tokio::test]
async fn health_check_fails_for_invalid_room_reference() {
    let ch = channel("not-a-room-reference");

    assert!(!ch.health_check().await);
}

#[tokio::test]
async fn listen_returns_otk_recovery_error_when_conflict_is_set() {
    let ch = channel("!room:server");
    ch.otk_conflict_detected.store(true, Ordering::Relaxed);
    let (tx, _rx) = tokio::sync::mpsc::channel(1);

    let err = ch.listen(tx).await.expect_err("OTK conflict should stop listener");

    assert!(err.to_string().contains("one-time key upload conflict"));
}
