//! 验证队列 owner 在会话控制请求上的回合状态机行为。
//!
//! 队列 owner 既要服务当前活跃 prompt，也要在缺少活跃控制器时走 fallback 通道。
//! 这些测试固定取消延迟、模式/模型切换路由和关闭期拒绝策略，避免跨进程控制
//! 在生命周期边界上产生竞态或悄然吞掉请求。

use std::sync::{Arc, Mutex};

use vw_acp::{
    OutputErrorOrigin, QueueControlFuture, QueueOwnerActiveSessionController,
    QueueOwnerTurnController, QueueOwnerTurnControllerOptions, QueueOwnerTurnState,
};

/// 记录测试控制器收到的控制请求。
#[derive(Default)]
struct ActiveControllerState {
    has_active_prompt: bool,
    cancel_calls: usize,
    mode_calls: Vec<String>,
    model_calls: Vec<String>,
}

/// 用于模拟真实活跃会话控制器的轻量测试实现。
#[derive(Clone)]
struct TestActiveController {
    state: Arc<Mutex<ActiveControllerState>>,
}

impl QueueOwnerActiveSessionController for TestActiveController {
    /// 返回当前 prompt 是否已经进入可取消状态。
    fn has_active_prompt(&self) -> bool {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).has_active_prompt
    }

    /// 记录取消请求并返回成功，便于断言延迟取消只触发一次。
    fn request_cancel_active_prompt(&self) -> QueueControlFuture<bool> {
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            state.cancel_calls += 1;
            Ok(true)
        })
    }

    /// 记录模式切换请求，模拟活跃会话内的直接控制路径。
    fn set_session_mode(&self, mode_id: String) -> QueueControlFuture<()> {
        let state = self.state.clone();
        Box::pin(async move {
            state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).mode_calls.push(mode_id);
            Ok(())
        })
    }

    /// 记录模型切换请求，模拟活跃会话内的直接控制路径。
    fn set_session_model(&self, model_id: String) -> QueueControlFuture<()> {
        let state = self.state.clone();
        Box::pin(async move {
            state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .model_calls
                .push(model_id);
            Ok(())
        })
    }

    /// 当前测试不覆盖配置项控制；如果被调用说明路由走错了分支。
    fn set_session_config_option(
        &self,
        _config_id: String,
        _value: String,
    ) -> QueueControlFuture<agent_client_protocol::SetSessionConfigOptionResponse> {
        Box::pin(async move { unreachable!() })
    }
}

/// 创建带可观测 hook 的 turn controller，用于区分 timeout 包装和 fallback 路径。
fn create_controller(
    timeouts: Arc<Mutex<Vec<Option<u64>>>>,
    fallback_mode_calls: Arc<Mutex<Vec<String>>>,
    fallback_model_calls: Arc<Mutex<Vec<String>>>,
) -> QueueOwnerTurnController {
    QueueOwnerTurnController::new(QueueOwnerTurnControllerOptions {
        with_timeout: Arc::new(move |future, timeout_ms| {
            let timeouts = timeouts.clone();
            Box::pin(async move {
                timeouts.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push(timeout_ms);
                future.await
            })
        }),
        with_timeout_config_option: Arc::new(|future, _timeout_ms| future),
        set_session_mode_fallback: Arc::new(move |mode_id, _timeout_ms| {
            let fallback_mode_calls = fallback_mode_calls.clone();
            Box::pin(async move {
                fallback_mode_calls
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(mode_id);
                Ok(())
            })
        }),
        set_session_model_fallback: Arc::new(move |model_id, _timeout_ms| {
            let fallback_model_calls = fallback_model_calls.clone();
            Box::pin(async move {
                fallback_model_calls
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(model_id);
                Ok(())
            })
        }),
        set_session_config_option_fallback: Arc::new(|_, _, _| {
            Box::pin(async move { unreachable!() })
        }),
    })
}

/// prompt 尚未活跃时收到取消请求，应先挂起并在活跃后再实际转发。
#[tokio::test]
async fn queue_owner_turn_controller_defers_cancel_until_prompt_is_active() {
    let timeouts = Arc::new(Mutex::new(Vec::new()));
    let fallback_mode_calls = Arc::new(Mutex::new(Vec::new()));
    let fallback_model_calls = Arc::new(Mutex::new(Vec::new()));
    let mut controller = create_controller(timeouts, fallback_mode_calls, fallback_model_calls);

    controller.begin_turn();
    assert_eq!(controller.lifecycle_state(), QueueOwnerTurnState::Starting);
    assert!(controller.request_cancel().await.expect("cancel request should succeed"));
    assert!(controller.has_pending_cancel());

    let active_state = Arc::new(Mutex::new(ActiveControllerState {
        has_active_prompt: true,
        ..ActiveControllerState::default()
    }));
    controller
        .set_active_controller(Arc::new(TestActiveController { state: active_state.clone() }));
    controller.mark_prompt_active();

    assert!(controller.apply_pending_cancel().await.expect("deferred cancel should succeed"));
    assert_eq!(controller.lifecycle_state(), QueueOwnerTurnState::Active);
    assert!(!controller.has_pending_cancel());
    assert_eq!(
        active_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).cancel_calls,
        1
    );
}

/// 有活跃控制器时优先直接控制；清除后才走 fallback，避免控制请求丢失。
#[tokio::test]
async fn queue_owner_turn_controller_prefers_active_controller_and_falls_back_when_missing() {
    let timeouts = Arc::new(Mutex::new(Vec::new()));
    let fallback_mode_calls = Arc::new(Mutex::new(Vec::new()));
    let fallback_model_calls = Arc::new(Mutex::new(Vec::new()));
    let mut controller = create_controller(
        timeouts.clone(),
        fallback_mode_calls.clone(),
        fallback_model_calls.clone(),
    );

    let active_state = Arc::new(Mutex::new(ActiveControllerState {
        has_active_prompt: true,
        ..ActiveControllerState::default()
    }));
    controller
        .set_active_controller(Arc::new(TestActiveController { state: active_state.clone() }));

    controller
        .set_session_mode("mode-a", Some(25))
        .await
        .expect("active mode change should succeed");
    controller
        .set_session_model("model-a", Some(50))
        .await
        .expect("active model change should succeed");
    controller.clear_active_controller();
    controller
        .set_session_mode("mode-b", Some(75))
        .await
        .expect("fallback mode change should succeed");
    controller
        .set_session_model("model-b", None)
        .await
        .expect("fallback model change should succeed");

    let active_state = active_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(active_state.mode_calls, vec!["mode-a".to_string()]);
    assert_eq!(active_state.model_calls, vec!["model-a".to_string()]);
    assert_eq!(
        timeouts.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone(),
        vec![Some(25), Some(50)]
    );
    assert_eq!(
        fallback_mode_calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone(),
        vec!["mode-b".to_string()]
    );
    assert_eq!(
        fallback_model_calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone(),
        vec!["model-b".to_string()]
    );
}

/// 关闭流程开始后必须拒绝新的控制请求，让调用端知道应稍后重试或重建连接。
#[tokio::test]
async fn queue_owner_turn_controller_rejects_control_requests_while_closing() {
    let timeouts = Arc::new(Mutex::new(Vec::new()));
    let fallback_mode_calls = Arc::new(Mutex::new(Vec::new()));
    let fallback_model_calls = Arc::new(Mutex::new(Vec::new()));
    let mut controller = create_controller(timeouts, fallback_mode_calls, fallback_model_calls);

    controller.begin_closing();
    let error = controller
        .set_session_mode("mode-a", Some(10))
        .await
        .expect_err("closing controller should reject control requests");

    assert_eq!(error.to_string(), "Queue owner is closing");
    assert_eq!(error.detail_code(), Some("QUEUE_OWNER_SHUTTING_DOWN"));
    assert_eq!(error.origin(), Some(OutputErrorOrigin::Queue));
    assert_eq!(error.retryable(), Some(true));
}
