use super::*;
use serde_json::json;

#[test]
fn extra_builds_json_map() {
    let map = extra([("task", json!("tick"))]);
    assert_eq!(map.get("task"), Some(&json!("tick")));
}

