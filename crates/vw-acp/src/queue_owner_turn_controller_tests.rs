use super::*;
use std::sync::Arc;

fn ok_void() -> QueueControlFuture<()> {
    Box::pin(async { Ok(()) })
}

fn options() -> QueueOwnerTurnControllerOptions {
    QueueOwnerTurnControllerOptions {
        with_timeout: Arc::new(|future, _| future),
        with_timeout_config_option: Arc::new(|future, _| future),
        set_session_mode_fallback: Arc::new(|_, _| ok_void()),
        set_session_model_fallback: Arc::new(|_, _| ok_void()),
        set_session_config_option_fallback: Arc::new(|_, _, _| {
            Box::pin(async { unreachable!("config option fallback is not used in these tests") })
        }),
    }
}

#[test]
fn turn_controller_tracks_lifecycle_state() {
    let mut controller = QueueOwnerTurnController::new(options());

    assert_eq!(controller.lifecycle_state(), QueueOwnerTurnState::Idle);
    controller.begin_turn();
    assert_eq!(controller.lifecycle_state(), QueueOwnerTurnState::Starting);
    controller.mark_prompt_active();
    assert_eq!(controller.lifecycle_state(), QueueOwnerTurnState::Active);
    controller.end_turn();
    assert_eq!(controller.lifecycle_state(), QueueOwnerTurnState::Idle);
}

#[tokio::test]
async fn closing_controller_rejects_control_requests() {
    let mut controller = QueueOwnerTurnController::new(options());
    controller.begin_closing();

    let error = controller
        .set_session_mode("plan", Some(10))
        .await
        .expect_err("closing controller rejects control");
    assert_eq!(error.detail_code(), Some("QUEUE_OWNER_SHUTTING_DOWN"));
    assert_eq!(error.retryable(), Some(true));
}
