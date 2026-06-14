use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct ImmediateChannel {
    listens: AtomicUsize,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Channel for ImmediateChannel {
    fn name(&self) -> &str {
        "immediate"
    }

    async fn send(&self, _message: &SendMessage) -> anyhow::Result<()> {
        Ok(())
    }

    async fn listen(
        &self,
        _tx: tokio::sync::mpsc::Sender<traits::ChannelMessage>,
    ) -> anyhow::Result<()> {
        self.listens.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn supervised_listener_exits_after_listener_returns_when_receiver_is_closed() {
    let channel = Arc::new(ImmediateChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    drop(rx);

    let handle = spawn_supervised_listener_with_health_interval(
        channel_dyn,
        tx,
        0,
        0,
        Duration::from_millis(0),
    );

    handle.await.expect("listener task should finish");
    assert_eq!(channel.listens.load(Ordering::SeqCst), 1);
}
