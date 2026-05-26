use super::*;

#[test]
fn node_control_request_deserializes_method_and_node() {
    let request: NodeControlRequest = serde_json::from_value(serde_json::json!({
        "method": "node.list",
        "node_id": "node-a"
    }))
    .expect("valid request");

    assert_eq!(request.method, "node.list");
    assert_eq!(request.node_id.as_deref(), Some("node-a"));
}
