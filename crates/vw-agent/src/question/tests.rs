use super::*;
use serde_json::json;

#[test]
fn extra_builds_json_map() {
    let map = extra([("request_id", json!("q1"))]);
    assert_eq!(map.get("request_id"), Some(&json!("q1")));
}

