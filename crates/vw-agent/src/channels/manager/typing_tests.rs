use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct TypingChannel {
    starts: AtomicUsize,
    stops: AtomicUsize,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Channel for TypingChannel {
    fn name(&self) -> &str {
        "typing"
    }

    async fn send(&self, _message: &SendMessage) -> anyhow::Result<()> {
        Ok(())
    }

    async fn listen(
        &self,
        _tx: tokio::sync::mpsc::Sender<traits::ChannelMessage>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn start_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn stop_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        self.stops.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn spawn_scoped_typing_task_stops_typing_when_cancelled() {
    tokio::time::pause();
    let channel = Arc::new(TypingChannel::default());
    let token = CancellationToken::new();

    let handle = spawn_scoped_typing_task(channel.clone(), "recipient".to_string(), token.clone());
    tokio::task::yield_now().await;
    token.cancel();
    handle.await.expect("typing task should finish");

    assert_eq!(channel.stops.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn spawn_scoped_typing_task_refreshes_until_cancelled() {
    tokio::time::pause();
    let channel = Arc::new(TypingChannel::default());
    let token = CancellationToken::new();

    let handle = spawn_scoped_typing_task(channel.clone(), "recipient".to_string(), token.clone());
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(CHANNEL_TYPING_REFRESH_INTERVAL_SECS)).await;
    tokio::task::yield_now().await;
    token.cancel();
    handle.await.expect("typing task should finish");

    assert!(channel.starts.load(Ordering::SeqCst) >= 1);
    assert_eq!(channel.stops.load(Ordering::SeqCst), 1);
}
