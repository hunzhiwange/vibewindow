use crate::tool::{GatewayRedisConfigBundle, GatewayRedisHistoryPage, GatewayRedisSshTunnelConfig};
use serde_json::json;

#[test]
fn redis_tool_defaults_match_gateway_contract() {
    let tunnel = GatewayRedisSshTunnelConfig::default();
    assert!(!tunnel.enabled);
    assert_eq!(tunnel.port, 22);
    assert_eq!(tunnel.timeout_secs, 30);

    let bundle: GatewayRedisConfigBundle = serde_json::from_value(json!({})).expect("valid bundle");
    assert_eq!(bundle.schema_version, 1);
    assert_eq!(bundle.default_load_count, 500);
    assert!(bundle.connections.is_empty());

    let page = GatewayRedisHistoryPage::default();
    assert_eq!(page.limit, 50);
    assert!(!page.has_more);
}
