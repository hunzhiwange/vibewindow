use super::signal::Signal;

#[tokio::test]
async fn trigger_releases_waiter() {
    let mut signal = Signal::new();
    signal.trigger();
    signal.wait().await;
}
