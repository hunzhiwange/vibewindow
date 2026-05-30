use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

use super::QQChannel;

fn heartbeat_payload(sequence: i64) -> Value {
    let data = if sequence >= 0 { json!(sequence) } else { json!(null) };
    json!({ "op": 1, "d": data })
}

impl QQChannel {
    /// 启动 WebSocket 监听循环。
    #[allow(clippy::too_many_lines)]
    pub(super) async fn listen_gateway(
        &self,
        tx: tokio::sync::mpsc::Sender<crate::app::agent::channels::traits::ChannelMessage>,
    ) -> anyhow::Result<()> {
        tracing::info!("QQ: authenticating...");
        let token = self.get_token().await?;

        tracing::info!("QQ: fetching gateway URL...");
        let gw_url = self.get_gateway_url(&token).await?;

        tracing::info!("QQ: connecting to gateway WebSocket...");
        let (ws_stream, _) = tokio_tungstenite::connect_async(&gw_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let hello = read.next().await.ok_or(anyhow::anyhow!("QQ: no hello frame"))??;
        let hello_data: Value = serde_json::from_str(&hello.to_string())?;
        let heartbeat_interval = hello_data
            .get("d")
            .and_then(|data| data.get("heartbeat_interval"))
            .and_then(Value::as_u64)
            .unwrap_or(41250);

        let intents: u64 = (1 << 25) | (1 << 30);
        let identify = json!({
            "op": 2,
            "d": {
                "token": format!("QQBot {token}"),
                "intents": intents,
                "properties": {
                    "os": "linux",
                    "browser": "vibewindow",
                    "device": "vibewindow",
                }
            }
        });
        write.send(Message::Text(identify.to_string().into())).await?;

        tracing::info!("QQ: connected and identified");

        let mut sequence: i64 = -1;
        let (hb_tx, mut hb_rx) = tokio::sync::mpsc::channel::<()>(1);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(heartbeat_interval));
            loop {
                interval.tick().await;
                if hb_tx.send(()).await.is_err() {
                    break;
                }
            }
        });

        loop {
            tokio::select! {
                _ = hb_rx.recv() => {
                    if write
                        .send(Message::Text(heartbeat_payload(sequence).to_string().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                msg = read.next() => {
                    let msg = match msg {
                        Some(Ok(Message::Text(text))) => text,
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => continue,
                    };

                    let event: Value = match serde_json::from_str(msg.as_ref()) {
                        Ok(event) => event,
                        Err(_) => continue,
                    };

                    if let Some(next_sequence) = event.get("s").and_then(Value::as_i64) {
                        sequence = next_sequence;
                    }

                    let op = event.get("op").and_then(Value::as_u64).unwrap_or(0);
                    match op {
                        1 => {
                            if write
                                .send(Message::Text(heartbeat_payload(sequence).to_string().into()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                            continue;
                        }
                        7 => {
                            tracing::warn!("QQ: received Reconnect (op 7)");
                            break;
                        }
                        9 => {
                            tracing::warn!("QQ: received Invalid Session (op 9)");
                            break;
                        }
                        _ => {}
                    }

                    if op != 0 {
                        continue;
                    }

                    let event_type = event.get("t").and_then(Value::as_str).unwrap_or("");
                    let Some(dispatch_payload) = event.get("d") else {
                        continue;
                    };

                    if let Some(channel_msg) =
                        self.parse_dispatch_message_event(event_type, dispatch_payload).await
                    {
                        if tx.send(channel_msg).await.is_err() {
                            tracing::warn!("QQ: message channel closed");
                            break;
                        }
                    }
                }
            }
        }

        anyhow::bail!("QQ WebSocket connection closed")
    }
}

#[cfg(test)]
#[path = "gateway_tests.rs"]
mod gateway_tests;
