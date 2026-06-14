use super::*;
use std::future::pending;
use std::time::Duration;

#[test]
fn classify_health_result_maps_success_failure_and_timeout() {
    assert_eq!(classify_health_result(&Ok(true)), ChannelHealthState::Healthy);
    assert_eq!(classify_health_result(&Ok(false)), ChannelHealthState::Unhealthy);
}

#[tokio::test]
async fn classify_health_result_maps_elapsed_to_timeout() {
    let elapsed = tokio::time::timeout(Duration::from_millis(0), pending::<()>()).await;

    assert_eq!(classify_health_result(&elapsed.map(|_| true)), ChannelHealthState::Timeout);
}

#[tokio::test]
async fn doctor_channels_returns_ok_when_no_realtime_channels_are_configured() {
    doctor_channels(Config::default()).await.expect("empty config should be a no-op");
}
