use super::*;
use serde_json::json;

#[test]
fn extra_builds_owned_json_map() {
    let map = extra([("name", json!("demo")), ("ok", json!(true))]);
    assert_eq!(map.get("name"), Some(&json!("demo")));
    assert_eq!(map.get("ok"), Some(&json!(true)));
}

