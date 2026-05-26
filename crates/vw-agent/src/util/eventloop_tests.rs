use super::eventloop::wait;

#[tokio::test]
async fn wait_yields_without_panicking() {
    wait().await;
}
