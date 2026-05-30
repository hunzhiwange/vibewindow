use super::*;

#[test]
fn pb_frame_header_value_returns_match_or_empty_string() {
    let frame = PbFrame {
        seq_id: 1,
        log_id: 2,
        service: 3,
        method: 4,
        headers: vec![PbHeader { key: "k".to_string(), value: "v".to_string() }],
        payload: None,
    };

    assert_eq!(frame.header_value("k"), "v");
    assert_eq!(frame.header_value("missing"), "");
}

#[test]
fn ws_endpoint_response_deserializes_optional_fields() {
    let response: WsEndpointResp = serde_json::from_value(serde_json::json!({
        "code": 0,
        "data": { "URL": "wss://example", "ClientConfig": { "PingInterval": 1000 } }
    }))
    .expect("endpoint response");

    assert_eq!(response.code, 0);
    assert_eq!(
        response.data.expect("data").client_config.expect("config").ping_interval,
        Some(1000)
    );
}
