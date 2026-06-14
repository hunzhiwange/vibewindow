use super::*;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;

fn cache_with_bot_event(event_id: &str) -> Mutex<(VecDeque<String>, HashSet<String>)> {
    let mut order = VecDeque::new();
    let mut lookup = HashSet::new();
    order.push_back(event_id.to_string());
    lookup.insert(event_id.to_string());
    Mutex::new((order, lookup))
}

fn parse_event(value: serde_json::Value) -> OriginalSyncRoomMessageEvent {
    serde_json::from_value(value).expect("valid Matrix room message event")
}

#[tokio::test]
async fn listen_impl_fails_fast_when_otk_conflict_is_already_detected() {
    let channel = MatrixChannel::new(
        "https://matrix.example.com".to_string(),
        "token".to_string(),
        "!room:matrix.example.com".to_string(),
        vec!["*".to_string()],
    );
    channel.otk_conflict_detected.store(true, Ordering::SeqCst);
    let (tx, _rx) = mpsc::channel(1);

    let error = channel.listen_impl(tx).await.unwrap_err();

    assert!(error.to_string().contains("one-time key upload conflict"));
    assert!(error.to_string().contains("paused Matrix sync"));
}

#[tokio::test]
async fn reply_cache_detects_replies_to_recent_bot_events() {
    let event = parse_event(serde_json::json!({
        "type": "m.room.message",
        "event_id": "$reply:matrix.example.com",
        "sender": "@user:matrix.example.com",
        "origin_server_ts": 1u64,
        "content": {
            "msgtype": "m.text",
            "body": "following up",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$bot-message:matrix.example.com"
                }
            }
        }
    }));
    let cache = cache_with_bot_event("$bot-message:matrix.example.com");

    assert!(MatrixChannel::is_reply_to_cached_bot_event(&event, &cache).await);
}

#[tokio::test]
async fn reply_cache_ignores_events_without_reply_relation() {
    let event = parse_event(serde_json::json!({
        "type": "m.room.message",
        "event_id": "$plain:matrix.example.com",
        "sender": "@user:matrix.example.com",
        "origin_server_ts": 1u64,
        "content": {
            "msgtype": "m.text",
            "body": "plain message"
        }
    }));
    let cache = cache_with_bot_event("$bot-message:matrix.example.com");

    assert!(!MatrixChannel::is_reply_to_cached_bot_event(&event, &cache).await);
}
