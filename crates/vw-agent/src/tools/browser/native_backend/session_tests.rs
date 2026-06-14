use super::*;

#[test]
fn active_client_reports_missing_session() {
    let state = NativeBrowserState::default();

    let err = state.active_client().expect_err("session should be absent");

    assert!(err.to_string().contains("No active native browser session"));
}

#[tokio::test]
async fn reset_session_is_idempotent_without_client() {
    let mut state = NativeBrowserState::default();

    state.reset_session().await;
    state.reset_session().await;

    assert!(state.active_client().is_err());
}

#[tokio::test]
async fn ensure_session_reports_webdriver_connection_context() {
    let mut state = NativeBrowserState::default();

    let err = state
        .ensure_session(true, "http://127.0.0.1:9", Some("  "))
        .await
        .expect_err("closed webdriver port should fail");

    assert!(err.to_string().contains("Failed to connect to WebDriver"));
}
