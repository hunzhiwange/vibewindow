use super::*;
use crate::queue_owner_turn_controller::QueueOwnerActiveSessionController;

#[test]
fn map_finish_reason_maps_known_values_and_defaults() {
    assert_eq!(map_finish_reason(Some("length")), StopReason::MaxTokens);
    assert_eq!(map_finish_reason(Some("max_turn_requests")), StopReason::MaxTurnRequests);
    assert_eq!(map_finish_reason(Some("refusal")), StopReason::Refusal);
    assert_eq!(map_finish_reason(Some("cancelled")), StopReason::Cancelled);
    assert_eq!(map_finish_reason(Some("other")), StopReason::EndTurn);
    assert_eq!(map_finish_reason(None), StopReason::EndTurn);
}

#[tokio::test]
async fn noop_active_session_controller_denies_active_cancel() {
    let controller = NoopActiveSessionController;

    assert!(!controller.has_active_prompt());
    assert!(!controller.request_cancel_active_prompt().await.unwrap());
    assert!(controller.set_session_mode("default".to_string()).await.is_ok());
    assert!(controller.set_session_model("model".to_string()).await.is_ok());
    assert!(
        controller.set_session_config_option("id".to_string(), "value".to_string()).await.is_err()
    );
}
