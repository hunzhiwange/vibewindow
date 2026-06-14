use super::*;

#[test]
fn should_refresh_last_recv_refreshes_only_activity_frames() {
    assert!(should_refresh_last_recv(&WsMsg::Binary(vec![1, 2, 3].into())));
    assert!(should_refresh_last_recv(&WsMsg::Ping(vec![1].into())));
    assert!(should_refresh_last_recv(&WsMsg::Pong(vec![2].into())));

    assert!(!should_refresh_last_recv(&WsMsg::Text("event".into())));
    assert!(!should_refresh_last_recv(&WsMsg::Close(None)));
}

#[test]
fn ws_client_config_ping_interval_defaults_are_interpretable() {
    let default_config = WsClientConfig::default();
    assert_eq!(default_config.ping_interval, None);

    let parsed: WsClientConfig =
        serde_json::from_str(r#"{"PingInterval":15}"#).expect("client config");
    assert_eq!(parsed.ping_interval, Some(15));
}

#[test]
fn pb_frame_header_lookup_handles_duplicate_and_missing_keys() {
    let frame = PbFrame {
        seq_id: 1,
        log_id: 2,
        service: 3,
        method: 1,
        headers: vec![
            PbHeader { key: "type".to_string(), value: "event".to_string() },
            PbHeader { key: "type".to_string(), value: "later".to_string() },
        ],
        payload: None,
    };

    assert_eq!(frame.header_value("type"), "event");
    assert_eq!(frame.header_value("missing"), "");
}
